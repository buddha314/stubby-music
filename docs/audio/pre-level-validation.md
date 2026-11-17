# AudioNimbus work before authored acoustic levels

Production caves, buildings, and enclosed spaces are not required to make useful
progress on spatial audio. The engine can validate its transport and algorithms with
small deterministic test scenes. Keep those fixtures in diagnostics or offline
validation; do not ship duplicate meshes into gameplay levels just for acoustics.

## Work that is unblocked now

1. **Finish live-path validation (OpenSpec 4.1c).** Measure callback allocations,
   average and tail CPU, the configured block latency, and behavior after an audio
   device/sample-rate restart. Keep the existing bounded launch watchdog. The
   intentional 110 Hz `--audio-demo` tone is useful for steady processing checks;
   clock blips reveal block edges and transient problems.
2. **Exercise the geometry bridge with generated fixtures (4.6).** Build a test-only
   open plane and a closed box from a few triangles, using generic and rock materials.
   Compare a clear source/listener path, a path through one wall, and a path inside
   the box. Verify direct gain/transmission changes and recovery when the path clears.
   This tests the Bevy snapshot, transforms, mesh/material conversion, and native
   scene rebuild before any authored cave exists.
3. **Prototype reflections/reverb (4.8) against the closed-box fixture.** The current
   worker enables direct simulation only. AudioNimbus supports a separate reflections
   simulation and effect path, but it requires reflection settings, source parameters,
   listener inputs, and an audio-graph mixer/bus. A short synthetic room can validate
   the worker-to-audio transfer and audible decay; it cannot certify the eventual
   level's room response.
4. **Measure movement and staleness (4.10).** Move source and listener transforms
   through the fixture at controlled intervals. Record simulation update rate,
   stale-result age, worker CPU/memory, and audio-thread CPU. Test several emitter
   counts without depending on authored assets.
5. **Record supported runtime policy (4.7 / 4.9).** Confirm HRTF enable/disable and
   speaker panning fallback, then decide whether Ambisonic buses or baked probes are
   needed for the game's authored content. These API and packaging checks do not need
   production geometry.

## Work that still needs authored spaces

- Listening reviews of material choices against the actual art and gameplay distances.
- Checking whether cave, tunnel, building, and doorway meshes represent their visible
  boundaries, including openings and thin walls.
- Tuning material absorption/transmission and reflection quality for the final scale.
- Confirming the sound communicates useful location cues during normal play.

Do not mark the acoustic-level listening requirement complete based on a synthetic
fixture. Keep fixture results and authored-level results as separate evidence.
