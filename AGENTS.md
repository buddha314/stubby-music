# AGENTS.md — stubby-music

Guidance for AI agents (Claude Code, Codex, etc.) and humans working in this repo.
`CLAUDE.md` imports this file; keep the real content here.

## What we are building

A 3D game that fuses **modular synthesis** (Eurorack-style patching: oscillators,
clocks, sequencers, filters, VCAs, connected by patch cables) with a **factory
builder** (Satisfactory-style discovery, placement, and throughput). The factory *is*
the patch: machines are modules, conveyors are cables, and the music is the output
the player is building.

### Resources and recipes: the builder-game model

- Treat this as a factory builder: discovering a waveform resource is akin to
  finding and mining coal in a builder game. World assets are resource nodes;
  recipes combine their yields into modular-synthesis constructions.
- The **lightning bug is a resource node**. It flashes white and makes a brief
  audio-rate click. Its harvested yield and recipe-book entry are **Square wave**,
  not a finished VCO, LFO or clock module.
- A square-wave building block can be used for **VCO**, **LFO** and **gate** roles.
  These are uses of the same primitive, not unrelated item types. All signals
  remain audio-rate `f32`; an LFO is a low-frequency use of an oscillator.
- Other waveform primitives will be found through **other world assets**. Do not
  invent those sources or assume a first machine recipe before it is designed.
- Keep resource yields, discovered recipe-book entries, inventories and recipe
  definitions data-driven. A collected pattern may later be consumed by recipes;
  learning it must not disappear from the recipe book when inventory reaches zero.
- Every lightning bug owns an independent seeded random clock. Use the shared
  `PulseClock` interface (`PeriodicClock`, `RandomClock`); other assets synchronize
  only by explicitly following the same clock ID. Preserve the flash appearance.
- Reuse **Firewheel** for graph/device/scheduling and existing **fundsp** DSP units
  and module wrappers. Compose the bug click from the existing square VCO,
  envelope and VCA. Do not hand-write replacement DSP when the installed crates
  already provide it. Small game-specific timing, metadata and recipe glue are
  appropriate; they are separate from DSP.

### Interaction controls

Follow Satisfactory's interaction conventions where applicable (research:
`docs/research/satisfactory-controls.md`). Use **E** for use/interact/collection;
holding E may collect successive nearby pickups without duplicating an instance.
Use **O** for the recipe book/codex. Reserve **Q** for build, **F** for dismantle,
**Tab** for inventory and **B** for flashlight; do not assign these to unrelated
player actions. WASD and Shift retain movement/sprint roles. Space/C are jump/hold-to-crouch. The grounded player uses free mouse look;
Escape or focus loss releases the cursor, and left click recaptures it.

### The opening

- The world starts on a flat, dry, **cracked desert mudflat** during a **silent
  windstorm** — blowing sand, low visibility, and *no sound at all*.
- Silence is the starting condition. Sound is what the player discovers.
- The first waveform resource the player finds is a **square wave**, yielded by a
  lightning bug. Its click is the first sound in the world and is audible only
  within a hearing range of the resource. Spawn is always out of range, so the game starts silent; the player wanders
  until they catch it, then follows it in by ear. (A square, not a sine, because its
  harmonics make it locatable; a pure sine is the hardest sound to find by ear.)
- The second is a **clock**, which appears in the world as a **pulsating light**. Its
  pulse is driven by the audio clock, not by the frame rate.

### Look and feel

- Full 3D, stylised: **Satisfactory meets Inside**. Industrial scale and readable
  machines, but with Inside's muted palette, fog, silhouettes, and dread.
- **Moody at first, gradually more cheerful.** A single progression value drives
  palette saturation, colour temperature, fog density, and storm intensity as the
  player builds out their patch. Never hard-cut between moods.
- The environment is **procedurally generated** from a seed. The same seed always
  produces the same world.

## Repo map (target layout — create as the work lands)

```
AGENTS.md                 this file
CLAUDE.md                 imports AGENTS.md
openspec/                 requirements and change proposals (see below)
docs/research/            research notes behind decisions
crates/
  stubby_audio/           audio engine — NO bevy dependency
  stubby_game/            the Bevy app
  stubby_recipes/         waveform resources, inventory, recipe book and crafting (no Bevy)
assets/                   runtime assets loaded by Bevy (glTF, textures, shaders)
assets-src/blender/       source .blend files — never loaded at runtime
tools/blender/            headless Blender export scripts
```

