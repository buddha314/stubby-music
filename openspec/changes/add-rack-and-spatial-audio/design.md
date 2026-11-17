## Context

`add-audio-engine` built a working engine core (`crates/stubby_audio`): a
game-facing `Engine` with `send`/`flush`/`drain_events`, Firewheel as the device and
graph layer, fundsp DSP, offline rendering, and 18 tests. Its modules are Firewheel
nodes, so the patch is a DAG and feedback is rejected. Research
(`docs/research/rack-and-spatial-audio.md`) showed two gaps. Modular play needs VCV
semantics (per-sample, 1-sample cables, feedback). And finding things by ear needs
HRTF, distance cues and occlusion, which neither Bevy 0.20 (two-ear distance gain
only) nor Firewheel (basic panning only) provide.

## Goals / Non-Goals

**Goals:**
- A rack with VCV semantics and VCV voltages, inside the existing `Engine` API.
- Spatial emission good enough to find a module by ear, within a fixed CPU budget.
- VCV-style control surface: smoothed parameters, lights, meters, scopes, patch data.

**Non-Goals:**
- The 3D module-panel UI itself (Bevy side). This change provides what it reads and
  writes.
- Polyphonic modules (the cable type supports them; no module uses them yet).
- Multi-threaded rack processing (not needed at the measured budget; see Risks).
- Physically based reverb and transmission (Steam Audio) — the upgrade path, not v1.

## Decisions

**D1 — Our own rack engine, per sample, double-buffered.** Each sample, every module
reads its inputs from the previous sample's output slots and writes the current ones;
the two slot arrays alternate. That gives the 1-sample cable delay, makes order
irrelevant, and makes cycles legal with no graph analysis. The module trait processes
one sample. fundsp units are used through their per-sample `tick()`.
*Alternatives:* (a) Firewheel nodes, the current core: it is a DAG and block-rate, so
there is no feedback. (b) Block processing with a 1-block delay only on cycles: it is
cheaper, but a 256-sample delay in a feedback loop is a comb filter, not FM.
(c) knyst or fundsp `Net`: block-based as well.

Spike (`spikes/spike_rack`, 16-core workstation, single thread, release, 256-frame
blocks at 48 kHz, every voice a feedback loop):

| Modules | Cables | Mean / block | p99 | % of 5.33 ms budget | ns per module-sample |
| --- | --- | --- | --- | --- | --- |
| 48 | 54 | 105 µs | 117 µs | 2.0% | 8.5 |
| 100 | 114 | 225 µs | 246 µs | 4.2% | 8.8 |
| 200 | 215 | 452 µs | 500 µs | 8.5% | 8.8 |
| 400 | 415 | 909 µs | 994 µs | 17.0% | 8.9 |

The cost is linear, and the feedback patches were finite, bounded and bit-identical
across runs, with zero allocations.

**D2 — The rack is one Firewheel node.** `RackNode` runs the rack in the same audio
callback, so rack time and the audio clock are the same clock. Clock events keep their
exact `sample_time`, there is no inter-thread buffer, and there is no drift. Its
outputs are emission stems: one mono channel per emitter that is currently voiced. The
`Engine` API stays. `send` validates against the rack's patch, and `flush` publishes
edits. Rack edits travel to the audio thread as prebuilt module boxes, through a
lock-free queue owned by `RackNode`, the same pattern as Firewheel's own. Removed
modules return through it so they are dropped on the game thread.

**D3 — VCV voltage standards, in `stubby_audio::signal`.** Audio ±5 V, gates 10 V,
triggers 10 V × 1 ms, a Schmitt trigger at 0.1/1.0 V, 1 V/oct from C4 (oscillators)
and 2 Hz (clocks and LFOs), no clamping, NaN → 0, and ±10 V = full scale at
emission. Converting now means updating ~10 constants and test values. Converting
after the module library grows would mean revisiting every module.

