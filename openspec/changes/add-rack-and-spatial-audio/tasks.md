## Before you start (handoff, 2026-10-31)

This change is ready for any agent to pick up. Read `proposal.md`, `design.md` and
`docs/research/rack-and-spatial-audio.md` first.

**Code today** (`crates/stubby_audio`, all on `main`):
- Modules are still Firewheel nodes in `src/modules.rs`: Sine, Square, Clock,
  Constant, Envelope, VCA. `src/engine.rs` is the game-facing `Engine` (`send` /
  `flush` / `drain_events` / `sample_to_instant`) and the offline renderer.
- `src/thread.rs` runs the engine on its own control thread (`AudioThread`, design
  D9 in `add-audio-engine`). The game uses it through
  `crates/stubby_game/src/audio.rs` (`SynthAudioPlugin`, the `Synth` resource,
  `ClockTick` messages, the F8 / `--audio-demo` test patch).
- Tests: `cargo test --workspace` (engine, thread and plugin tests). The spike that
  produced this change's numbers is `spikes/spike_rack` (its own workspace:
  `cd spikes && cargo run --release -p spike_rack`).

**Keep stable:** the `Engine` and `AudioThread` APIs and the `Synth` plugin surface.
The rack replaces what is *behind* them. Game code and the Jackdaw adapter must not
need changes beyond module kinds and voltage values.

**Decide first (task 0.1):** what hosts the rack. D2 assumes one Firewheel node. We
don't control Firewheel's source: it has an open bug of ours (BillyDM/Firewheel#145),
isn't `Send`, and splits glam versions with Bevy. The alternative is our own thin
`cpal` host. See "Host options" in the research note. Task 2.4 depends on this choice.

**Coordinate with `add-lightning-bug-pulse`:** another agent is adding a
`random_clock` module to `stubby_audio` and a creature pulse in the game. Port
that module to the rack in 2.6, along with the others.

**Conventions:** work in your own git worktree off `main`; claim a task by adding
`(claimed: <agent>)` to its line in the same PR as your first commit for it.

## 0. Decisions

- [ ] 0.1 Host the rack in a Firewheel node (D2) or in our own `cpal` host; record the choice in design.md and update D2 and task 2.4

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
