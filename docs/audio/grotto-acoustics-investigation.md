# Grotto acoustics: findings and remaining work

**Status: 2026-11-18.** This note records the current diagnosis of the opening grotto's occlusion and reverb path. It separates source-level findings, offline tests, and in-game measurements. It does not treat one kind of evidence as proof of another.

## Summary

- **The initial occlusion failure was in the authored-asset-to-runtime bridge.** Jackdaw flattens LOD placed models into mesh entities; acoustic metadata on a prefab root or glTF node was not guaranteed to be present on those runtime mesh entities. AudioNimbus can only include meshes that reach `publish_acoustic_scene` with `AcousticOccluder` attached. The bridge now propagates a prefab root's `AcousticOccluder` to descendant mesh entities after load, and retains support for glTF `"acoustic": "rock"` extras.
- **Occlusion has both synthetic and authored-level evidence.** The offline zero-transmission wall test is allocation-checked. A bounded grotto review measured a large level difference between matched-distance blocked and clear positions. The review does not establish reflection/reverb behavior.
- **The live per-emitter reflection path now has matching-output and short-tail evidence.** In the bounded run, AudioNimbus reported five reflection outputs for five active emitters. After muting the two demo VCAs, the inside meter recorded a post-mute residual at about 52 ms, then silence by about 102 ms. The clear outside point showed a smaller decay over about 135 ms; the blocked point was already near the meter floor. This supports a short parametric reflection tail in the grotto, not a long cave reverb.
- **OpenSpec 4.8 remains incomplete.** The callback allocation test has not exercised non-empty reflection parameters while snapshots are replaced, and AudioNimbus's listener-centric output is not routed to a shared environmental bus. The level meter measures the whole world bus, not an isolated wet signal; subjective listening is not captured in the review.

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

The current implementation has the simulation worker read `SharedSimulationOutput` away from the callback and publish the latest snapshot via `triple_buffer`. The callback obtains a borrowed latest value from the preallocated single-consumer output and applies AudioNimbus's parametric `ReflectionEffect` and Ambisonics decode using buffers allocated when the processor is constructed. The producer-side mutex is used by the simulation/control side; it is not acquired by the audio callback.

This design is intended to avoid callback-side ArcSwap access and pooled snapshot destruction. The live review confirms that output IDs match active emitters, but it does not prove callback safety while a non-empty snapshot is read and replaced. The existing offline live-emitter allocation test can see an empty snapshot; the isolated synthetic tail test is not guarded by `assert_no_alloc`. Add a focused test that feeds real or retained reflection parameters through the exact Firewheel processor while replacing snapshots and removing emitters. Do not mark the whole reflection path real-time safe based only on the triple-buffer API.

### Existing and missing evidence

- The previous synthetic closed 8 m box test ran the reflection simulation for an impulse and silence and verified finite, non-zero parametric output/tail. This proves the SDK simulation and effect can produce a result for that fixture.
- A prior callback test with reflection snapshot loading failed the allocation guard; removing callback-side snapshot loading made the guard pass. The current transfer uses `triple_buffer`, but the existing allocation test still does not exercise a non-empty reflection snapshot in the callback.
- The latest bounded launch logged 800 initial `missing shared simulation inputs: SimulationFlags(REFLECTIONS)` messages before AudioNimbus reported its first `outputs=5 matching_emitters=5` snapshot. The outputs later became non-empty and matched all active emitters, so the startup warnings do not mean the reflection path remained disconnected. Reduce or avoid the initial error burst using an AudioNimbus-supported initialization sequence.
- The review driver now includes a controlled source mute and 50 ms world-peak sampling. It detects a short post-mute decay, but without a reflection-disabled control it cannot attribute every residual sample solely to the reflection effect.
- AudioNimbus also produces a listener-centric `output.listener`; the current game processor reads only `output.sources[emitter]`. There is no shared listener environmental bus yet, and subjective listening is not captured.

## Why this took several iterations

The work involved two independent bridges and their evidence was easy to conflate:

1. The **scene bridge** decides whether authored geometry/material reaches AudioNimbus at all. A visually correct prefab can still lack acoustic metadata on its runtime meshes after Jackdaw's flattening/import steps.
2. The **reflection bridge** moves simulation results to the audio callback. A reflection API probe can succeed while no live emitter receives those results; a callback can render safely while receiving `None` or a non-matching result.
3. The **review signal** must isolate the feature being checked. A continuous meter validates loudness/occlusion at a position but cannot prove a reverb tail. Reverb needs a controlled source onset/stop and a dry-versus-wet decay capture or a listening review with repeatable source and listener positions.

These boundaries should remain explicit in future reports. Record source/listener coordinates, geometry publication/material counts, reflection output and matching counts, callback allocation result, and decay evidence as separate observations.

## Next validation sequence

1. Add a reflection-disabled control run with the same source mute timing and positions; compare post-mute envelopes to attribute the measured tail to the AudioNimbus reflection effect rather than the rest of the output chain.
2. Add a focused callback allocation test that consumes non-empty reflection parameters while snapshots are replaced and emitters are removed/re-added. Verify retained native handles are released away from the callback and shutdown joins the worker.
3. Route or explicitly exclude AudioNimbus's listener-centric output. If included, design the listener environmental bus so it does not duplicate per-emitter reflections, then add an offline comparison test.
4. Reduce the initial missing-input error burst with a package-supported initialization sequence, then rerun the bounded review and retain the reflection-match trace, occlusion readings, and tail envelope.
5. Have a listener confirm the grotto's short tail is audible and useful. Keep OpenSpec 4.8 and grotto task 4.1 open until the control, callback ownership test, bus decision, and listening review pass; keep 4.6's authored-level occlusion evidence separate.

## Package references

- [AudioNimbus simulation threading](https://docs.rs/audionimbus/latest/audionimbus/simulation/)
- [AudioNimbus crate documentation](https://docs.rs/audionimbus/latest/audionimbus/)
- [ArcSwap `ArcSwapAny` API](https://docs.rs/arc-swap/latest/arc_swap/struct.ArcSwapAny.html)
- [ArcSwap debt-list source](https://docs.rs/arc-swap/latest/src/arc_swap/debt/list.rs.html)
- [triple_buffer 9 API](https://docs.rs/triple_buffer/9.0.0/triple_buffer/)
- [rtrb API](https://docs.rs/rtrb/latest/rtrb/)
