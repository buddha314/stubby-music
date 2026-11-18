# Research: open spatial and environmental audio alternatives

*Researched 2026-11-18. Project capability and maturity statements below are based on
upstream documentation and are not independent performance or quality measurements.*

## Decision question

Can an existing FOSS library provide the environmental audio the game needs while
reducing project-owned code? The target is broader than a cave effect:

- binaural/HRTF localization and source directivity;
- distance cues, air absorption, geometric occlusion and material transmission;
- early reflections, late/listener-centric reverb, and ideally diffraction/pathing;
- static and moving game geometry, with a manageable authoring workflow;
- runtime use from our Bevy/Jackdaw world and Firewheel audio graph;
- realtime-safe handoff: no waits, locks, allocations, I/O, or expensive scene work on
  the device callback;
- an open-source license and a reproducible native build for supported platforms.

## Current baseline and the observed reverb issue

The current baseline is AudioNimbus 0.17.0 over Steam Audio 4.8.1, built from source
with the optional Intel IPP path disabled. Steam Audio documents HRTF, direct-path
occlusion/transmission, reflections, environmental effects, moving geometry and
Ambisonics. AudioNimbus provides the safe Rust-facing API. Both are permissively
licensed for our use: AudioNimbus is MIT OR Apache-2.0, and Steam Audio is Apache-2.0.
The optional IPP component has separate restrictive terms; keep it disabled and make
release notices reflect the actual build configuration.

The current game evidence says direct geometry occlusion works. It does not establish
that the engine lacks reverb. The integration has not yet routed the separate
listener-centric environmental output into the game mix, and the per-source
reflection path is subtle. The `BinauralSpatializer` wet contribution is also scaled
down. Therefore replacing the engine before wiring and measuring both outputs would
confound an integration defect with a library capability limit.

