## 1. Audio library assessment

**Current gate:** the native SDK question is resolved for Linux x86_64: Steam Audio
4.8.1 built from source with `--minimal`, with IPP disabled, and an isolated AudioNimbus
0.17.0 executable linked and initialized its default HRTF. Direct geometry occlusion
and direct-effect processing are also verified. This is only the first vertical slice
of a broader environmental-audio engine assessment. Reflections/reverb, transmission,
pathing, baking/probes, Ambisonics, custom HRTFs, integration, callback safety,
cross-platform artifacts, and performance remain unverified.

- [x] 1.1 Inspect AudioNimbus and Steam Audio licenses, SDK redistribution terms,
  platform support, build requirements, dependency health, and current upstream AI /
  contribution policy; record sources and date. Finding: wrapper crates are MIT OR
  Apache-2.0, while Steam Audio's third-party notice includes restrictive Intel IPP
  binary-only terms. Bevy 0.19 is the only listed adapter line; source links and
  implications are in design.md.
- [x] 1.1a Compare using AudioNimbus standalone with updating/forking its Bevy adapter
  for the workspace's Bevy-main revision; record fork scope, upstreaming options, and
  maintenance plan. Finding: the standalone API compiles without Bevy; evaluate it
  before forking. A fork also needs to fit the workspace git-source allow-list.
- [x] 1.1b Resolve whether Steam Audio can be built without Intel IPP. Finding:
  Steam Audio v4.8.1 `--minimal` explicitly disables IPP and its optional Embree,
  Radeon Rays, and TrueAudio Next paths; CMake selects PFFFT and built-in vector math.
  The Linux x86_64 build and native HRTF initialization succeeded; details and caveats
  are in design.md.
- [ ] 1.2 Build a feature matrix and isolated prototypes for AudioNimbus/Steam Audio's
  useful documented capabilities: HRTF and custom SOFA, directivity/distance/air
  absorption, occlusion and material transmission, reflection algorithms and
  listener-centric reverb, Ambisonics, baked reflection probes, and pathing. Use open
  mudflat, enclosed cave, occluding wall, and around-corner fixtures; the first two are
  baseline fixtures, not the full scope. Record capability, quality, CPU/memory,
  supported backend, dependencies/license, and project-specific gaps per feature.
  Candidate alternatives and the current integration decision are tracked in
  [spatial-audio-alternatives.md](../../../docs/research/spatial-audio-alternatives.md);
  OMG Audio is the leading candidate for a measured spike, not an adopted replacement.
- [ ] 1.3 Define and prototype scene integration: convert Bevy/Avian geometry and
  acoustic materials; handle static, moving, and streamed geometry; establish scene
  lifecycle, probe placement/bake artifacts, invalidation, and designer workflows.
- [ ] 1.4 Prototype bounded parameter/result transfer into Firewheel-hosted processors
  and buses, including direct path, HRTF, reflections/reverb, and Ambisonic mixing where
  applicable. Measure latency, CPU/memory, allocations, locks, and stale-result behavior
  for moving emitters/listeners and dynamic occluders.
- [ ] 1.5 Establish repeatable IPP-free native builds and licensing/notice audits for
  Linux, Windows, and macOS; then demonstrate offline CI and packaged runtime loading.
- [ ] 1.6 Update `add-rack-and-spatial-audio/design.md` and tasks to select the broadest
  useful AudioNimbus integration the measured gates support, and identify only concrete
  unsupported or unsuitable features for narrow game-owned adapters.

## 2. Rules authoring replacement assessment

- [x] 2.1 Compare current JSON Editor and RulesDocument responsibilities with maintained
  schema-driven FOSS authoring/editing frameworks. Existing `add-crafting-designer`
  research already compared schema editors and alternatives; JSON Editor remains the
  focused FOSS form renderer.
- [x] 2.2 Verify undo, ID reference updates, validation, disk conflict detection,
  canonical serialization, migration recording, and offline operation in the candidate.
  These guarantees live in `RulesDocument`; the schema renderer does not replace them.
- [x] 2.3 Record adoption or retain decision; preserve the rules domain model and
  invariants unless a demonstrated equivalent exists. Decision: retain both current
  parts; no replacement with lower maintenance cost was identified.

## 3. Spline, mesh, and environment assessment

- [x] 3.1 Survey Bevy-main-compatible curve and procedural mesh libraries against
  stream path arc-length, mesh bending, normals/tangents, and cable tube requirements.
  Bevy curves cover Bezier evaluation; `bevy_gradmesh` 0.3.9 is a candidate for cable
  tubes, but its compatibility with this Bevy-main revision and mesh output is unproved.
- [x] 3.2 Survey focused FOSS options for deterministic seeded noise and spatial-field
  sampling against current sky, ridge, and foliage-air behavior. Candidate: `noise`
  0.9 provides seeded fBm and Worley, but tileable cloud output and generator stability
  still need a focused fixture before replacing existing functions.
- [ ] 3.3 Prototype only candidates that cover current requirements; record measured
  maintenance/code reduction and any output compatibility changes.

## 4. Adopt verified replacements

- [ ] 4.1 Adopt the candidates that pass the decision rule, adding exact dependency
  versions and license/source entries as required by `deny.toml`.
- [ ] 4.2 Keep game-specific behavior in narrow adapters and retain custom code only for
  documented uncovered requirements.
- [ ] 4.3 Update research, dependency architecture notes, OpenSpec requirements, and
  workspace dependency ownership to match the implemented outcome.
