## Why

The modular synth is the game, not background audio, and it has to behave like VCV
Rack: players self-patch, and feedback loops are everyday Eurorack. The engine core
from `add-audio-engine` built modules as Firewheel nodes in a DAG, which forbids
feedback (`EngineError::Cycle`). Modules also have to sound *from the world*, so
players can find them by listening. That needs HRTF, distance cues, occlusion, and a
voice budget, none of which Bevy 0.20 or Firewheel provide out of the box. Research:
`docs/research/rack-and-spatial-audio.md`. Spike: `spikes/spike_rack`.

## What Changes

- **A rack engine with VCV semantics.** Every module runs once per sample, and every
  cable delays by one sample, so any patch, feedback included, is valid and
  deterministic. It is hosted as a single Firewheel node, so it shares the audio
  callback and clock.
- **BREAKING (vs the in-flight `add-audio-engine`):** module DSP moves from Firewheel
  nodes into the rack, and `EngineError::Cycle` goes away. The `Engine` API
  (`send`, `flush`, `drain_events`, `sample_to_instant`, offline rendering) stays.
- **BREAKING:** signals adopt VCV voltage standards (audio ±5 V, gates 10 V,
  Schmitt-trigger inputs, 2 Hz clock reference) in place of ±1.0 audio and gates
  above 0.5.
- **VCV-style control:** smoothed parameters; lights, meters and scopes read back
  without locks; patches serialized as data.
- **Spatial emission:** modules emit at their world position. HRTF goes to the most
  audible emitters, cheap panning to the next tier, and the rest go silent, with
  distance cues, a per-emitter hearing radius, raycast occlusion, and a speaker
  fallback.
- The HRTF node is vendored from `firewheel-ircam-hrtf` and ported to Firewheel 0.14,
  because the published crate targets Firewheel 0.12.

## Capabilities

### New Capabilities
- `modular-rack`: per-sample processing, 1-sample cables, feedback, voltage
  standards, polyphony-ready cables, determinism, hosting, and the performance budget.
- `spatial-emission`: emitters, the listener, HRTF/panning/virtual tiers with an
  audibility-ranked budget, distance cues, hearing radius, occlusion, and the speaker
  fallback.
- `rack-control`: parameter smoothing, lock-free lights/meters/scopes, cable-edit
  semantics, and patch serialization.

### Modified Capabilities
<!-- None in openspec/specs/ yet. This change supersedes parts of the unarchived
     `add-audio-engine` change: its `synth-modules` signal conventions, its rejection
     of cyclic patches, and modules as Firewheel nodes. Reconcile before archiving
     either; see design.md "Relationship to add-audio-engine". -->

## Impact

- `crates/stubby_audio`: new `rack` module (the engine and module trait), a
  `RackNode` Firewheel host, a `spatial` module (emitters, tiers, occlusion inputs),
  and rewritten `signal` constants. Existing modules are ported to the rack trait.
- Dependencies: `hrtf` 0.8.1 (Fyrox's HRTF DSP); Firewheel feature `glam-32`; a
  vendored HRTF node with its MIT/Apache notice.
- Bevy side (later): the listener transform and per-emitter occlusion rays are
  supplied by the game each frame.
- Risk to track: Bevy 0.20 uses glam 0.33 while Firewheel 0.14 and the HRTF node use
  glam 0.32 (Firewheel issue #144).
