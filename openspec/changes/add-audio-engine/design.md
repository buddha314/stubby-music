## Context

Greenfield repo. The game is a modular synth expressed as a factory, so the audio
graph changes continuously during play. Bevy 0.20's built-in audio is rodio-based
asset playback with no patch graph. The full library survey is in
`docs/research/audio-engine.md`.

## Goals / Non-Goals

**Goals:**
- Realtime-safe, runtime-editable DSP graph on its own thread.
- A game-facing API that hides which DSP library is underneath.
- Sample-accurate clock events the game can sync visuals and factory logic to.
- Offline rendering, so DSP is unit-testable in CI with no audio device.

**Non-Goals:**
- Spatial/3D audio for modules (later change).
- Audio input, MIDI, or plugin hosting (VST/CLAP).
- Running audio in a separate OS process.
- Module UI, patch cables in 3D, factory mechanics.

## Decisions

**D1 — Separate crate, no Bevy dependency.** `stubby_audio` exposes
`AudioEngine::start(config) -> (EngineHandle, EngineEvents)`, plus
`render_offline(patch, frames)`. The Bevy plugin in `stubby_game` wraps the handles
as a `Resource`. *Alternative:* `bevy_seedling` (nodes as ECS entities). Rejected as
the foundation because it couples graph topology to the ECS. It may still inform the
Bevy-side API.

**D2 — Thread, not process.** The cpal callback thread renders. *Alternative:* a
synth-server process (SuperCollider-style) for crash isolation. Rejected for now
because of IPC and lifecycle cost. The command protocol is plain data, so moving to
a process later stays possible.

**D3 — Rings and snapshots.** `rtrb` SPSC rings for `Command` (game→audio),
`Event` (audio→game), and `Garbage` (audio→drop thread). `triple_buffer` carries
latest-value state (sample position, clock phases, meters). No `Mutex` anywhere the
audio thread can see. *Alternative:* `crossbeam-channel`. Rejected because it can
allocate and has no realtime guarantee.

**D4 — Firewheel for the engine, fundsp for the DSP (decided by spike,
2026-10-31).** Firewheel owns the graph, device I/O, scheduling, declicking and the
audio clock. Each synth module is a Firewheel node whose processor runs fundsp
units. The spike (`spikes/`, results below) measured all three options on one
harness, offline:

| | fundsp `Net` | Firewheel + fundsp DSP |
| --- | --- | --- |
| Tone / clock accuracy | exact (440.000 Hz; 20 ticks, 0-sample error) | exact |
| Edit latency | next block (sample 2560 of 2560) | next block |
| Clicks on add/remove, by default | **yes** (12× / 8× step ratio); clean only with explicit `fade_in`/`crossfade` | clean (automatic 10 ms declick) |
| Audio-thread allocations | 0 | 0 |
| Render, 201 modules (budget 5.33 ms) | 0.12–0.26 ms | 0.14–0.24 ms |
| Game-side add + commit at 201 modules | 0.04–0.19 ms (clones the whole graph) | 0.02–0.07 ms |

Render cost is a wash, so the decision rests on what each saves us. Firewheel
already provides what we would otherwise build: declick on graph edits,
device-loss tolerance (`CpalStream`), sample-accurate scheduled events, a musical
transport, and `audio_clock_instant()` for D6. It is also slated to become Bevy's
default audio engine. fundsp provides the module library Firewheel lacks:
oscillators, Moog/SVF/biquad filters, ADSR, noise, delays, reverb. The spike's
`FunSineNode` showed a fundsp unit hosted inside a Firewheel node with zero
audio-thread allocations and no extra cost.

*Rejected:* fundsp `Net` alone (we would rebuild declick, scheduling and device
handling ourselves, and `commit()` cost grows with patch size); `knyst`
(self-declared API churn); `kira` (playback rather than synthesis).

**Firewheel 0.14 bug found by the spike.** In `firewheel-graph`
`processor/process.rs` (~line 217), a silent graph-output channel triggers
`output.fill_frames_with(..)`, which zeros **every** channel in the block, erasing
channels already written. Symptom: left channel silent whenever right is silent.
Workaround: a keep-alive node that writes zeros to every output and returns
`OutputsModified` (see `spikes/spike_firewheel`). Report upstream; drop the
workaround when fixed.

**D5 — Module IDs are game-assigned.** The game allocates `ModuleId(u64)` when it
sends `AddModule`, so it never waits on a round trip to learn an ID. The audio side
keeps a fixed-capacity map, preallocated, from ID to node index.

**D6 — Time sync.** Each block, the audio thread publishes
`SyncPoint { sample_pos, instant }` via `triple_buffer`. The game maps
`event.sample_time → Instant` with a smoothed rate estimate, and schedules visual
peaks at the predicted display time.

## Risks / Trade-offs

- [fundsp `Net` commits rebuild more than needed for large patches] → measure commit
  cost at 200 modules during the spike; batch edits per frame.
- [Firewheel and fundsp both pre-1.0] → the boundary in D1 limits the blast radius;
  pin exact versions.
- [Firewheel's API shifts as it is upstreamed into Bevy] → only `stubby_audio`
  touches Firewheel types; game code sees our `Command`/`Event` protocol.
- [fundsp oscillators pick a hash-derived start phase] → set phase explicitly for
  deterministic offline tests.
- [PipeWire/ALSA buffer sizes vary per machine] → make buffer size configurable;
  default 256 frames at 48 kHz; log negotiated config at startup (off the audio thread).
- [Panics in DSP code kill audio] → `catch_unwind` is not realtime-safe. Instead,
  forbid `unwrap` in DSP code via clippy `unwrap_used` on `stubby_audio`, and cover
  modules with offline tests.

## Open Questions

- ~~fundsp `Net` vs `firewheel`~~: decided, see D4.
- Firewheel pulls in `bevy_platform` 0.19 (a small utility crate). Our game uses
  Bevy 0.20, so the build will carry two `bevy_platform` versions until Firewheel
  catches up. This costs compile time, not correctness.
- Not yet heard by a human: run `cargo run --release -p spike_firewheel -- live`
  and `-p spike_fundsp -- live` to listen to the add/retune/remove sequence on real
  hardware (from the `spikes/` directory).
- Should factory simulation ticks be driven by the audio clock (musical) or a fixed
  Bevy timestep (gameplay)? Leaning toward audio-clock-driven for machines that
  "fire" and fixed-timestep for physics/movement.
- Polyphony/voice model: needed once sequencers arrive; out of scope here.