## Requirements live in OpenSpec

We track requirements with [OpenSpec](https://github.com/Fission-AI/OpenSpec)
(`openspec` CLI, v1.4). **Do not start a feature without a change proposal.**

- `openspec/specs/` holds the accepted, current requirements, by capability.
- `openspec/changes/<name>/` holds in-flight proposals: `proposal.md`, `design.md`,
  `tasks.md`, and delta specs under `specs/<capability>/spec.md`.
- Workflow: `/opsx:propose` → `/opsx:apply` → `/opsx:archive` (Claude Code slash
  commands in `.claude/commands/opsx/`). Or use the CLI directly:
  `openspec list`, `openspec show <change>`, `openspec validate --strict <change>`.
- Requirements use SHALL/MUST and every requirement needs at least one
  `#### Scenario:` written as WHEN/THEN. Run `openspec validate --strict` before
  committing spec changes.
- When implementation reveals a requirement was wrong, fix the spec in the same
  change. Do not let code and spec drift.

## Architecture rules

### The audio engine is not a Bevy system

Audio runs in `crates/stubby_audio`, which **must not depend on `bevy`**. It owns the
audio device (via `cpal`) and runs the DSP graph on the realtime audio callback
thread. Bevy talks to it only through a message boundary:

- **Game → audio:** `Engine::send(Command)` validates and stages an edit (add/remove
  module, connect/disconnect, set parameter); `Engine::flush()` once per frame
  publishes them over Firewheel's lock-free channel.
- **Audio → game:** `Engine::drain_events` returns timestamped events (clock ticks)
  from `rtrb` rings; `Engine::sample_to_instant` maps their sample times to wall time.
- `Engine` is **not `Send`** (Firewheel's context isn't), so the game never holds it
  directly. `AudioThread` runs it on its own control thread and hands out a
  `Send + Sync` handle. In Bevy that handle is the `Synth` resource
  (`stubby_game::audio::SynthAudioPlugin`): send commands during the frame; they are
  committed as one batch in `Last`, and clock ticks arrive as `ClockTick` messages
  carrying the predicted instant (`at`) each beat is heard.

See `docs/research/audio-engine.md` and the `add-audio-engine` change for why.

**Engine stack (decided by spike, 2026-10-31):** Firewheel runs the graph, device
and scheduling; each module is a Firewheel node whose processor runs
fundsp units. Only `stubby_audio` may name Firewheel or fundsp types. Firewheel
0.14 zeros all output channels when any one is silent, so keep the keep-alive node
from `spikes/spike_firewheel` until that is fixed upstream.

**Realtime rules for anything that runs on the audio thread:**

- Sound-producing modules fade in on add and out before removal (Firewheel does not
  declick node add/remove). Gates and CV switch instantly.
- No allocation, no deallocation, no locks (`Mutex`, `RwLock`), no I/O, no logging,
  no `println!`, no channel that can block. Debug builds enforce allocation with
  `assert_no_alloc`.
- Build nodes *off* the audio thread and send them in; send replaced nodes *back out*
  to be dropped elsewhere.
- Never `.unwrap()` or panic in the callback. A panic there kills sound for the session.

### The audio clock is the musical source of truth

Anything that must be in time with the music — the clock light's pulse, factory
machines firing on beats — is driven by **events from the audio thread carrying a
sample timestamp**, not by Bevy's `Time`. Frame timing jitters; the sample counter
does not. Visuals interpolate toward the audio clock.

### Signals follow Eurorack conventions

All module ports carry `f32` at audio rate, as in VCV Rack — there is no separate
control-rate signal type. Use a nominal ±1.0 range for audio, 1.0 per octave for
pitch CV (V/oct), and gates/triggers high above 0.5. Keep these in one place
(`stubby_audio::signal`) and do not invent per-module conventions.

### Procedural generation is seeded and deterministic

All world generation takes an explicit seed and uses seeded RNG
(e.g. `rand_chacha`) — never `thread_rng()` or the system clock. Generation must be
testable by asserting identical output for identical seeds.

## Blender asset pipeline

- Blender is installed at `~/.local/bin/blender`.
- Sources live in `assets-src/blender/`. Export to **glTF binary (`.glb`)** into
  `assets/models/`. Bevy loads `.glb`, never `.blend`.
- Export headlessly and reproducibly with a script, not by hand:
  `blender -b assets-src/blender/<file>.blend -P tools/blender/export_gltf.py`.
- Apply transforms before export. Use metres, +Y up in Bevy (the glTF exporter
  converts from Blender's +Z up).
- Keep the stylised look in materials Bevy can actually render: PBR base
  colour/roughness/emissive. Do not rely on Blender-only shader nodes, since they will
  not survive export.
- The cracked-mud ground is **generated in-engine**, not modelled. Blender is for
  props, machines/modules, cables, and the player.

## Stack

| Concern | Choice | Version (crates.io, 2026-10-31) |
| ------- | ------ | ------------------------------- |
| Engine | `bevy` | 0.20.1 |
| Audio I/O | `cpal` | 0.18.2 |
| Audio engine (graph, device, scheduling) | `firewheel` | 0.14.0 |
| DSP inside modules | `fundsp` | 0.23.0 |
| RT messaging | `rtrb`, `triple_buffer` | 0.4.1 / 9.0.0 |
| RT safety | `assert_no_alloc` (debug) | 1.1.2 |
| Noise | `noise` | 0.9.0 |
| Particles (sand) | `bevy_hanabi` | 0.19.0 |

Check a crate's Bevy compatibility before adding it, since Bevy plugins lag Bevy
releases. Pin exact minor versions in `Cargo.toml`.

## Conventions

- Rust 2024 edition, `cargo fmt`, and `cargo clippy --all-targets -- -D warnings`
  clean before committing.
- Tests: `cargo test --workspace`. DSP modules get offline render tests (render N
  samples, assert frequency/amplitude/phase) — they need no audio device.
- Commit only when asked. Branch off `main` for feature work.

### Working alongside other agents

Several agents work in this repo at once. To avoid stepping on each other:

- **Use your own git worktree** off `main` (`git worktree add ../stubby-music-<topic>
  -b <topic> origin/main`). Never edit, switch branches in, or pull into another
  session's checkout. The main checkout at `~/src/stubby-music` is usually someone's
  live workspace, and often has the Jackdaw editor open on it.
- **Claim tasks in OpenSpec:** add `(claimed: <agent>)` to the task's line in
  `tasks.md`, in the PR with your first commit for it. Check claims before starting.
- **Shared files** (`Cargo.lock`, `crates/*/src/lib.rs`, `jackdaw/Cargo.toml`) conflict
  most often. Keep edits to them small, and update from `main` before opening a PR.
- **Current owners (2026-10-31):** Blender assets (desert-opening 1.3, 1.4) → Claude.
  Lightning-bug pulse → the `add-lightning-bug-pulse` agent. Rack and spatial audio
  (`add-rack-and-spatial-audio`) → unclaimed, ready to start.
- Fast iteration: enable Bevy's `dynamic_linking` feature for dev builds only,
  never in release.

### Waveform collection audio and asset status

Square wave is an asset **in development**. Use the stable `square_wave` item ID
and waveform metadata; do not assume its final visual asset or production recipe.
A successful waveform harvest emits its actual yield stacks and plays one
listener-level synthesis confirmation per distinct waveform type, not per item.
Use the shared existing VCO/envelope/VCA composition. Failed or repeated collection
must stay silent. Bugs currently yield 2 items, configured in the catalog.
See `docs/research/firebug-audio-and-yields.md` for duration and mining research.

### Infinite mines and exhaustible resources

Loose lightning bugs are **exhaustible** and collected once. Scrub-oak trees are
**infinite mines** that will produce fresh finite lightning bugs. A tree's bug
production increases as water quality and light quality improve. Infinite means
no finite reserve, not infinite instantaneous output. The source types, catalog
mine definition and editor `MineNode` exist; production rates, quality simulation,
spawn loops and tree visuals remain deferred in
`openspec/changes/add-scrub-oak-production/`. Keep mines separate from E pickup
queries. The domain chain is scrub oak mine → finite bug → square-wave items.

### Shared Jackdaw development fork

Use the shared editor source checkout `/home/buddha/src/jackdaw`, not a per-game
copy. Its development `origin` is **https://github.com/buddha314/jackdaw.git**.
Keep **https://github.com/jbuehler23/jackdaw.git** as `upstream` for reference and
selective integration. Develop and publish fixes to the buddha314 fork; do not
wait for upstream acceptance before using them locally or in this game. Preserve
this remote convention when fetching, opening PRs, or configuring new checkouts.