Sources: [AudioNimbus](https://github.com/MaxenceMaire/audionimbus), [Steam Audio
documentation](https://github.com/ValveSoftware/steam-audio/blob/master/core/doc/index.rst),
[Steam Audio guide](https://github.com/ValveSoftware/steam-audio/blob/master/core/doc/guide.rst),
[Steam Audio license](https://github.com/ValveSoftware/steam-audio/blob/master/LICENSE.md),
[Steam Audio third-party notices](https://github.com/ValveSoftware/steam-audio/blob/master/core/THIRDPARTY.md).

## Candidates

| Project | What it covers | License / state | Fit and disposition |
| --- | --- | --- | --- |
| **AudioNimbus + Steam Audio** | HRTF, directivity/distance, geometry occlusion and transmission, reflections/reverb, Ambisonics, probes/pathing. | AudioNimbus MIT OR Apache-2.0; Steam Audio Apache-2.0. Native SDK is already built and direct occlusion has been exercised here. | **Keep as the production baseline.** Broadest mature feature set found and already integrated. First complete the listener wet bus and capability matrix. |
| **OMG Audio** | Rust engine with mesh/BVH propagation, HRTF, wall transmission, diffraction, early paths, coupled reverb/RT60, and Ambisonic output. | Apache-2.0 code; built-in HRIR data carries separate Apache attribution. Rapidly evolving and young; published performance figures are maintainer-reported. | **Run a bounded Firewheel prototype.** The high-level mixer is clip-oriented, but the lower-level `omg_dsp::Renderer::process_block` accepts live mono samples, so procedural modular output is technically possible. No Bevy/Firewheel adapter exists. The main gates are frame-update allocations, shared Ambisonic bus routing, scene ingestion, and pinned API stability. |
| **Resonance Audio** | HRTF, shoebox room reflections and late reverb; host supplies occlusion values. | Apache-2.0; upstream archived/read-only since 2023. | Not a full geometric replacement: room proxies and host raycasts would leave game-specific geometry work. Stale upstream makes it a poor foundation. Useful as a reference for room/reverb controls only. |
| **OpenAL Soft (EFX)** | Positional rendering and effect slots for air absorption, occlusion and environmental reverb. | LGPL-2.0-or-later; maintained. | An audio API/effects layer rather than a scene propagation solver. We would still compute geometry/material effects and would be adding a second host alongside Firewheel. Not a replacement. |
| **`phonon_rs`** | Unofficial Rust rewrite of Steam Audio direct effects: distance, air absorption, directivity, occlusion and transmission. | Apache-2.0; early-stage and explicitly incomplete. | Missing reflections, listener reverb, Ambisonics and pathing in the documented feature set. Does not solve the current audible reverb requirement. Watch only. |
| **Atrium** | Rust audio architecture with a processor abstraction and plans for ray-traced reflections, FDN reverb and occlusion. | License and implementation maturity require direct verification before evaluation. | The README describes important spatial features as future work. Architecture reference, not a present replacement. |
| **bevy_seedling / Firewheel** | Bevy-facing audio graph, playback, routing, effect chains and custom processors. | MIT OR Apache-2.0. | Complementary integration layer, not a geometric acoustic solver. We already use Firewheel; compare only if a concrete reduction in our Bevy/Firewheel glue is demonstrated. |
| **`bevy-raytraced-audio`** | A newly indexed Bevy CPU ray-traced propagation project. | Search-index evidence says MIT; project is extremely new and needs direct code/license review. | Track as an emerging candidate, not a usable dependency yet. Verify the repository, API, audio-thread model, feature scope and activity before a spike. |

### OMG Audio assessment and prototype gate

I inspected upstream commit `4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869` locally.
The useful integration seam is lower-level than the README's clip-oriented `Mixer`:
`omg_dsp::Renderer::process_block` takes arbitrary live mono `f32` samples and emits
point-rendered stereo plus a shared order-2 Ambisonic bus contribution. `omg_core::ParamBlock`
is the simulation-to-render contract, carrying path taps, multiband reverb, and remote
room sends. This makes a procedural Firewheel integration plausible without changing
the game into a clip-based audio engine.

There are still concrete adapter costs. Geometry uses meters and +Z up and must be
converted from Bevy/Avian's coordinate frame and authored materials. A renderer per
live source must consume bounded frame updates; point-rendered stereo can be summed,
while the shared Ambisonic contributions need a coherent aggregate decode. In the
inspected commit, `Renderer::set_params` constructs several temporary vectors, and
`threaded::AudioOut::render` calls `Mixer::apply` when new simulation frames arrive.
The upstream `Mixer` documentation itself says `apply` allocates and is not strictly
real-time safe. We therefore cannot use the packaged threaded host directly in our
callback. The prototype should first determine whether its own bounded Firewheel
handoff can call the low-level renderer safely, and whether fixing/upstreaming reusable
parameter scratch storage is a small focused change.

The pinned checkout's `cargo test -p omg-audio --test fx_mixer` passed (9 tests). This
is upstream unit evidence only, not a Firewheel integration, live-signal test, or
independent acoustic-quality measurement. Code is Apache-2.0; the checked-in HRIR
notice cites SADIE II data under Apache-2.0. Keep media/data notices separate from the
code license. Sources: [OMG Audio upstream](https://github.com/JustGoscha/omg-audio),
[pinned renderer](https://github.com/JustGoscha/omg-audio/blob/4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869/crates/omg-dsp/src/renderer.rs),
[pinned threaded host](https://github.com/JustGoscha/omg-audio/blob/4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869/crates/omg-audio/src/threaded.rs),
[license](https://github.com/JustGoscha/omg-audio/blob/4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869/LICENSE),
[HRIR notice](https://github.com/JustGoscha/omg-audio/blob/4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869/crates/omg-audio/hrir/NOTICE.md).

#### Firewheel live-signal spike (2026-11-18)

`spikes/spike_omg_audio/` now hosts a pinned, one-source prototype. Firewheel generates
a square-wave input, sends it through an OMG `Renderer::process_block` node, and the
node accumulates the 9-channel diffuse bus, decodes it with OMG's `OutputStage`, and
returns stereo. The integration test creates a real Firewheel processor, builds its
OMG path parameters before rendering, and processes sixteen 128-frame blocks. Output
is finite and nonzero; the callback path passes `assert_no_alloc` at 48 kHz. This
confirms that OMG's live-sample API can host one procedural signal in Firewheel without
callback allocation when path parameters are already installed.

This is deliberately not adoption evidence yet: the prototype uses the speaker
fallback (no HRTF dataset), one source, and a fixed parameter frame. It decodes the
Ambisonic bus per node, which must be replaced with one shared decode for a multi-source
mix. It does not exercise frame replacement, parameter-update allocation, game
geometry/material conversion, moving sources/listeners, or quality/performance against
the grotto. In particular, upstream `Renderer::set_params` still builds temporary
vectors; the next prototype step must move those allocations out of callback updates
or provide bounded reusable scratch before testing changing frames under the guard.
The spike pins upstream source by Git revision and remains outside the production
workspace/dependency policy until those gates pass.

### Callback-safety blocker found in the current AudioNimbus reflection handoff

Inspection of the locally installed AudioNimbus 0.17.0 source found a concrete native
memory race in the current reflection-output design. Its `ReflectionsReverbOutput`
contains reflection parameters pointing at Steam Audio-managed impulse-response
buffers. Upstream's `unsafe impl Sync` documentation explicitly says the simulation
thread may overwrite those buffers while the audio thread reads them; it describes the
result as potentially corrupted reverb. Our `triple_buffer` protects publication of
the Rust snapshot, but its `Arc` keeps the wrapper alive, not the native IR memory from
being overwritten. The same handoff is used by the existing per-emitter reflection
path, so adding a listener bus would widen this exposure.

Do not route listener reverb to the callback or claim callback-safe reflection until
the buffer lifetime is made safe. Viable next checks are an upstream AudioNimbus fix
that provides immutable/ref-counted IR snapshots, or a supported API to copy the IR
into application-owned storage on the simulation worker. Copying must stay off the
callback and be measured for allocation/copy cost. If neither is available, disabling
the dynamic reflection path is safer than assuming the race is inaudible. The project
must also exercise callback allocation and replacement with a non-empty snapshot after
the ownership fix.

This is a code-level finding from the exact 0.17.0 crate source, not a runtime race
reproduction. Source: [AudioNimbus reflection output safety comments](https://github.com/MaxenceMaire/audionimbus/blob/main/src/wiring/step/reflections_reverb.rs).

## Comparison against our actual requirements

| Requirement | Steam Audio via AudioNimbus | OMG Audio | Resonance Audio | OpenAL Soft EFX |
| --- | --- | --- | --- | --- |
| Geometric direct occlusion | Documented and verified in our grotto | Claimed/documented; verify against our mesh paths | Host-side occlusion input | Host computes and sets EFX occlusion |
| Material transmission | Documented | Claimed/documented | No comparable geometry solver | No comparable geometry solver |
| Early reflections | Supported; integration quality still to measure | Reflection taps documented; verify direction, timing and quality | Room-model reflections | No geometric reflection solver |
| Late/listener reverb | Supported; game output route remains incomplete | Coupled late field/RT60 documented with approximations | Room model and reverb | Reverb effect, but room parameters/feeds supplied by host |
| Diffraction/pathing | Steam Audio offers propagation/pathing modes; our current integration is unverified | Claimed, with documented approximation | Not equivalent geometric path search | No |
| Bevy/Firewheel integration | Our code exists; listener bus and callback handoff need finishing | No advertised adapter found | No current adapter for our stack | Requires another audio host or substantial integration |
| Realtime behavior in our game | Must finish measuring transfer, callback cost and staleness | Must inspect implementation and measure handoff/cost | Archived code; host model differs | Mature effects API, but solver gap remains |

## Best practices for our implementation and level workflow

1. **Separate propagation from rendering.** Run geometry queries, scene updates and
   acoustic simulation at a bounded cadence on a worker/control thread. The audio
   callback consumes a bounded latest snapshot or precomputed parameters; it never
   waits for simulation.
2. **Keep the realtime boundary explicit.** Use fixed-capacity messages or a
   lock-free/latest-value transfer. Build and destroy SDK objects off the callback;
   return retired objects to a non-realtime thread. Measure callback CPU and ensure
   stale results degrade smoothly rather than blocking.
3. **Route direct and environmental paths separately.** Make dry/direct, early
   reflections and late listener reverb independently audible and measurable. Provide
   a clear dry/wet A/B, wet output meter, and a known loud test signal before tuning
   gain. Avoid multiplying the same wet signal in both per-source and listener buses.
4. **Track scene and result freshness.** Expose acoustic scene revision, mesh/material
   counts, matched emitter count, simulation update age, and per-band wet level/RT60 in
   a developer view. This distinguishes “wrong material,” “no geometry submitted,”
   “stale propagation,” and “wet bus not routed.”
5. **Author acoustics on meaningful meshes.** Mark only sound-relevant geometry;
   choose absorption, scattering and transmission by material. Keep the marker on the
   actual mesh child where the runtime imports geometry. Document representative
   source/listener paths for clear, blocked, indoor, outdoor and around-corner cases.
6. **Test more than the grotto.** Use matched level and source fixtures for open space,
   a small enclosed space, a large enclosure, a single wall with known material,
   multiple occluders, a moving listener, a moving source and changing geometry.
   Compare direct-only, early-only, late-only, and combined output.
7. **Use game-level acceptance criteria.** Record wet/dry RMS and decay over time,
   frequency-dependent changes, latency, CPU/memory and platform. “I can hear an echo”
   in one headset is useful listening evidence, but not a substitute for output routing
   and signal measurements. Headphone review remains necessary for binaural quality.

These practices align with the existing realtime callback rules in `AGENTS.md` and
the open integration tasks in
`openspec/changes/reduce-bespoke-infrastructure/tasks.md` 1.2–1.6.

## Recommendation and next steps

**Do not switch away from AudioNimbus yet.** The current failure to detect reverb is
consistent with incomplete wet-output integration and conservative gain, while the
incumbent already supplies the documented capabilities. Finish a controlled direct /
reflection / listener-reverb signal-path review first.

Run a time-boxed OMG Audio spike in an isolated worktree only after recording the
incumbent’s complete measurements. Pin a commit and cover one static mesh scene plus a
moving source/listener. Evaluate API stability, code and media licenses, platform
builds, output access, callback isolation, CPU/memory and the amount of Bevy/Firewheel
adapter code. Keep or reject it based on measured feature parity and net maintenance
cost. If it performs well, propose upstreaming a Bevy/Firewheel adapter before carrying
a long-lived fork.

Continue the existing OpenSpec assessment rather than creating a duplicate proposal:

- 1.2: exercise the documented capability matrix and acoustic fixtures;
- 1.3: verify geometry/material and dynamic-scene lifecycle;
- 1.4: finish direct, reflection and listener-bus handoff measurements;
- 1.5: complete IPP-free platform builds and notices;
- 1.6: select based on measured coverage and net project-owned code.

## Source register

- [AudioNimbus](https://github.com/MaxenceMaire/audionimbus)
- [Steam Audio documentation](https://github.com/ValveSoftware/steam-audio/blob/master/core/doc/index.rst)
- [Steam Audio guide](https://github.com/ValveSoftware/steam-audio/blob/master/core/doc/guide.rst)
- [Steam Audio Apache-2.0 license](https://github.com/ValveSoftware/steam-audio/blob/master/LICENSE.md)
- [Steam Audio third-party notices](https://github.com/ValveSoftware/steam-audio/blob/master/core/THIRDPARTY.md)
- [OMG Audio](https://github.com/JustGoscha/omg-audio)
- [Resonance Audio (archived)](https://github.com/resonance-audio/resonance-audio)
- [Resonance Audio API](https://github.com/resonance-audio/resonance-audio/blob/master/platforms/common/room_effects_utils.h)
- [OpenAL Soft](https://github.com/kcat/openal-soft)
- [`phonon_rs`](https://github.com/GitGhillie/phonon_rs)
- [Atrium](https://github.com/vospascal/atrium)
- [bevy_seedling](https://github.com/corvusprudens/bevy_seedling)
- [`bevy-raytraced-audio`](https://github.com/sagan-software/bevy-raytraced-audio)
