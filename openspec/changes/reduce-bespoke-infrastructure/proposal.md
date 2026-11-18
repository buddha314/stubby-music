## Why

The workspace already uses established open-source foundations for its engine,
physics, audio graph, and DSP, but several areas still contain substantial custom
infrastructure. A focused review can identify where a maintained library can replace
that work without losing the game's deterministic behavior, realtime audio guarantees,
authoring safeguards, or Bevy-main compatibility.

Current review candidates include the schema-driven rules editor and server, spline
sampling and cable/stream mesh utilities, and procedural environment helpers. These
areas are not presumed replaceable: the work must compare real requirements and
maintenance costs before recommending adoption.

The spatial-audio design also needs a fresh FOSS review. `add-rack-and-spatial-audio`
currently plans custom occlusion inputs and a vendored HRTF processor. AudioNimbus,
a Rust wrapper for Valve's Steam Audio, advertises geometry-based occlusion,
reflections, reverb, HRTF, and air absorption. Its published Bevy integration currently
targets Bevy 0.19, while this workspace follows Bevy main; its simulation output is
separate from audio-buffer rendering. Evaluate whether its standalone API can fit
behind the existing `stubby_audio` boundary and Firewheel host before committing to
the custom spatial path.

## What Changes

- Add a documented FOSS replacement assessment for rules authoring, spline and mesh
  generation, procedural environment helpers, and spatial audio.
- Align in-flight code-heavy proposals with the assessment: do not commit to vendoring
  an HRTF node before the AudioNimbus/Steam Audio evaluation, and use maintained FOSS
  noise/PRNG implementations for terrain primitives where they preserve determinism.
- Evaluate AudioNimbus/Steam Audio as a candidate environmental-audio engine against
  the full useful capability set: binaural and custom HRTF, Ambisonics, distance,
  directivity and air absorption, occlusion and material transmission, reflections and
  listener-centric reverb, baked probes, pathing around geometry, dynamic geometry,
  acoustic material authoring, threading, realtime constraints, Firewheel integration,
  build/runtime distribution, platform support, and license obligations. Open-flat and
  cave fixtures are baseline checks rather than scope limits.
- Prototype and adopt a FOSS component only when it meets the existing requirements
  and has a lower ongoing maintenance cost; otherwise record the concrete mismatch and
  retain the narrowest necessary custom adapter.
- Replace custom editor, spline, mesh, or environment code only where an identified
  maintained project demonstrably covers the current behavior. Preserve deterministic
  world generation and rules validation/save guarantees.
- Update the spatial-audio design and dependency plan based on the evaluation before
  implementing occlusion, HRTF, or environmental effects.

## Capabilities

### New Capabilities
- `foss-infrastructure`: evidence-based evaluation and adoption of FOSS libraries for
  existing custom infrastructure, including compatibility and maintenance criteria.

### Modified Capabilities
- `spatial-emission`: select and integrate a FOSS environmental audio implementation
  where compatible, with a documented fallback when it cannot meet engine constraints.

## Impact

- Research and design: `docs/research/` and
  `openspec/changes/add-rack-and-spatial-audio/design.md`.
- Potential implementation touchpoints: `stubby_audio`, `bevy_stubby_audio`,
  `stubby_rules_server`, `stubby_recipes`, `bevy_stubby_rack`, `stubby_streams`, and
  `stubby_world`; actual code changes depend on the evaluation.
- Candidate library: AudioNimbus (`audionimbus`) over Valve Steam Audio, dual MIT /
  Apache-2.0 wrapper. The wrapper's native SDK distribution, build requirements,
  and runtime packaging must be separately checked before adoption.
- No library is adopted solely because it is open source or feature-rich. Existing
  FOSS dependencies remain preferred when they already solve the problem.
