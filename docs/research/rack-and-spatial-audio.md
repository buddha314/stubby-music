# Research: a VCV-style rack inside a spatial game-audio engine

*2026-10-31. Follows `audio-engine.md` and the `add-audio-engine` engine core. Crate
versions are from crates.io on this date.*

## The question

The modular synth is the core of the game, so it is not ordinary game audio. Players
will patch it the way they patch VCV Rack: knobs, jacks, cables, self-patching. And
what it emits has to be **spatial**: modules sound from where they stand in the
world, and finding things by ear is a core mechanic. What are the best practices,
and what does that change in the engine we just built?

## Finding 1: the rack and the game-audio engine are different machines

Games that use procedural synthesis keep the synth as a self-contained generator
that feeds the game's audio engine, and keep the game engine in charge of mixing and
spatialization. No Man's Sky ran its VocAlien vocal synth as a plugin inside Wwise.
Wwise did the routing and positioning, and the synth only produced sound. The team
also limited procedural synthesis to creature vocals and fauna for cost and risk
reasons. Our synth is the whole game, so we can't make that cut, but the split
holds: **synth generates; game audio places and mixes.**

A modular rack also has requirements a game-audio graph doesn't:

| Need | VCV Rack | Firewheel graph (what `stubby_audio` uses now) |
| ---- | -------- | ---------------------------------------------- |
| Feedback / self-patching | Allowed. Every cable adds a **1-sample delay**, so any topology is valid. | **Rejected.** It is a DAG, and our engine already returns `EngineError::Cycle`. |
| Timing between modules | Per sample. A clock through a utility module arrives exactly 1 sample late. | Per block (256+ frames); events are block-scheduled. |
| Cable contents | Up to **16 polyphonic channels** per cable. | Fixed channel count per edge. |
| Unit of processing | Every module, once per sample. | Every node, once per block. |

Feedback isn't an edge case in Eurorack. FM feedback, self-oscillating filters,
sequencers resetting themselves, and envelopes retriggering their own clocks are all
everyday patches. **A rack built from Firewheel nodes can't do any of them.** The
1-sample cable delay is also what makes the patch order-independent: each sample,
every module reads its inputs from the previous sample's outputs.

### Recommendation: two layers, one audio callback

```
 Firewheel graph (game audio)                                       device
 ┌──────────────────────────────────────────────────────────────┐
 │ RackNode ── stem 0 ─► HRTF spatializer (near, audible)  ─┐   │
 │  (our rack engine,  ─ stem 1 ─► HRTF spatializer         ─┤   │
 │   per-sample, VCV   ─ stem 2 ─► basic panner (far)       ─┼─► master ─► out
 │   semantics)        ─ stem n ─► (virtual: no voice)       │   │
 │ SFX / footsteps / UI / ambience (Firewheel samplers) ────┘   │
 └──────────────────────────────────────────────────────────────┘
```

- **Rack engine (ours).** Runs every module once per sample, with double-buffered
  port values (a 1-sample cable delay), so any patch, cyclic or not, is valid and
  deterministic. Module DSP still uses fundsp, through its per-sample `tick()` API.
  Polyphony comes later, but the cable type should allow it from day one
  (`[f32; 16]` plus a channel count), so adding it doesn't change every module.
- **Hosted inside one Firewheel node.** The rack renders inside the same audio
  callback as everything else, so there is no second clock, drift, or extra latency.
  Its outputs are **emission stems**, one per sounding module in the world.
- **Firewheel stays the game-audio layer.** It does spatialization per stem, game
  SFX and UI sounds, buses, ducking, device handling, and the audio clock. What we
  already built for it (the command boundary, `flush`, events, `sample_to_instant`,
  offline rendering, the test rig) carries over. The modules move from Firewheel
  nodes into the rack.
- *Rejected:* the rack on its own thread feeding Firewheel through a ring buffer.
  That means two clocks drifting apart and a buffer of extra latency. It's only worth
  it if the rack outgrows one core, and VCV handles that differently, by spreading
  modules across worker threads *inside* the engine.

## Finding 2: use VCV's voltage standards, not our own

Our `signal.rs` currently uses ±1.0 audio and gate > 0.5. VCV publishes concrete
standards, and players who know VCV or hardware Eurorack will expect them:

| Signal | VCV standard |
| ------ | ------------ |
| Audio | ±5 V (10 Vpp) |
| CV | unipolar 0–10 V, bipolar ±5 V |
| Pitch | 1 V/oct, f = f₀·2^V; f₀ = 261.6256 Hz (C4) for oscillators, **2 Hz (120 BPM) for LFOs and clocks** |
| Gate | 10 V while active |
| Trigger out | 10 V for 1 ms |
| Trigger in | Schmitt trigger, low ≈ 0.1 V, high ≈ 1–2 V |
| Clock + reset | ignore CLOCK for 1 ms after RESET |
| Range | don't clamp to ±1; saturate only if gain > 1 (hardware rails ≈ ±11.7 V) |
| NaN / ∞ | output 0 instead |
| Level | ±10 V = 0 dBFS at the output stage |

