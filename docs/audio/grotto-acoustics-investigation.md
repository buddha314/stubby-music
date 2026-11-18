# Grotto acoustics: findings and remaining work

**Status: 2026-11-18.** This note records the current diagnosis of the opening grotto's occlusion and reverb path. It separates source-level findings, offline tests, and in-game measurements. It does not treat one kind of evidence as proof of another.

## Summary

- **The initial occlusion failure was in the authored-asset-to-runtime bridge.** Jackdaw flattens LOD placed models into mesh entities; acoustic metadata on a prefab root or glTF node was not guaranteed to be present on those runtime mesh entities. AudioNimbus can only include meshes that reach `publish_acoustic_scene` with `AcousticOccluder` attached. The bridge now propagates a prefab root's `AcousticOccluder` to descendant mesh entities after load, and retains support for glTF `"acoustic": "rock"` extras.
- **Occlusion has both synthetic and authored-level evidence.** The offline zero-transmission wall test is allocation-checked. A bounded grotto review measured a large level difference between matched-distance blocked and clear positions. The review does not establish reflection/reverb behavior.
- **The reflection simulation/effect works in isolation, but integrated reverb is not yet demonstrated.** An earlier synthetic 8 m room test produced a finite, non-zero tail. The current live path moves the latest reflection snapshot through `triple_buffer` before the audio callback reads it, addressing a concrete real-time hazard in AudioNimbus's `arc-swap` output access. No retained result from the current handoff proves that live reflection parameters match the grotto emitter, and the grotto review's level meter is not a reverb-tail measurement.
- **The immediate remaining blocker is verification of the complete handoff and rendered tail.** Confirm non-empty, emitter-matched reflection parameters in the bounded live review, then measure or listen to a source stop and its decay inside the grotto. Keep OpenSpec 4.8 and grotto task 4.1 open until this is done.

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
- Its recorded steady peak readings were about `−97.4 dBFS` at the blocked position and `−33.7 dBFS` at the clear position (about 63.7 dB difference). This supports strong attenuation along that authored blocked path and recovery on the clear path. The inside readings are not a controlled dry/wet comparison, and the level script does not measure a decay after stopping the source.
- Screenshots and raw events from that bounded run were written to `/tmp/grotto-audio-review/` on the test machine; runtime logs are under `/tmp/jackdaw-flicker-grotto-audio-review-final.log` and related named logs. These are machine-local artifacts, not versioned test fixtures.

### Limits

This is one source, one grotto, and one set of positions. It does not prove all boulder instances or all paths are correct, acoustic materials are physically calibrated, or occlusion is smooth during arbitrary movement. It also does not replace the level-authoring audit for future caves, walls, or buildings.

## Reverb: package behavior and integration risks

### Confirmed package behavior

The current dependency is AudioNimbus 0.17 with Steam Audio's native SDK. Its reflection/reverb simulation runs separately from the Firewheel audio callback and publishes results through `SharedSimulationOutput`. AudioNimbus leaves application of the result to the audio buffer to the host integration.

The reflection output is backed by `arc-swap`. In the installed `arc-swap 1.9.5` source, `load_full()` delegates to `load()` and unwraps its guard; it is not an independent real-time-safe alternative. The per-thread debt-list path can lazily create a node with `Box::leak` when no reusable node exists. Also, AudioNimbus uses `object_pool::ReusableOwned`; returning a released object to that pool takes a `parking_lot::Mutex`. Therefore, loading or dropping the final pooled snapshot in the audio callback can violate the project's no-allocation/no-lock callback rule.

### Current ownership design

The current uncommitted implementation has the simulation worker read `SharedSimulationOutput` away from the callback and publish the latest snapshot via `triple_buffer`. The callback obtains a borrowed latest value from the preallocated single-consumer output and applies AudioNimbus's parametric `ReflectionEffect` and Ambisonics decode using buffers allocated when the processor is constructed. The producer-side mutex is used by the simulation/control side; it is not acquired by the audio callback.

