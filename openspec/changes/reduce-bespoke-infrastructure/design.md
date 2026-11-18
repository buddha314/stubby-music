## Context

The prior code review found meaningful custom implementation, but custom line count
alone is not evidence of waste. The rules model encodes game-specific IDs, migrations,
validation, undo, and safe writes. Stream meshes encode an authored construction
language. Air and sky helpers express the game's own visual model. Any replacement
must preserve those contracts and lower total maintenance burden.

Spatial audio has an existing in-flight design in
`openspec/changes/add-rack-and-spatial-audio/`. That proposal budgets HRTF voices,
uses game-provided occlusion factors, and currently proposes a vendored HRTF node.
This change evaluates whether environmental simulation should supply some or all of
those results, rather than starting a competing spatial design.

## Candidate: AudioNimbus / Steam Audio

Upstream documentation describes geometry-based sound propagation, directivity,
distance attenuation, air absorption, occlusion and material transmission, reflections
and reverb, HRTF and custom SOFA HRTFs, Ambisonics, pathing around geometry, and baked
probe data for reflections and pathing. It also advertises GPU acceleration paths, but
the minimal FOSS build disables those optional backends; CPU simulation is the baseline
to assess. The crate docs currently list Bevy 0.19 integration, and state that the
Bevy adapter stops at simulation output; audio-buffer processing remains the caller's
responsibility. Therefore the evaluation must target the standalone API and an adapter
to `stubby_audio`, not assume that its Bevy feature or playback path fits this game.

The target is the broad useful Steam Audio capability set, not only a one-off occlusion
or cave-reverb demonstration. The mudflat and cave are baseline fixtures for validating
the pipeline. The assessment must cover direct and indirect propagation, authorable
geometry/material mapping, static and moving emitters/listeners, dynamic occluders,
listener-centric environmental beds, HRTF spatialization, Ambisonic mixing, and a
practical bake/probe workflow. It should distinguish features that can ship in a
minimal CPU-only FOSS build from optional acceleration or platform-specific features.

Check these constraints before adopting:

1. License and redistribution terms for both AudioNimbus and the Steam Audio SDK
   binaries/data; assess generated downloads and all release platforms.
2. Build reproducibility and offline/CI builds. The documented auto-install path
   downloads a large SDK and needs curl/wget and Clang; compare manual vendoring or
   system SDK packaging without silently adding a network build step.
3. Thread ownership and realtime safety: simulation must not allocate, lock, or run
   expensive geometry work in Firewheel's callback. Establish a bounded handoff from
   game/worker threads to the audio processor.
4. Geometry lifetime and updates for moving emitters/occluders, authored material
   properties, and the game's terrain and prefab collision representation.
5. Whether the simulation can provide stable occlusion, reflections/reverb, and
   HRTF parameters at the cadence required by the existing spatial-emission scenarios.
6. Whether Firewheel can host the resulting DSP without forking or replacing its
   device, clock, and graph responsibilities.
7. CPU cost and memory for expected emitter/geometry counts, measured on the project's
   reference machine and with offline deterministic fixtures where possible.

### Fork and AI-policy notes

AudioNimbus is dual-licensed MIT/Apache-2.0. A fork may update its Bevy integration
for the workspace's Bevy-main revision, and may adapt its standalone simulation API
to Firewheel. Keep upstream notices and license files, document modifications, and
track upstream commits so future changes can be compared and selectively merged.
Prefer a small compatibility fork or upstream contribution over maintaining a broad
divergent copy.

The public repository README and license information were reviewed on 2026-11-17.
They state the dual license and describe the project, but the reviewed material does
not state an AI-assisted contribution policy. No repository-level AI policy was
located in the accessible upstream pages during this review. This is not evidence
that AI use is prohibited or permitted under a separate policy: check the current
repository files and any applicable maintainer contribution instructions before
submitting an AI-assisted upstream PR. The MIT/Apache license grants software-use
rights; it does not itself answer a project's contributor policy or establish the
provenance of every contribution.

### Compatibility and FOSS assessment (2026-11-17)