Switching now costs a few constants and test values. Switching after dozens of
modules exist costs a lot more.

## Finding 3: VCV-style control is a separate thread's view of the rack

VCV separates the engine (DSP) from the UI (the module panel): the UI writes
parameter values the engine reads, and reads back lights and meters the engine
writes. For us the "UI" is Bevy rendering a module panel in 3D. Practices worth
copying:

- **Parameters are owned by the rack.** The game sets targets without locks
  (`SetParam`, as now). The rack **smooths** each change over a few milliseconds, so
  turning a knob never produces zipper noise.
- **Lights, meters and scopes flow back without locks.** Use one atomic or
  triple-buffered value per light, smoothed in the rack, and one ring of recent
  samples per scope. The game reads them at frame rate. This is also how the clock
  module's pulsing light should work: read the gate light, not just the tick events.
- **Cable edits are graph edits.** They happen at a sample boundary. A cable that
  carries audio straight to an emitter should fade, while a cable carrying gates or CV
  should switch instantly (the same rule as D7).
- **A patch is data.** Store module kinds, parameter values, cables and world
  positions in a serde form (VCV saves `.vcv` JSON). That one format serves save
  games, undo, and offline test fixtures.

## Finding 4: finding things by listening

The reference is Papa Sangre and its sequel: audio-only games in which players
navigate entirely by real-time binaural audio placed around the listener. What
carries over:

- **Use HRTF, not panning, for anything the player is meant to find.** Panning only
  places sound left to right. HRTF adds front/back and some elevation. Firewheel's
  `SpatialBasicNode` is explicitly "basic panning and filtering". **`firewheel-ircam-hrtf`
  0.5.0** is an HRTF node for Firewheel 0.14 (the one `bevy_seedling` uses for its
  `hrtf` feature), with 10 selectable IRCAM listener profiles. Steam Audio via
  **`audionimbus` 0.16.0** adds occlusion and physically based reverb, but it brings a
  native C++ library. Start with the IRCAM node; consider Steam Audio if occlusion
  becomes a mechanic.
- **Front/back confusion is the known failure of HRTF.** The fix is head movement:
  sound should follow the camera's yaw every frame, so turning reveals where a sound
  is. Offer the HRTF profile as a setting ("which of these sounds is in front?"), and
  fall back to panning on speakers, where HRTF works poorly.
- **Distance needs more cues than volume.** Combine gain rolloff (tuned for gameplay,
  longer than physically real), a low-pass that deepens with distance (air
  absorption), and, once there is a reverb send, a falling direct-to-reverb ratio. A
  propagation delay at roughly 340 m/s makes distant pulses arrive late, which is
  subtle but real.
- **Keep the number of simultaneous sounds small.** Audio-only games limit
  simultaneous sources so players aren't overwhelmed. For us this is also the CPU
  budget. Rank emitters by how loud they'd be at the listener and give the top N
  (start around 16) full HRTF. Give the next tier cheap panning, and render the rest
  **silently**: the rack still computes them, because their signals may feed other
  modules and drive the factory, but nobody hears them. Use hysteresis and a short
  crossfade when an emitter changes tier.
- **A focus mechanic** (hold to listen) can briefly duck everything except what the
  player is facing. It is the same tool as the ranking above, and it turns the mix
  limit into gameplay.

### Occlusion: what engines give you