**D4 — Three spatial tiers with a voice budget.** Each frame the game ranks emitters
by estimated audibility (gain after distance, hearing radius and occlusion). The top
`hrtf_voices` get HRTF, the next `pan_voices` get `SpatialBasicNode`, and the rest are
virtual: the rack still computes them, but no voice is assigned. A pool of HRTF and
pan nodes is preallocated; voice assignment is a routing change plus a crossfade, not
node creation. Hysteresis (an emitter must beat the incumbent by a margin, and stay
for at least a second) prevents flapping.

Spike, moving emitters, single thread:

| Emitters | HRTF mean / p99 / max | Basic pan mean |
| --- | --- | --- |
| 1 | 24 µs / 51 µs / 63 µs | 1.8 µs |
| 16 | 400 µs / 816 µs / 2.3 ms | 25 µs |
| 32 | 789 µs / 1.57 ms / 3.5 ms | 50 µs |
| 64 | 1.61 ms / 3.46 ms / **5.56 ms** | 103 µs |

HRTF costs about 25 µs per emitter per block, roughly 16× panning. At 64 the worst
block exceeds the whole budget, so the upstream "avoid more than 32–64" is too high
for us. **Default: 16 HRTF + 32 panned** ≈ 8.5% mean. Together with a 200-module rack,
that is about 17% of the budget, which leaves room for SFX and for the slower laptop
(`breq`). Both numbers are settings.

**D5 — HRTF FFT size matched to the audio block.** The node's default overlap-save
block is 4 × 128 = 512 frames, against our 256-frame audio block, so every other block
does all the FFT work. A 2 × 128 FFT has the same mean but half the p99:

| Emitters | FFT 4×128 mean / p99 | FFT 2×128 mean / p99 |
| --- | --- | --- |
| 16 | 420 µs / 835 µs | 405 µs / 450 µs |
| 32 | 804 µs / 1.61 ms | 813 µs / 904 µs |

Use 2 × 128 at a 256-frame block (generally, an FFT size of twice the block length,
split into 128-frame slices). **Never use `slice_count = 1`:** it panics inside
`rustfft` (insufficient scratch buffer in `hrtf` 0.8.1).

**D6 — Use AudioNimbus over Steam Audio as the spatial/environmental audio engine.**
The user selected AudioNimbus as the default path, continuing until a concrete blocker
is found. Use the Bevy-free AudioNimbus API behind `stubby_audio`; do not fork its Bevy
adapter unless an identified ECS feature is needed. Steam Audio owns direct propagation,
HRTF, material transmission, reflections/reverb, Ambisonics, and supported probe/pathing
features where they meet the game's needs. Keep Firewheel as device/graph/clock host and
bridge simulation outputs to its audio processors without blocking the callback. Build
and package the SDK from a pinned source revision with optional closed components
disabled; never make Cargo fetch an opaque SDK binary during a normal build.

The Linux x86_64 native link, default HRTF initialization, direct effect, and geometry
occlusion have been verified in an isolated AudioNimbus 0.17.0 / Steam Audio 4.8.1
probe. The AudioNimbus core demo documents a Firewheel-backed audio graph and async
direct/reflection/reverb output consumption. This is a direction decision, not proof of
all effects or target platforms. Replace D6's former vendored IRCAM HRTF node only after
AudioNimbus's binaural effect is hosted in Firewheel and the SDK artifact path is
reproducible. Do not retain parallel custom HRTF DSP without a measured gap.

The initial Firewheel node adapter buffers arbitrary callback lengths into fixed-size
AudioNimbus blocks. It adds one configured block of latency (about 5.33 ms at 256
frames/48 kHz); include this in source localization and interaction listening checks.
The Firewheel node compiles and a standalone processor renders finite directional
stereo on Linux, but live `Engine` routing, callback allocation guards, runtime device
rate changes, and CPU cost are still open.

**D7 — Let AudioNimbus/Steam Audio calculate environmental propagation.** Source and
listener transforms plus acoustic scene geometry/materials are supplied outside the
audio callback. Direct outputs (including distance, air absorption, directivity,
occlusion and transmission when configured) and indirect outputs (reflections/reverb,
and later pathing where useful) are transferred into Firewheel processors. Game hearing
radius and emitter budgeting remain game policy. Do not also apply a duplicate custom
occlusion ray and hand-tuned low-pass when the native simulation supplies the same
result; retain narrow adapters only for unsupported policy or authoring needs. Keep
speaker panning fallback and smoothing where needed for stable controls.

