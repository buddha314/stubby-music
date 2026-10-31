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

**D4 — fundsp `Net` behind the boundary, pending the spike.** `Net`'s
frontend/backend split already gives realtime-safe commits with off-thread
deallocation. *Alternative:* `firewheel`, which is game-oriented with built-in
scheduling. Evaluated in the spike (tasks §2) before D4 is final. *Rejected:*
`knyst` (self-declared API churn), `kira` (playback rather than synthesis).

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
- [PipeWire/ALSA buffer sizes vary per machine] → make buffer size configurable;
  default 256 frames at 48 kHz; log negotiated config at startup (off the audio thread).
- [Panics in DSP code kill audio] → `catch_unwind` is not realtime-safe. Instead,
  forbid `unwrap` in DSP code via clippy `unwrap_used` on `stubby_audio`, and cover
  modules with offline tests.

## Open Questions

- fundsp `Net` vs `firewheel`: decided by the spike. Record the result here.
- Should factory simulation ticks be driven by the audio clock (musical) or a fixed
  Bevy timestep (gameplay)? Leaning toward audio-clock-driven for machines that
  "fire" and fixed-timestep for physics/movement.
- Polyphony/voice model: needed once sequencers arrive; out of scope here.