| Engine | Built-in occlusion | Notes |
| ------ | ------------------ | ----- |
| **Unreal** | **Yes.** Attenuation settings have an occlusion trace (a ray from listener to source on a trace channel), applying a low-pass cutoff and volume reduction, interpolated over time. | Steam Audio and Resonance plugins add physically based occlusion and transmission. |
| **Godot 4** | **No.** `AudioStreamPlayer3D` has distance attenuation, a distance low-pass (`attenuation_filter_cutoff_hz`), and `Area3D` reverb buses. Occlusion comes from add-ons (SpatialAudio3D, Ovani's voxel-based plugin) or a hand-written raycast that lowers that cutoff. | |
| **Bevy 0.20** | **No.** `bevy_audio` 0.20.1 (rodio) does spatial sound with two ear positions on a `SpatialListener` and per-ear distance gain. No occlusion, no HRTF, no filtering. Firewheel, Bevy's planned replacement, isn't upstream yet; `bevy_seedling` targets Bevy 0.19 and is being ported. | Verified in the `bevy_audio-0.20.1` source. |
| **Firewheel 0.14** | **No.** `SpatialBasicNode` does panning and filtering; HRTF comes from `firewheel-ircam-hrtf`. | |
| **Steam Audio** (`audionimbus` 0.16) | **Yes**, plus transmission through materials and physically based reverb. | Native C++ library; `bevy_steam_audio` is work in progress. |

**Recommendation:** do occlusion Unreal-style, ourselves. Each emitter gets an
occlusion factor from a raycast (Bevy has mesh ray casting built in). The factor
lowers that emitter's low-pass cutoff and gain, smoothed over ~100 ms. This is the
same filter the distance cue already needs, so it costs little extra. On an open
mudflat, terrain and the player's own factory are the only occluders, so this is
plenty to start with. Keep Steam Audio as the upgrade path if walls, materials or
reverb become mechanics.

### The design catch: a pure sine is the hardest thing to locate

Localization works on transients, broadband spectra, and the differences between the
ears. A steady pure tone gives the least of all three: no onsets, no spectral detail,
and above about 1.5 kHz its timing cue between the ears becomes ambiguous. The
opening, where the first thing to find is a sine wave, is therefore the worst case
for finding by ear. The clock, which emits clicks, is the best case.

**Decided (2026-10-31): the first discovery is a square wave, found by ear within a
hearing range.** A band-limited square's odd harmonics give it the spectral detail a
sine lacks. It is audible only within a hearing radius (35 m to start) and is always
placed farther from spawn than that radius (40–120 m), so the game still opens in
silence. The player wanders until they first catch it, then follows it in. The clock
is second, and its clicks are the easiest thing in the game to locate. Keep the
square's pitch low (under 1 kHz), where the timing difference between the ears is
unambiguous.

## Proposed changes to the plan

1. **The rack replaces the Firewheel-node modules.** Build a rack engine with VCV
   semantics (per-sample processing, 1-sample cables, feedback allowed), hosted as a
   single Firewheel node that outputs emission stems. The command, flush, event and
   offline APIs stay; `EngineError::Cycle` goes away.
2. **Adopt VCV voltage standards** in `stubby_audio::signal` (audio ±5 V, gates
   10 V, Schmitt-trigger inputs, a 2 Hz clock reference, NaN → 0, no clamping).
3. **Add parameter smoothing, and light/meter feedback** to the module contract.
4. **Spatial layer:** an `Emitter` per sounding module with a world position, HRTF
   through `firewheel-ircam-hrtf`, distance cues (gain, low-pass, delay), and an
   audibility-ranked voice budget with hysteresis.
5. **Spike first:** measure per-sample rack cost at 200 modules, and IRCAM HRTF cost
   per emitter, before fixing the voice budget.
6. **Game design:** the first discovery is a **Square VCO**, audible only within a
   hearing radius that spawn is always outside (decided).
7. **Occlusion:** our own raycast-driven low-pass and gain per emitter; Steam Audio
   later if needed.

## Spike results

Measured in `spikes/spike_rack`; the numbers and decisions are in
`openspec/changes/add-rack-and-spatial-audio/design.md`. In short: the per-sample rack
costs about 8.8 ns per module-sample (200 modules = 8.5% of a 256-frame block). HRTF
costs about 25 µs per emitter per block, roughly 16× panning, and 64 HRTF emitters can
blow the budget. The default is therefore 16 HRTF + 32 panned emitters, with the HRTF
FFT set to 2×128.

## Host options (2026-10-31)

We don't control Firewheel's source, so the options were reviewed:

| Option | Fixes | Costs |
| ------ | ----- | ----- |
| **Own thin host on `cpal`** (recommended) | Bug #145, `Send` engine, `assert_no_alloc` on the live callback, glam split | Device-loss recovery, device-rate handling, a small SFX sampler; ~1–2k lines |
| Fork Firewheel | Keeps today's code working; we can fix #145 | Merging a large crate that is being reshaped for Bevy; still not `Send` |
| `kira` / `oddio` / `rodio` as the base | — | All external too; playback-focused, no HRTF; `oddio` is quiet, `rodio` not strictly realtime-safe |
| Separate synth process (SuperCollider, VCV) | Crash isolation | GPL, IPC, process management; VCV can't be embedded |
| Keep Firewheel behind our own trait | Buys time; mostly in place | The dependency stays; live callback still unguarded |

`cpal`, `fundsp` and `hrtf` stay either way: `cpal` is the I/O foundation everything
uses, and the other two are pure DSP crates we could vendor if they were abandoned.

## Sources

- VCV Rack manual: [Voltage Standards](https://vcvrack.com/manual/VoltageStandards),
  [Polyphony](https://vcvrack.com/manual/Polyphony)
- [Behind the Sound of No Man's Sky: Q&A with Paul Weir](https://www.audiokinetic.com/en/blog/behind-the-sound-of-no-mans-sky-a-qa-with-paul-weir-on-procedural-audio)
- Papa Sangre: [MCV interview with Paul Bennun](https://mcvuk.com/?p=70547),
  [PapaEngine](https://www.mcvuk.com/development/key-release-papaengine),
  [TU Delft thesis on audio games](https://repository.tudelft.nl/file/File_9ccc1677-d20b-4eba-873b-1ddeeba6a01f)
- Rust spatial audio: [audionimbus](https://lib.rs/crates/audionimbus),
  [bevy_seedling](https://lib.rs/crates/bevy_seedling) (`hrtf` feature →
  `firewheel-ircam-hrtf` 0.5.0), [bevy_steam_audio](https://lib.rs/crates/bevy_steam_audio) (WIP)
- `firewheel-nodes` 0.14 `spatial_basic.rs` (read locally)