AudioNimbus 0.17.0 is the crates.io wrapper release inspected for this assessment.
Its wrapper and `audionimbus-sys` crates declare `MIT OR Apache-2.0`; the standalone
API compiles with `default-features = false, features = ["dynamic"]`, without its Bevy
integration. Its sys crate is versioned 4.8.2-rc.5 but deliberately generates bindings
for Steam Audio 4.8.1; its build script emits `-lphonon` and accepts
`STEAMAUDIO_LIB_DIR`. This is an unusual version pairing, but it is explicitly pinned
in the crate source and matches the current 4.8.1 SDK release.

The upstream Bevy adapter remains on Bevy 0.19. The standalone API is therefore the
lower-cost path to evaluate first; a fork should be considered only if the standalone
API proves insufficient and a Bevy ECS adapter is still useful. A git fork under this
workspace must also fit `deny.toml`'s restricted git-source policy (Bevy upstream and
the `buddha314` organization); a crates.io dependency passes the existing source
policy.

The inspected `audionimbus-sys` build script supports Rust target triples for Windows
x86/x64 MSVC, Linux x86/x64 GNU, macOS x86_64/aarch64, Android ARMv7/ARMv8/x86/x64,
and iOS ARM64 for binding generation. Its `auto-install` maps corresponding Steam Audio
release artifacts, including macOS universal and iOS static archives. That is evidence
of intended target support, not proof that this project's toolchains can build and ship
them. The wrapper's manual **static linking** is narrower: x86/x64 Linux GNU, x86/x64
Windows MSVC, and x86_64/aarch64 macOS. Android and iOS need dynamic-linking behavior
or a project-specific native packaging arrangement; iOS packaging in particular needs
validation. WebAssembly Emscripten appears in the binding target match but not in the
auto-install target map, and should be considered unsupported until separately proven.
This project presently targets desktop Linux, so Linux is the immediate release gate;
Windows and macOS should be built before claiming desktop portability.

The native integration route is unambiguous: without auto-install, `audionimbus-sys`
uses checked-in Steam Audio headers to generate bindings, emits `-lphonon`, and adds
the directory from `STEAMAUDIO_LIB_DIR` to the native linker search path. The SDK
library must therefore be provisioned before Cargo links the final binary and made
available at runtime (or linked statically). `cargo check` alone cannot establish this.
`auto-install` downloads the Valve prebuilt release archive from GitHub during Cargo's
build, so it is unsuitable for offline/reproducible builds and does not let this project
select an IPP-free artifact. The source-built `--minimal` library is the cleaner FOSS
route. Its Linux output was audited for dynamic dependencies and IPP symbols; repeat
the audit for each shipped artifact and include only notices for components actually
present.

In the wrapper's static-link branch, the Rust build script links `phonon`, `pffft`,
`mysofa`, zlib and the platform C++/system runtimes as archives. This is useful for
reducing runtime loader concerns, but increases executable size and requires producing
the matching archives per target. Dynamic linking is the simpler first packaging
choice if the game installer reliably ships `libphonon` beside the executable and the
loader can find it. Neither packaging model has been exercised outside Linux x86_64.

Steam Audio's source repository declares Apache-2.0. Its third-party notice includes
Intel IPP terms that are binary-only and do not allow modification, but the SDK source
build makes IPP optional. In a clean checkout of tag `v4.8.1`, `core/build/build.py
--minimal` explicitly sets `STEAMAUDIO_ENABLE_IPP=FALSE`, disables Embree, Radeon Rays,
and TrueAudio Next, and the CMake fallback uses built-in vector math plus PFFFT. The
build dependency manifest pins PFFFT, libmysofa, zlib, and FlatBuffers; all are
permissively licensed. IPP therefore need not be part of our build or distribution.
The SDK package's generated `THIRDPARTY.md` includes the IPP notice unconditionally,
so a product notice should list only components actually included and preserve the
applicable Apache-2.0, PFFFT, libmysofa, zlib, and FlatBuffers notices. Re-audit the
exact built artifact and notices per target before release.

#### Native build and link probe

An isolated Linux x86_64 probe was completed in `/tmp` on 2026-11-17:

- Cloned Steam Audio tag `v4.8.1` and built `libphonon.so` with the upstream
  `--minimal --operation ci_build` configuration. IPP was not installed; configuration
  found PFFFT, zlib, libmysofa, and FlatBuffers. The resulting binary dynamically
  depends only on the platform C/C++ runtime libraries; no IPP dynamic dependency or
  IPP symbol was present.
- The environment's CMake 4 and GCC 16 exposed compatibility issues in old pinned
  dependency build scripts (CMake minimum policy and a FlatBuffers `-Werror` warning).
  The probe completed by building PFFFT/FlatBuffers with compatibility flags. A
  supported/pinned CMake and compiler toolchain should be used in CI rather than
  carrying these local workarounds into the game.
- Linked the probe with `STEAMAUDIO_LIB_DIR` and ran it with `LD_LIBRARY_PATH`. It
  printed `Steam Audio 264193: Context and default HRTF initialized`; `264193` is the
  packed 4.8.1 API version. A preallocated mono block also processed through the
  `DirectEffect` wrapper with finite output. A runtime-selected default ray-tracing
  scene returned direct occlusion `1.0` for a clear source-listener path and `0.0` after
  adding a concrete wall between them. This verifies geometry updates, direct
  simulation, result retrieval, and effect processing together on Linux x86_64. It is
  not a Firewheel integration, callback-allocation proof, or audio-quality review.
- A Rust `cargo check` without the SDK succeeds because it does not link native symbols;
  a real `cargo build` fails with `unable to find library -lphonon` until the SDK is
  supplied. The automatic installer downloads Valve's prebuilt SDK and is not our
  FOSS-clean route. CI and packaging must build or fetch a verified IPP-free artifact,
  then set the build-time and runtime library paths.

Conclusion: **AudioNimbus is viable for a Linux prototype under the project's
permissive FOSS policy, with high confidence on license feasibility and direct
spatial-simulation usability.** This establishes a candidate engine, not that the broad
feature set is already integrated or that the cave is the maximum scope. Approval for
game integration or release still depends on feature-by-feature validation, the
Firewheel graph architecture, real-time constraints, performance, Windows/macOS builds,
and clean repeatable CI packaging. Reflection, pathing, transmission, ambisonics,
listener-centric reverb, geometry/material authoring, and baked-data workflows remain
unproven in this project even where the upstream API documents support.

AudioNimbus's `wiring` runner performs simulation on a worker and publishes snapshots,
which suits a non-callback simulation boundary in principle. Its direct and reflection
effect wrappers accept caller-owned audio-buffer slices and call the native processing
API, so they can be hosted by a Firewheel processor without wrapper-owned sample-buffer
allocation. This does not prove that every native effect call is allocation-free or
lock-free; measure the exact effect calls under the workspace's `assert_no_alloc`
callback guard. Scene and simulator code internally uses synchronization, so create
and update geometry on the control/simulation side and publish bounded effect
parameters/results to the audio processor. Never run scene simulation in the callback.

Do not bind the main workspace to AudioNimbus until a minimal Firewheel integration,
callback-safety check, target-platform matrix, and reproducible CI artifact flow are
demonstrated. Evaluate the full useful feature set against the game's environmental
audio goals. Do not reject documented capabilities merely because a narrow authored
reverb preset can approximate one fixture. If a feature or platform fails a gate, record
the measured incompatibility and retain or implement only the narrow adapter needed to
cover that gap.

### AudioNimbus capability inventory and evidence status

Upstream Steam Audio documentation describes these capability groups; status here is
project verification as of 2026-11-17, not a claim that the engine lacks them:

| Capability | Project status | Required follow-up |
| --- | --- | --- |
| Direct HRTF/binaural rendering and built-in HRTF | HRTF initialization verified; actual binaural rendering is not yet verified in our probe | Render moving mono sources; inspect CPU and transitions; validate user/headphone spatial settings |
| Custom HRTF / SOFA and Ambisonics encode, rotate, decode | Not verified | Check asset/data licensing, format loading, graph layout, and performance |
| Distance attenuation, directivity, air absorption | DirectEffect processing verified; simulation outputs not independently checked | Exercise directivity and air absorption from source/listener/material scene inputs |
| Geometry occlusion | Verified for clear path vs. a concrete wall | Validate partial occlusion, multiple surfaces, dynamic geometry, and update latency |
| Material transmission | Not verified | Map game acoustic materials; test multi-surface transmission and artistic controls |
| Real-time reflections and parametric/convolution/hybrid reverb | Not verified in our probe | Compare algorithms, render impulse/room fixtures, test worker simulation and effect rendering |
| Baked reflection/reverb probes | Not verified | Establish deterministic bake inputs, artifact format/versioning, batch tooling and runtime loading |
| Pathing around corners/openings | Not verified | Prototype probe placement, bake pipeline, moving source/listener behavior and dynamic-occluder handling |
| Baked data management / scene lifecycle | Not verified | Define geometry extraction, acoustic materials, invalidation and asset/editor ownership |
| Optional GPU/accelerated ray tracing | Excluded from minimal FOSS baseline | Treat as optional only; audit each backend's license, availability, and fallback |

The sys crate's `auto-install` is a convenience for local development, not a deployment
strategy: it downloads and extracts a native SDK as part of Cargo compilation and
copies libraries into Cargo's output directory. The production path should pin the SDK
tag and dependency sources, build the selected minimal configuration in a controlled
artifact job, hash and audit the output, then package it explicitly with the game.
Never let an unpinned build-time download determine shipped native code.

The official guide describes direct effects and transmission, reflection simulation and
reverb, probe baking, and pathing as separate but composable systems. Treat each as a
candidate capability, then prioritize against authored game requirements rather than
collapsing the evaluation to the first working occlusion path.

## Other replacement candidates

- **Rules designer:** keep the current JSON Editor frontend and `RulesDocument` model
  for now. The frontend is already a vendored MIT-licensed schema form renderer; the
  editing model enforces domain guarantees that a generic form library does not
  replace. Replacing it has no demonstrated code or maintenance win.
- **Curves:** Bevy 0.20 already includes cubic Bezier curves, sampling, and curve
  reparameterization. Prefer that existing engine API for curve evaluation and
  tangents before adding a spline crate. Arc-length tables, game-specific cable sag,
  and arbitrary stream-mesh bending may still need small adapters. `bevy_gradmesh`
  0.3.9 (MIT OR Apache-2.0) advertises 3D polyline tubes and profile sweeps and lists
  Bevy 0.20 dependencies, making it a candidate for the rack's cable tube mesh. It is
  not yet shown to unify with this workspace's Bevy-main git revision or to preserve
  the current mesh attributes. It does not directly replace bending the authored
  stream segment mesh with its normals and tangents.
- **Noise:** `noise` 0.9 provides seeded fBm and Worley noise, matching the terrain
  design's two noise primitives. Prefer it over handwritten noise functions if its
  seeded output and seam behavior meet the world-generation contract. Tileable cloud
  noise may remain custom unless the crate's periodic options prove equivalent.
- **Randomness:** use the project's specified seeded PRNG (`rand_chacha`) rather than
  custom random-number code or `thread_rng`; derive independent streams from stable
  world/chunk keys.

## Decision rule

For each candidate, record repository, license, maintenance/activity, compatible
versions, covered behavior, gaps, integration cost, and a minimal prototype result.
Adopt only if it meets all required behavior, complies with the repo's license and
dependency rules, and reduces the code and maintenance we own. Otherwise mark it
"retain custom" with reasons. This change does not require replacing every candidate.

## Risks

- A native spatial-audio SDK may complicate distribution, supported targets, CI, and
  release packaging even when its Rust wrapper is permissively licensed.
- A Bevy integration pinned behind Bevy main may not compile; the standalone API may
  still be usable but will require an adapter.
- Moving simulation to a worker can add latency or stale geometry state; audio-thread
  work must remain bounded and allocation-free.
- Replacing deterministic algorithms can alter saved world layouts; preserve generator
  revision semantics if any generated output changes.
