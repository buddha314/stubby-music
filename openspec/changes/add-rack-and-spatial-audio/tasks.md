## 1. Spike (done)

- [x] 1.1 Rack prototype: per-sample, 1-sample cables, feedback in every voice; cost at 50–400 modules
- [x] 1.2 HRTF vs panning cost at 1–64 moving emitters; confirm no audio-thread allocation
- [x] 1.3 HRTF FFT size versus the 256-frame block (2×128 halves p99; 1×256 panics)
- [x] 1.4 Port the IRCAM HRTF node to Firewheel 0.14 (only the `glam-32` feature needed)

## 2. Rack engine

- [ ] 2.1 `signal`: VCV voltage constants, Schmitt trigger, pulse generator, NaN guard
- [ ] 2.2 `Module` trait (per sample), port types (audio/CV/gate), 16-channel cable value
- [ ] 2.3 Double-buffered slot store; cable table; order-independence test
- [ ] 2.4 `RackNode` Firewheel host: stems out, lock-free edit queue in, removed modules back out
- [ ] 2.5 Wire `Engine::send`/`flush` to the rack; remove `EngineError::Cycle`; feedback test
- [ ] 2.6 Port Sine, Square, Clock and Constant to the rack in volts; update the engine tests
- [ ] 2.7 Determinism test (two renders, 200 modules) and budget benchmark (< 0.8 ms mean)

## 3. Control surface

- [ ] 3.1 Parameter smoothing (one-pole, 10 ms default) with a zipper-noise test
- [ ] 3.2 Lights and meters as atomics; clock gate light; game-side read API
- [ ] 3.3 Scope rings for probed ports
- [ ] 3.4 Typed cable fades: audio-to-emitter fades, gate/CV instant
- [ ] 3.5 Patch serialization (serde) and a round-trip render test

## 4. Spatial emission

- [ ] 4.1 Vendor the HRTF node (Irc1040 plus profile selection) with its license notice; FFT 2×128; reject `slice_count = 1`
- [ ] 4.2 Emitter registry: module output, world position, hearing radius
- [ ] 4.3 Preallocated HRTF and pan voice pools; routing stems to voices
- [ ] 4.4 Audibility ranking with hysteresis; 20 ms crossfade on tier change
- [ ] 4.5 Distance gain, radius fade-out, air-absorption low-pass
- [ ] 4.6 Occlusion factor input → cutoff and gain, smoothed 100 ms
- [ ] 4.7 Listener transform input; speaker fallback (HRTF off)
- [ ] 4.8 Spatial budget benchmark: 16 HRTF + 32 pan, moving, < 0.8 ms mean

## 5. Validation

- [ ] 5.1 Run both benchmarks on `breq` (laptop) and record results here
- [ ] 5.2 Listening test: find a hidden Square VCO by ear with headphones, then on speakers
- [ ] 5.3 Reconcile with `add-audio-engine` before archiving either (design.md)
