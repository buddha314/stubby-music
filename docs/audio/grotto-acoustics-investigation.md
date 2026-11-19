# Grotto acoustics: findings and remaining work

**Status: 2026-11-18.** This note records the current diagnosis of the opening grotto's occlusion and reverb path. It separates source-level findings, offline tests, and in-game measurements. It does not treat one kind of evidence as proof of another.

## Summary

- **The initial occlusion failure was in the authored-asset-to-runtime bridge.** Jackdaw flattens LOD placed models into mesh entities; acoustic metadata on a prefab root or glTF node was not guaranteed to be present on those runtime mesh entities. AudioNimbus can only include meshes that reach `publish_acoustic_scene` with `AcousticOccluder` attached. The bridge now propagates a prefab root's `AcousticOccluder` to descendant mesh entities after load, and retains support for glTF `"acoustic": "rock"` extras.
- **Occlusion has both synthetic and authored-level evidence.** The offline zero-transmission wall test is allocation-checked. A bounded grotto review measured a large level difference between matched-distance blocked and clear positions. The review does not establish reflection/reverb behavior.
- **The live per-emitter reflection path now has matching-output and short-tail evidence.** In the bounded run, AudioNimbus reported five reflection outputs for five active emitters. After muting the two demo VCAs, the inside meter recorded a post-mute residual at about 52 ms, then silence by about 102 ms. The clear outside point showed a smaller decay over about 135 ms; the blocked point was already near the meter floor. This supports a short parametric reflection tail in the grotto, not a long cave reverb.
- **OpenSpec 4.8 remains incomplete.** An allocation-guarded offline test now processes repeated non-empty per-source reflection snapshots while the worker publishes replacements. Pinned Steam Audio source review shows the active Parametric algorithm reads copied reverb parameters and does not dereference its mutable native IR pointer. Convolution and Hybrid remain unsafe without an owned IR handoff. The listener-centric output is not routed to a shared environmental bus. The level meter measures the whole world bus, not an isolated wet signal. The player has confirmed the grotto reverb is audible at a 0.75 per-emitter wet gain; a controlled live dry/wet comparison is the next validation.

## Occlusion: what the code and measurements establish

### Runtime data path

1. A prefab or model provides an acoustic marker/material.
2. `stubby_world::prefab_presentation::mark_acoustic_rock` puts that material on descendant `Mesh3d` entities. The system handles late-loaded mesh children as well as glTF extras.
3. `stubby_game::glue::publish_acoustic_scene`, after transform propagation, converts marked mesh vertices and triangle indices into world space and publishes the acoustic scene to `stubby_audio`.
4. The AudioNimbus worker builds/updates static geometry and its direct simulation uses transmission rays and material transmission. Direct output reaches the live emitter's `DirectEffect`.

The first unsuccessful grotto checks are consistent with a missing step between Jackdaw's placed prefab and the runtime acoustic mesh set; visual presence of a boulder alone did not prove acoustic registration. The code changes now cover both ways of authoring the material: `AcousticOccluder` on the prefab root (copied to descendant mesh entities) and glTF node extras. This is the specific bridge repair, not an additional raycast occlusion system.

### Evidence

- `audionimbus_static_wall_transmission_attenuates_live_emitter` builds a synthetic two-triangle wall with zero transmission, renders under `assert_no_alloc`, and asserts blocked RMS is below 80% of clear RMS. A prior recorded run measured clear RMS `0.25694` and blocked RMS `0`. The assertion and callback allocation guard are in `crates/stubby_audio/tests/engine.rs`.
- The bounded `--grotto-audio-review` captured the same source at `[69.4393, 0.65, -59.1967]` while moving the listener between:
  - inside: `[70.5, 1.6, -64.5]`;
  - blocked outside: `[76.89, 1.6, -74.48]`;
  - clear outside: `[69.44, 1.6, -42.2]`.
- Its recorded steady peak readings were about `−97.4 dBFS` at the blocked position and `−33.7 dBFS` at the clear position (about 63.7 dB difference). This supports strong attenuation along that authored blocked path and recovery on the clear path.
- A follow-up bounded review emits both demo voices for 1.5 s at each position, then sets their VCA gains to zero and samples the latest-block peak every 50 ms. The trace reported `outputs=5 matching_emitters=5` after entering the grotto. At the inside listener, the last pre-mute reading was about `−8.7 dBFS`; the first post-mute reading at about +52 ms was `−42.6 dBFS`, followed by silence by +102 ms. At the clear outside listener, readings were about `−72.9 dBFS` at +83 ms and `−100.1 dBFS` at +135 ms, then silence by +200 ms. At the blocked outside listener, the pre-mute level was already about `−97.4 dBFS` and the first post-mute sample was silent. These are world-bus peaks, not an isolated wet stem; they show a short post-mute decay at the unblocked views, while a reflection-disabled control is still needed to attribute all residual energy exclusively to reflections.
- Screenshots and raw events from that bounded run were written to `/tmp/grotto-audio-review/` on the test machine; runtime logs are under `/tmp/jackdaw-flicker-grotto-audio-review-final.log` and related named logs. These are machine-local artifacts, not versioned test fixtures.

