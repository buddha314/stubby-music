## 1. Workspace

- [x] 1.1 Create Cargo workspace with `crates/stubby_audio` (edition 2024); `stubby_game` lands with the desert-opening work
- [x] 1.2 Install `alsa-lib-devel`; confirm `cpal` enumerates the default output device
- [x] 1.3 Enable clippy `unwrap_used`/`panic` lints on `stubby_audio`

## 2. Spike: fundsp Net vs firewheel (time-boxed, 2 days)

- [x] 2.1 Build sine + clock patch on `fundsp::Net` behind a throwaway boundary
- [x] 2.2 Build the same patch on `firewheel`
- [x] 2.3 Compare: add/remove modules while playing (clicks?), 200-module commit cost, tick timing precision, DSP coverage
- [x] 2.4 Record the decision in `design.md` (D4, Open Questions) and update AGENTS.md stack table
- [x] 2.5 ~~Listen to both spike `-- live` runs~~: superseded by the engine-core live tests (3.3, 4.6), heard clean on `breq`
- [x] 2.6 Report the Firewheel silent-channel bug upstream (BillyDM/Firewheel#145)

## 3. Engine core

- [x] 3.1 Define `Command`, `Event`, `ModuleId`, `PortRef`, and signal constants in `stubby_audio::signal`
- [x] 3.2 Validate commands game-side and publish via Firewheel's channel; `rtrb` event rings; `flush` returns `Err` on full (D3)
- [x] 3.3 Output stream via `firewheel-cpal` (`Engine::start`): run with `--example play` on `breq` (2026-10-31); sine fade-in, octave jump and fade-out clean, no crackle
- [x] 3.4 Implement graph edits (add/remove/connect/disconnect); Firewheel frees removed nodes off-thread
- [x] 3.5 Audio clock and `sample_to_instant` from Firewheel's corrected clock (D6)
- [x] 3.6 Run every offline test render under `assert_no_alloc` (the live callback belongs to `firewheel-cpal`; see Open Questions)
- [x] 3.7 Implement `render_offline`
- [x] 3.8 Fade sound sources in on add and out before removal (D7)
- [x] 3.9 ~~Declick cable connect/disconnect~~: moved to `add-rack-and-spatial-audio` task 3.4 (typed cable fades)

## 4. Modules

- [x] 4.1 Sine VCO with freq param and V/oct input; offline test for 440 Hz peak and V/oct accuracy
- [x] 4.2 Clock with BPM, gate and phase outputs, tick events; offline test for 20 ticks at 120 BPM over 10 s
- [x] 4.3 Test fan-out and rejection of a second cable to one input
- [x] 4.4 Constant CV module and master-output bus
- [x] 4.5 Envelope and VCA modules; `play` example patches clock → envelope → VCA so the clock is audible
- [x] 4.6 Listen to the clock blips in `--example play`: heard on `breq` (2026-10-31), clean and on the beat

## 5. Bevy integration

- [x] 5.0 Decide how Bevy holds the non-`Send` engine: a control thread behind a `Send` handle (D9)
- [x] 5.1 `SynthAudioPlugin` inserting the `Synth` resource (an `AudioThread`), turning ticks into `ClockTick` messages each frame and committing in `Last`; also wired into the Jackdaw adapter
- [x] 5.2 Sample-time → wall-time mapping: each `ClockTick` carries `at`, predicted from an audio-clock snapshot
- [x] 5.3 Verify a fresh game outputs digital silence (`a_fresh_game_is_silent`)
- [x] 5.4 Stall test: the engine keeps rendering through a 200 ms game-thread stall (`engine_keeps_running_while_the_game_thread_stalls`)
- [ ] 5.5 Hear the game's audio live: `cargo run -p stubby_game -- --audio-demo` (or F8), and in Jackdaw Play
  Also run with `--module-demo` to check that the clock module's light flashes on the beat you hear (`add-module-asset-binding` 2.5).
  Blocked on: the `add-lightning-bug-pulse` branch updating from `main` (it overlaps the audio wiring in six files). Then rebuild the Jackdaw game and play "Desert opening + audio demo".
