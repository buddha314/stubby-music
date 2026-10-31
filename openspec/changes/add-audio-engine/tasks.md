## 1. Workspace

- [ ] 1.1 Create Cargo workspace with `crates/stubby_audio` and `crates/stubby_game` (edition 2024)
- [ ] 1.2 Install `alsa-lib-devel`; confirm `cpal` enumerates the default output device
- [ ] 1.3 Enable clippy `unwrap_used`/`panic` lints on `stubby_audio`

## 2. Spike: fundsp Net vs firewheel (time-boxed, 2 days)

- [ ] 2.1 Build sine + clock patch on `fundsp::Net` behind a throwaway boundary
- [ ] 2.2 Build the same patch on `firewheel`
- [ ] 2.3 Compare: add/remove modules while playing (clicks?), 200-module commit cost, tick timing precision, DSP coverage
- [ ] 2.4 Record the decision in `design.md` (D4, Open Questions) and update AGENTS.md stack table

## 3. Engine core

- [ ] 3.1 Define `Command`, `Event`, `ModuleId`, `PortRef`, and signal constants in `stubby_audio::signal`
- [ ] 3.2 Implement rtrb command/event/garbage rings and `EngineHandle::send` returning `Err` on full
- [ ] 3.3 Implement the cpal output stream with bounded command drain per block
- [ ] 3.4 Implement graph edits (add/remove/connect/disconnect) with off-thread drop
- [ ] 3.5 Publish `SyncPoint` and sample counter via `triple_buffer`
- [ ] 3.6 Wrap the callback in `assert_no_alloc` for debug builds
- [ ] 3.7 Implement `render_offline`

## 4. Modules

- [ ] 4.1 Sine VCO with freq param and V/oct input; offline test for 440 Hz peak and V/oct accuracy
- [ ] 4.2 Clock with BPM, gate and phase outputs, tick events; offline test for 20 ticks at 120 BPM over 10 s
- [ ] 4.3 Test fan-out and rejection of a second cable to one input

## 5. Bevy integration

- [ ] 5.1 `AudioPlugin` inserting `EngineHandle` as a `Resource` and draining events into Bevy messages each frame
- [ ] 5.2 Sample-time → display-time mapping from `SyncPoint`
- [ ] 5.3 Verify a fresh game outputs digital silence
- [ ] 5.4 Stall-test: block the main thread for 200 ms and confirm no underrun