### Limits

This is one source, one grotto, and one set of positions. It does not prove all boulder instances or all paths are correct, acoustic materials are physically calibrated, or occlusion is smooth during arbitrary movement. It also does not replace the level-authoring audit for future caves, walls, or buildings.

## Reverb: package behavior and integration risks

### Confirmed package behavior

The current dependency is AudioNimbus 0.17 with Steam Audio's native SDK. Its reflection/reverb simulation runs separately from the Firewheel audio callback and publishes results through `SharedSimulationOutput`. AudioNimbus leaves application of the result to the audio buffer to the host integration.

The reflection output is backed by `arc-swap`. In the installed `arc-swap 1.9.5` source, `load_full()` delegates to `load()` and unwraps its guard; it is not an independent real-time-safe alternative. The per-thread debt-list path can lazily create a node with `Box::leak` when no reusable node exists. Also, AudioNimbus uses `object_pool::ReusableOwned`; returning a released object to that pool takes a `parking_lot::Mutex`. Therefore, loading or dropping the final pooled snapshot in the audio callback can violate the project's no-allocation/no-lock callback rule.

### Current ownership design

The current implementation has the simulation worker read `SharedSimulationOutput` away from the callback and publish the latest snapshot via `triple_buffer`. The callback obtains a borrowed latest value from the preallocated single-consumer output and applies AudioNimbus's parametric `ReflectionEffect` and Ambisonics decode using buffers allocated when the processor is constructed. The producer-side mutex is used by the simulation/control side; it is not acquired by the audio callback. The worker pauses reflection simulation until it has received its shared simulation inputs, avoiding the initial missing-input warning burst.

This design is intended to avoid callback-side ArcSwap access and pooled snapshot destruction. An allocation-guarded integration test now feeds repeated non-empty source snapshots from a live AudioNimbus worker through the same binaural/reflection processing method used by the Firewheel node while the worker replaces snapshots. It asserts the active algorithm is Parametric, output stays finite, and processing allocates nothing. This closes the specific non-empty Parametric snapshot test gap; it does not validate the native device callback under load or device restarts.

The unsafe `Sync` marker in AudioNimbus's reflection output is still a concern for its native impulse-response pointer. Inspection of the exact pinned Steam Audio source shows the simulator writes that pointer on every reflection update. The active Parametric `ReflectionEffect` path uses the copied `reverbTimes` values and does not dereference the IR pointer. Convolution and Hybrid effects consume the pointer and must not be enabled unless the integration adds an owned, callback-safe IR handoff. This is a source-backed scope limit, not a general claim that the upstream type is safe.

### Existing and missing evidence

- The player confirmed that reverb is audible in the live grotto using `cargo run -p
  stubby_jackdaw --features dev -- --grotto-audio-review=/tmp/grotto-audionimbus-review`.
  The session's per-emitter reflection wet gain was 0.75. The player had previously
  reported the boulder occlusion was clearly audible. This supports audibility at the
  current tuning, but does not establish that the tail length or level is artistically
  final, nor does it isolate the listener-centric reverb bus.

- The previous synthetic closed 8 m box test ran the reflection simulation for an impulse and silence and verified finite, non-zero parametric output/tail. This proves the SDK simulation and effect can produce a result for that fixture.
- A prior callback test with reflection snapshot loading failed the allocation guard; removing callback-side snapshot loading made the guard pass. The new focused test exercises non-empty Parametric snapshots through the binaural processor under the allocation guard; actual device callback scheduling, load and device restart remain unvalidated.
- A later offline worker test exposed the same initial missing-input warning burst. Pausing the reflection worker until its first `set_input` call removed the burst in that test. The game still needs a bounded launch check to confirm its initialized reflection path behaves cleanly.
- A synthetic material probe confirms that the game's generic and rock transmission bands reach the AudioNimbus direct simulation and that rock transmission is lower in all three bands. An end-to-end RMS probe on the same simple wall fixture measured generic `0.02268` versus rock `0.01917` (about 15% lower); that fixture does not support claiming a large or perceptually sufficient material separation. Keep the authored grotto listening result as the evidence for its particular blocked path.
- The review driver now includes a controlled source mute and 50 ms world-peak sampling. It detects a short post-mute decay, but without a reflection-disabled control it cannot attribute every residual sample solely to the reflection effect.
- AudioNimbus also produces a listener-centric `output.listener`; the current game processor reads only `output.sources[emitter]`. There is no shared listener environmental bus yet. The player's listening confirmation is recorded above but is not part of the machine review log.