This design is intended to avoid callback-side ArcSwap access and pooled snapshot destruction. It is a code-path argument, not yet a complete integration result: run the focused test with the current revision under `assert_no_alloc`, verify reflection source/output matching, and verify ownership remains valid while snapshots are replaced and emitters are removed. Do not mark the whole reflection path real-time safe based only on the triple-buffer API; exercise the exact native effect path.

### Existing and missing evidence

- The previous synthetic closed 8 m box test ran the reflection simulation for an impulse and silence and verified finite, non-zero parametric output/tail. This proves the SDK simulation and effect can produce a result for that fixture.
- A prior callback test with reflection snapshot loading failed the allocation guard; removing callback-side snapshot loading made the guard pass. This isolated the unsafe access path. The latest implementation changes the transfer to a triple buffer, but its current full test output was not retained, so this note does not claim that the newest code passed.
- A bounded live launch logged repeated initial errors `missing shared simulation inputs: SimulationFlags(REFLECTIONS)` before the game had published its first simulation input. The log sequence places those messages before the first game catalog-load message. This demonstrates a startup race/noisy initial worker state in that run; it does not establish whether reflection outputs later reached a matching emitter. Filter expected startup diagnostics only after verifying the worker's input/output lifecycle, or fix initialization ordering in a way supported by AudioNimbus's API.
- The trace added to the current branch is intended to report output count and matches against emitter `PortRef`s when those counts change. The available retained grotto event file contains only listener positions and peak levels, not those trace values. Capture and retain the trace from the next rebuilt review.
- No current in-game test stops a sound and measures its tail. The meter remains active throughout the three views, so its values cannot distinguish direct sound from reflection energy.

## Why this took several iterations

The work involved two independent bridges and their evidence was easy to conflate:

1. The **scene bridge** decides whether authored geometry/material reaches AudioNimbus at all. A visually correct prefab can still lack acoustic metadata on its runtime meshes after Jackdaw's flattening/import steps.
2. The **reflection bridge** moves simulation results to the audio callback. A reflection API probe can succeed while no live emitter receives those results; a callback can render safely while receiving `None` or a non-matching result.
3. The **review signal** must isolate the feature being checked. A continuous meter validates loudness/occlusion at a position but cannot prove a reverb tail. Reverb needs a controlled source onset/stop and a dry-versus-wet decay capture or a listening review with repeatable source and listener positions.

These boundaries should remain explicit in future reports. Record source/listener coordinates, geometry publication/material counts, reflection output and matching counts, callback allocation result, and decay evidence as separate observations.

## Next validation sequence

1. Rebase this branch on current `origin/main`, rebuild with the pinned AudioNimbus SDK, and run the focused `stubby_audio` spatial test(s) under the callback allocation guard.
2. Run the bounded standalone grotto review. Retain the log lines for initial worker input, nonzero reflection output count, and a matching emitter `PortRef`; separately retain the occlusion position/level events.
3. Add a controlled reverb review: same source and listener, fixed geometry, short repeatable tone or impulse followed by silence, and a captured level envelope. Compare with reflections disabled or source/listener in a dry control position. Confirm the effect has a measurable decay after direct output stops and is audible without clipping or an unexplained constant bed.
4. Re-run the callback allocation test while snapshots are being replaced, and exercise emitter removal/re-addition. Verify shutdown joins the reflection worker and retained native handles are dropped off the callback.
5. Only then update OpenSpec `add-rack-and-spatial-audio` 4.8 and grotto task 4.1; keep 4.6's authored-level check separate from the reverb acceptance.

## Package references

- [AudioNimbus simulation threading](https://docs.rs/audionimbus/latest/audionimbus/simulation/)
- [AudioNimbus crate documentation](https://docs.rs/audionimbus/latest/audionimbus/)
- [ArcSwap `ArcSwapAny` API](https://docs.rs/arc-swap/latest/arc_swap/struct.ArcSwapAny.html)
- [ArcSwap debt-list source](https://docs.rs/arc-swap/latest/src/arc_swap/debt/list.rs.html)
- [triple_buffer 9 API](https://docs.rs/triple_buffer/9.0.0/triple_buffer/)
- [rtrb API](https://docs.rs/rtrb/latest/rtrb/)
