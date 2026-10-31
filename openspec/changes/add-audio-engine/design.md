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

**D3 — Firewheel's channel carries commands; `rtrb` carries events back.**
*(Revised when building the core.)* `Engine::send` validates a `Command` and applies
it to the `FirewheelContext` on the game thread. `Engine::flush`, called once per
frame, publishes the frame's edits over Firewheel's own lock-free context→processor
channel, so a second command ring would only duplicate it. Removed nodes are freed
by Firewheel off the audio thread, so no garbage ring is needed either. Events come
back on one `rtrb` ring per Clock. Firewheel may rebuild a processor (for example
after a device change), so each construction makes a fresh ring and posts the
consumer to a game-side mailbox; that mutex is only touched on the game thread.
`SetParam`s coalesce per frame (last value wins), which bounds per-flush work by the
number of distinct parameters rather than the number of sends.

**D4 — Firewheel for the engine, fundsp for the DSP (decided by spike,
2026-10-31).** Firewheel owns the graph, device I/O, scheduling and the audio clock. Each synth module is a Firewheel node whose processor runs fundsp
units. The spike (`spikes/`, results below) measured all three options on one
harness, offline:

| | fundsp `Net` | Firewheel + fundsp DSP |
| --- | --- | --- |
| Tone / clock accuracy | exact (440.000 Hz; 20 ticks, 0-sample error) | exact |
| Edit latency | next block (sample 2560 of 2560) | next block |
| Clicks on add/remove, by default | **yes** (12× / 8× step ratio); clean only with explicit `fade_in`/`crossfade` | **yes**, see correction below |
| Audio-thread allocations | 0 | 0 |
| Render, 201 modules (budget 5.33 ms) | 0.12–0.26 ms | 0.14–0.24 ms |
| Game-side add + commit at 201 modules | 0.04–0.19 ms (clones the whole graph) | 0.02–0.07 ms |

Render cost is a wash, so the decision rests on what each saves us. Firewheel
already provides what we would otherwise build: device-loss tolerance (`CpalStream`), sample-accurate scheduled events, a musical
transport, and `audio_clock_instant()` for D6. It is also slated to become Bevy's
default audio engine. fundsp provides the module library Firewheel lacks:
oscillators, Moog/SVF/biquad filters, ADSR, noise, delays, reverb. The spike's
`FunSineNode` showed a fundsp unit hosted inside a Firewheel node with zero
audio-thread allocations and no extra cost.

*Rejected:* fundsp `Net` alone (we would rebuild scheduling and device handling
ourselves, and `commit()` cost grows with patch size); `knyst`
(self-declared API churn); `kira` (playback rather than synthesis).

**Correction (2026-10-31, found while building the engine core).** The spike's
"clean" Firewheel add/remove results were coincidences, not declicking. Firewheel
does **not** fade nodes on add or remove; it only declicks *bypass* toggles. The
spike's sine started at phase 0, so adding it could not step. And removal happened at
frame 38,400, where the 330 Hz sine had completed exactly 264 cycles and sat at zero.
Removing at an arbitrary frame steps 5.8× the steady bound. Sound-producing modules
therefore need their own fade envelope; bypass can't do it, because it routes a node's
input (e.g. the Sine VCO's pitch CV) to its output. The decision in D4 still stands on
the other reasons above.

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

**D6 — Time sync.** *(Revised when building the core.)* Firewheel already
publishes its audio clock with a delay-corrected reading
(`audio_clock_corrected`), so `Engine::sample_to_instant` maps an event's
`sample_time` to a predicted `Instant` from that clock. No `triple_buffer` needed.

**D7 — Sound sources fade; gates and CV switch instantly.** Firewheel does not
declick node add/remove (see the D4 correction). Sound-producing modules (today the
Sine VCO) fade in over Firewheel's declick time when added. On removal they fade out
and the node is deleted on a later `flush`, 50 ms later. Clock and Constant switch
instantly, because fading a gate moves its edge and a CV fade is a pitch glide.
Cable connect/disconnect is **not** declicked yet: Firewheel has no per-edge gain.

**D8 — The engine is single-threaded.** `FirewheelContext` is not `Send` (each node
holds an untyped `Box<dyn Any>`), so neither is `Engine`. It must live on one thread
for its whole life.

**D9 — The engine runs on its own control thread; the game holds a `Send` handle
(task 5.0, decided 2026-10-31).** `AudioThread` (in `stubby_audio`) creates the
`Engine` on a dedicated thread and returns a `Send + Sync` handle. The game sends
commands during the frame and calls `commit()` once at the end of it. The thread
applies that frame's commands as one batch, flushes, drains events, and publishes an
audio-clock snapshot for `sample_to_instant`. Validation errors come back as
`Notice::Rejected` rather than return values. In Bevy the handle is a plain
`Resource` (`Synth`), with `ClockTick` messages in `PreUpdate` and the commit in
`Last`. *Alternative:* a `NonSend` resource. That would pin audio systems to the main
thread, tie the engine to a Bevy API Bevy has talked of removing, and let a slow frame
delay the engine's housekeeping. The handle shape also survives replacing Firewheel.
`AudioThread::start_offline` renders in real time without a device, for tests and
headless runs.

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

- **Superseded in part by `add-rack-and-spatial-audio`:** modules move from Firewheel
  nodes into a VCV-style rack (feedback allowed, VCV voltages). See that change's
  design.md, "Relationship to add-audio-engine".

- ~~How Bevy holds a non-`Send` engine~~: decided, see D9.
- Declicking cable connect/disconnect: a hidden gain node per cable, or accept
  Eurorack-faithful cable clicks.
- The live device callback belongs to `firewheel-cpal`, so `assert_no_alloc` cannot
  wrap it. Allocation-freedom is enforced by the offline tests, which run the same
  processor code. A Firewheel hook, or our own cpal backend, would close the gap.

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