### Listener reverb is a distinct signal path

Inspection of the pinned AudioNimbus 0.17.0 source resolves what this output means. In
`wiring/step/reflections_reverb.rs`, `ReflectionsReverbOutput` contains both a
`sources: HashMap<SourceId, ...>` described as per-source reflection effect parameters
and an optional `listener` described as listener-centric reverb. The simulation step
populates these independently from each source and the configured listener source. The
current game callback reads only `sources[emitter]`; it never reads `listener`.

AudioNimbus's FMOD and Wwise adapter docs describe listener-centric reverb as a source
placed at the listener, with reflections simulated there. In this game, the intended
use is consequently a shared environmental wet bus fed by the already mixed dry world
signal, while per-source reflection processing remains directional. Whether using both
paths double-counts the same early energy, and the correct wet gain/latency, still need
an A/B render and listening review. The listener result is not a ready-made wet audio
signal: the host must run `ReflectionEffect` on the bus input and decode its Ambisonic
output, just as the per-source path currently does.

This narrows the implementation question: route `output.listener` into a Firewheel
listener bus after emitter dry signals are summed, with a no-allocation handoff that
retains native reflection parameters off the callback; compare per-source reflections
alone against per-source plus listener reverb in the same grotto. Do not add another
game-owned reverb algorithm. The listener source is currently initialized at the
default pose and its reflection parameters become available only after a simulation
frame completes, which also explains why the first empty snapshot and startup warning
burst need an explicit readiness/initial-input policy.

## Why this took several iterations

The work involved two independent bridges and their evidence was easy to conflate:

1. The **scene bridge** decides whether authored geometry/material reaches AudioNimbus at all. A visually correct prefab can still lack acoustic metadata on its runtime meshes after Jackdaw's flattening/import steps.
2. The **reflection bridge** moves simulation results to the audio callback. A reflection API probe can succeed while no live emitter receives those results; a callback can render safely while receiving `None` or a non-matching result.
3. The **review signal** must isolate the feature being checked. A continuous meter validates loudness/occlusion at a position but cannot prove a reverb tail. Reverb needs a controlled source onset/stop and a dry-versus-wet decay capture or a listening review with repeatable source and listener positions.

These boundaries should remain explicit in future reports. Record source/listener coordinates, geometry publication/material counts, reflection output and matching counts, callback allocation result, and decay evidence as separate observations.

## Next validation sequence

1. **Player A/B is now the next step:** keep the game and a sounding burbler running. In a second terminal run `cargo run -p stubby_rules_server`, open `http://127.0.0.1:7878/`, and use **Spatial Audio → Reflection wet gain**. At the same listener position inside the grotto, apply `0.00`, listen, then apply `0.75` and listen again. Repeat outside behind the boulder without moving the source. This separates the per-emitter reflection contribution from the clearly audible occlusion path without changing source level or listener position. The setting is session-only; the panel tunes per-emitter reflections, not the unrouted listener-centric bus.
2. If the A/B confirms the current effect is useful and not too subtle, implement and compare AudioNimbus's listener-centric result on a shared Firewheel environmental bus against the current per-emitter path. Keep the two paths independently switchable during review to detect duplicated early energy.
3. Run a bounded game launch after the worker initialization change and record whether the initial warning burst is gone. Then validate device/sample-rate restart and CPU cost.
4. Broaden material/path coverage and implement the remaining voice-pool, audibility-ranking, speaker-fallback, and spatial-budget tasks. These do not need to block the immediate wet/dry listening comparison.
5. **Existing player confirmation:** wet gain 0.75 has audible grotto reverb and boulder occlusion is clearly audible. This does not replace the controlled dry/wet A/B or decide whether the listener-centric bus improves the result.

## Package references

- [AudioNimbus simulation threading](https://docs.rs/audionimbus/latest/audionimbus/simulation/)
- [AudioNimbus crate documentation](https://docs.rs/audionimbus/latest/audionimbus/)
- [ArcSwap `ArcSwapAny` API](https://docs.rs/arc-swap/latest/arc_swap/struct.ArcSwapAny.html)
- [ArcSwap debt-list source](https://docs.rs/arc-swap/latest/src/arc_swap/debt/list.rs.html)
- [triple_buffer 9 API](https://docs.rs/triple_buffer/9.0.0/triple_buffer/)
- [rtrb API](https://docs.rs/rtrb/latest/rtrb/)
