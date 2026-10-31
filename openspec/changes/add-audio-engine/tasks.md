## 1. Workspace

- [x] 1.1 Create Cargo workspace with `crates/stubby_audio` (edition 2024); `stubby_game` lands with the desert-opening work
- [x] 1.2 Install `alsa-lib-devel`; confirm `cpal` enumerates the default output device
- [x] 1.3 Enable clippy `unwrap_used`/`panic` lints on `stubby_audio`

## 2. Spike: fundsp Net vs firewheel (time-boxed, 2 days)

- [x] 2.1 Build sine + clock patch on `fundsp::Net` behind a throwaway boundary
- [x] 2.2 Build the same patch on `firewheel`
- [x] 2.3 Compare: add/remove modules while playing (clicks?), 200-module commit cost, tick timing precision, DSP coverage
- [x] 2.4 Record the decision in `design.md` (D4, Open Questions) and update AGENTS.md stack table
- [ ] 2.5 Listen to both `-- live` runs on real hardware and note any audible clicks
- [x] 2.6 Report the Firewheel silent-channel bug upstream (BillyDM/Firewheel#145)

## 3. Engine core

- [x] 3.1 Define `Command`, `Event`, `ModuleId`, `PortRef`, and signal constants in `stubby_audio::signal`
- [x] 3.2 Validate commands game-side and publish via Firewheel's channel; `rtrb` event rings; `flush` returns `Err` on full (D3)
- [ ] 3.3 Output stream via `firewheel-cpal` (`Engine::start`): implemented, not yet run on a device; run `--example play` and listen
- [x] 3.4 Implement graph edits (add/remove/connect/disconnect); Firewheel frees removed nodes off-thread
- [x] 3.5 Audio clock and `sample_to_instant` from Firewheel's corrected clock (D6)
- [x] 3.6 Run every offline test render under `assert_no_alloc` (the live callback belongs to `firewheel-cpal`; see Open Questions)
- [x] 3.7 Implement `render_offline`
- [x] 3.8 Fade sound sources in on add and out before removal (D7)
- [ ] 3.9 Declick cable connect/disconnect, or decide to accept cable clicks

## 4. Modules

- [x] 4.1 Sine VCO with freq param and V/oct input; offline test for 440 Hz peak and V/oct accuracy
- [x] 4.2 Clock with BPM, gate and phase outputs, tick events; offline test for 20 ticks at 120 BPM over 10 s
- [x] 4.3 Test fan-out and rejection of a second cable to one input
- [x] 4.4 Constant CV module and master-output bus

## 5. Bevy integration

- [ ] 5.0 Decide how Bevy holds the non-`Send` engine (D8, Open Questions)
- [ ] 5.1 `AudioPlugin` inserting `EngineHandle` as a `Resource` and draining events into Bevy messages each frame
- [ ] 5.2 Sample-time → display-time mapping from `SyncPoint`
- [ ] 5.3 Verify a fresh game outputs digital silence
- [ ] 5.4 Stall-test: block the main thread for 200 ms and confirm no underrun