**D8 — Control data flows back without locks.** Lights and meters are `f32` atomics
(stored as bits), smoothed in the rack and read by the game at frame rate. Scopes are
one `rtrb` ring per probed port. Parameters are smoothed in the rack with a one-pole
ramp (default 10 ms).

**D9 — Cable fades follow signal type.** A cable is faded if it carries audio
towards an emitter. Each port is tagged by type (audio, CV, gate), so the rack knows.
Gate and CV cables switch instantly. This keeps the D7 rule from `add-audio-engine`
and extends it to cables, which closes that change's task 3.9.

## Relationship to add-audio-engine

That change stays the record of the engine core, the spike decision (D4) and its
correction. Before archiving either change:
- `synth-modules` "Eurorack signal conventions" (±1.0 audio, gate above 0.5) is replaced
  by `modular-rack` "VCV voltage standards".
- `audio-engine` cycle rejection (`EngineError::Cycle`) is replaced by `modular-rack`
  "Feedback patches are valid".
- `synth-modules` "Module and port model" (single cable per input, fan-out) still
  holds; it moves from Firewheel edges to rack cables.
Archive `add-audio-engine` first, then turn these notes into MODIFIED deltas here.

## Risks / Trade-offs

- [The rack is single-threaded; ~8.8 ns per module-sample means about 1,100 modules
  fill a whole block] → budget is 200 modules for v1; VCV-style worker threads
  splitting modules, with a barrier per sample, are the known path if patches grow.
- [Tail latency: the spike saw occasional 2–3 ms max blocks on a loaded machine,
  probably OS preemption] → measure on `breq` under a running game; raise the
  device buffer if needed; consider realtime thread priority.
- [HRTF needs headphones; front/back confusion persists without head movement] →
  camera-driven listener (spec), speaker fallback, listener-profile setting.
- [glam split: Bevy 0.20 uses glam 0.33, Firewheel 0.14 and the HRTF node use 0.32
  (Firewheel #144)] → only `stubby_audio` touches glam 0.32 types; the Bevy side
  passes plain `[f32; 3]` positions.
- [Native Steam Audio SDK complicates FOSS packaging] → pin the source tag and
  permissive dependency revisions, build the minimal IPP-free SDK in controlled CI,
  audit each platform artifact and notice set, and test runtime loader behavior.
- [Simulation results can be stale while geometry or emitters move] → publish snapshots
  asynchronously with timestamps; never block or wait in the Firewheel callback.
- [AudioNimbus API or native SDK update breaks the pinned version pairing] → pin
  AudioNimbus and Steam Audio together; update only with an isolated native-link and
  effect-render probe.

## Migration Plan

1. Land the rack beside the existing modules, behind the same `Engine` API.
2. Port Sine, Square, Clock and Constant to the rack trait and VCV voltages, and
   update the 18 engine tests to volts.
3. Remove the Firewheel-node modules and `EngineError::Cycle`.
4. Add the spatial layer; the master-output bus becomes "emitter at the listener"
   (non-spatial) for UI and debug use.

Rollback: the Firewheel-node core is in git history on `audio-engine-core`.

## Open Questions

- **Resolved host decision (0.1): retain Firewheel as the device, graph, and clock host.**
  AudioNimbus supplies spatial/environmental simulation and effects within the existing
  engine boundary. Its asynchronous simulation output is handed to Firewheel processors;
  no simulation, waits, or scene mutation may run on the callback. Revisit a native
  `cpal` host only if a measured Firewheel limitation blocks this architecture.

- Which emitter does a module expose by default (its main output, or an explicit
  "speaker" module)? Leaning towards: VCOs, VCAs and mixers emit by default; utilities
  do not.
- Should the game rank emitters by estimated loudness (cheap, game-side) or by the
  rack's measured stem RMS (accurate, one frame late)? Leaning towards measured RMS
  from the meters in D8.
- Propagation delay (sound at ~340 m/s): realism versus confusion at gameplay
  distances. Prototype it and play-test.
