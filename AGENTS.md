# AGENTS.md — stubby-music

Guidance for AI agents (Claude Code, Codex, etc.) and humans working in this repo.
`CLAUDE.md` imports this file; keep the real content here.

## Resume after reboot: display-flicker workaround

The user reports applying the grubby workaround before reboot: remove
`amdgpu.dcfeaturemask=0x8` and add `amdgpu.dcdebugmask=0x410`. Do not apply it again
automatically. After reboot, first read
`/sys/module/amdgpu/parameters/dcdebugmask`: expected **1040** (decimal 0x410), and
check `/proc/cmdline` for the intended AMD options. This confirms configuration,
not that the flicker is fixed. If the option is absent, investigate before graphics.

Resume in `/home/buddha/src/stubby-music-live-listen` on `add-jackdaw-live-listen`.
Rebuild the game/adapter from the moved worktree before testing. Use only
`tools/jackdaw_timed_test.py` for graphical diagnostics: it has an independent
20-second kill timer. Begin at fixed 60Hz; then compare 165Hz with automatic
60Hz recovery. No unbounded graphical launches while the regression is open.
Do not reboot or log out the user automatically. Post-reboot verification is complete: dcdebugmask=1040 and a 20-second 165Hz
Play test passed with user-confirmed no flicker. This is bounded validation, not
a claim of indefinite stability. Keep the watchdog for further graphical tests.

Evidence and detailed history: `docs/research/jackdaw-live-listen.md` and
`/home/buddha/.cache/stubby-music-live-listen/evidence/`.

## What we are building

A 3D game that fuses **modular synthesis** (Eurorack-style patching: oscillators,
clocks, sequencers, filters, VCAs, connected by patch cables) with a **factory
builder** (Satisfactory-style discovery, placement, and throughput). The factory *is*
the patch: machines are modules, conveyors are cables, and the music is the output
the player is building.

**Theme: the music of nature brings light to the world.** The player grows plants
and animals by singing or playing music to them. Where life grows, light quality
rises and the bleak desert blooms (see *Look and feel*). **Streams are staff music**:
they deliver music to plants and animals as nutrients, the world's own patch cables.
They start as raw mud from burblers (below) and are cleaned up by other constructors.
Music is how the world gets better; it is never just decoration. **All machines are
instruments**: every machine is a rack the player patches and plays, controlled by
audio, as the burbler is.

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
player actions. **Tab** opens the inventory screen and the upper right shows
the goals hub: long- and short-term goals as the items to make, as Satisfactory's
milestones and Space Elevator phases (`openspec/changes/add-inventory-and-goals-hud/`).
Goals guide; they never end the game. WASD and Shift retain movement/sprint roles. Space/C are jump/hold-to-crouch. The grounded player uses free mouse look;
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
- **Bleak and hazy at first, magically colourful where life returns.** The desert
  opens bleak: hazy, desaturated, brown-grey. Music played to plants and animals
  grows them, and where life grows the world blooms into solarpunk by day and
  lunarpunk by night. The blend is local, not global: **light quality**, a field of
  10 m cells whose value follows game conditions (mostly the density of plants and
  animals). Time of day is global. Lighting for every
  combination is art-authored in `assets/lighting/lighting_keys.json`; see
  `docs/art/style-guide.md`. Never hard-cut between looks.
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
  stubby_recipes/         rules files, inventory, recipe book, crafting, goals (no Bevy, ever)
  bevy_stubby_recipes/    the crafting system as a thin Bevy plugin over stubby_recipes
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

### Rules are data; the schema is code

Game rules (items, resources, mines, recipes, goals) are JSON rows under
`assets/recipes/`, interpreted and validated by `stubby_recipes`. **The designer owns
the rows; Codex owns the schema and validator; OpenSpec owns rule types.** Two flows:

- **Data PR:** a new row or value of an existing type (a recipe, a goal, a yield). Edit
  the JSON only; `cargo test -p stubby_recipes` must pass. No OpenSpec change.
- **Rule PR:** a new rule type (a field, a trigger, an unlock kind, an accounting
  mode). OpenSpec change first, then schema and validator code, then example rows.

The crafting system (`stubby_recipes`: rules, inventory, recipe book, crafting,
goals) is **ours, not a community crate**; no Bevy crafting or inventory crate fit
(`docs/research/crafting-and-inventory-crates.md`). Do not swap one in without an
OpenSpec change. It is shaped as a **Bevy community plugin** (`add-recipes-bevy-plugin`):
the Bevy-free core `stubby_recipes`, a thin `bevy_stubby_recipes` plugin the game adds
with `add_plugins`, and only presentation (HUD, input, screens) in `stubby_game`. Never
add a Bevy dependency to the core, not even optional; Bevy versions live only in the
plugin. Use community crates around it (serde, schemars, and as planned
`bevy_common_assets`, `moonshine-save`).

How to add or change a rule (designer or agent, same steps):

1. Edit the domain file: `items.json`, `resources.json`, `mines.json` or
   `recipes.json` (goals will be `goals.json`). There is no `catalog.json`; do not
   recreate it or read rules from anywhere else.
2. Run `cargo run -p stubby_recipes --bin rules-fmt`: it sorts rows, regenerates
   `schema/`, appends new IDs to `ids.lock` and reports every error.
3. `cargo test -p stubby_recipes` passes; open the PR as **Data** in the template.

Never hand-edit `assets/recipes/schema/` or remove lines from `ids.lock`; both are the
formatter's. Code reads rules only through `stubby_recipes` (`shipped_catalog()`,
`Rules::load`), never by parsing the files itself. When a rule type changes, update
the schema code, run `rules-fmt`, and update the files in the same PR. The authoring
guide for designers is `assets/recipes/README.md` (root `README.md` has the short form).

Rows never hold logic: if a row needs a condition or formula, that is a new rule type.
IDs are `snake_case` `[a-z0-9_]`, unique across all registries, and part of the save
format: rename or remove one only with a row in `assets/recipes/migrations.json`.
Loading is strict (unknown fields fail) and reports every error with file, ID and
field. See `openspec/changes/add-rules-data-conventions/` and
`docs/research/crafting-and-goal-tracking.md`.

### Procedural generation is seeded and deterministic

All world generation takes an explicit seed and uses seeded RNG
(e.g. `rand_chacha`) — never `thread_rng()` or the system clock. Generation must be
testable by asserting identical output for identical seeds.

## Blender asset pipeline

- Blender is installed at `~/.local/bin/blender`.
- Sources live in `assets-src/blender/<category>/`. Export to **glTF binary (`.glb`)**
  into the same folder under `assets/models/` (`creatures`, `flora/<family>`,
  `machines`, `modules`, `props/<family>`, `streams`). Bevy loads `.glb`, never
  `.blend`.
- Export headlessly and reproducibly with a script, not by hand:
  `blender -b assets-src/blender/<category>/<file>.blend -P tools/blender/export_gltf.py`.
- Apply transforms before export. Use metres, +Y up in Bevy (the glTF exporter
  converts from Blender's +Z up).
- Keep the stylised look in materials Bevy can actually render: PBR base
  colour/roughness/emissive. Do not rely on Blender-only shader nodes, since they will
  not survive export.
- The cracked-mud ground is **generated in-engine**, not modelled. Blender is for
  props, machines/modules, cables, and the player.
- **Levels are designed with Jackdaw prefabs**, the standard editor workflow: art
  delivers the glb, a prefab wraps it with its gameplay components, and levels place
  prefabs. Every placeable world asset has one under `jackdaw/assets/prefabs/`. Make
  new ones in Jackdaw (place the glb, add components, **Save as Prefab**; see
  `jackdaw/README.md`) and edit them there. `cargo test --test prefabs` in `jackdaw/`
  validates them.
- Ship each glb in its starting pose using **node scale** (Blender's hidden-in-render
  flag does not export).
- **Levels of detail belong to the model, never to a placement** (Unity's, Unreal's
  and Godot's practice, and Jackdaw's). Hand-made levels go inside the one glb as nodes
  named `<id>_LOD0`, `<id>_LOD1`, ...; Jackdaw's **Import LOD levels**
  (`model.lod.import`) reads them into the model's import settings, or **Generate**
  (`model.lod.generate`) simplifies the model's own meshes. **Apply** writes the
  settings to `<model>.glb.meta`; commit it. Level designers place one prefab; the
  engine picks the level at runtime. No `LodGroup` on prefabs, no `_LOD1.glb` files.

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
| Particles (sand) | `bevy_hanabi` | 0.19.0 (needs Bevy 0.19; no 0.20 release yet) |
| Physics | `avian3d` | 0.8.0 |
| Player controller | `bevy-tnua`, `bevy-tnua-avian3d` | 0.33.1 / 0.13.1 |
| Input | `bevy_enhanced_input` | 0.27.2 |

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
- **Never share a `target/` directory between checkouts that are in use.** Cargo names a
  workspace crate's artifacts the same in every checkout, so a shared
  `CARGO_TARGET_DIR` lets one agent's build replace another's test binaries. The
  result is missing or flaky tests that look like real failures (2026-10-31). Each
  worktree builds in its own `target/`, even though the first build is slower.
- **Engine module IDs are picked by hand, so keep to your range:** bug clocks 1–3,
  bug voices 100–199, module showcase 8000–8999, audio demo 9000–9999, collection cues
  10 000 and up. A duplicate `ModuleId` makes the engine reject the module. A shared
  allocator replaces these ranges (`add-module-asset-binding` 2.1).
- **Current owners (2026-10-31):** the work is split by discipline.
  - **Claude:** Blender assets and the art style guide — everything under
    `tools/blender/`, `assets-src/blender/`, `assets/models/`, `assets/modules/`,
    `assets/lighting/` and `docs/art/`, and the `add-art-direction` change.
  - **Codex:** services and game logic — `crates/`, including the day/night cycle and
    light-quality field (desert-opening 3.4–3.7), placement (2.5), the module ID
    allocator (`add-module-asset-binding` 2.1) and `add-rack-and-spatial-audio`.
  - Art hands over data and models, not code: when an asset needs game support, write
    it as a task in the owning change rather than editing the crate.
  Codex completed bounded post-reboot validation for audio-engine 5.5 /
  module-asset-binding 2.5 in `add-jackdaw-live-listen`: user confirmed no flicker
  during 20-second 165Hz Play. Separate 10ms on-screen timing remains open.
  Use the independent 20-second diagnostic watchdog for graphical tests.
  Update this line when you claim work.
- **Next milestone: the real opening scene.** Once the burbler, the lightning bugs,
  the stream and the base audio component are in the game, start iterating on the
  opening scene itself rather than on parts.
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

Loose lightning bugs are **exhaustible** and collected once per availability period.
They replenish at the next night boundary strictly after collection. Catalog policies
also support timed returns and no replenishment. The shared calendar uses 45-minute
days and 15-minute nights at 1×; bracket keys change simulation speed and backslash
resets it. Audio pitch and musical clocks remain sample timed. Scrub-oak trees are
**infinite mines** that will produce fresh finite lightning bugs. A tree's bug
production increases as water quality and light quality improve. A tree **starts
dormant** (winter) and escalates to **full bloom as clean water is applied**; bugs
emerge from its open blooms, so a dormant tree makes none. Trees vary procedurally:
seeded variants plus per-instance variation from the world seed (`assets/flora/`).
Infinite means no finite reserve, not infinite instantaneous output. The source types, catalog
mine definition and editor `MineNode` exist; production rates, quality simulation,
spawn loops and placement remain deferred (the tree asset exists) in
`openspec/changes/add-scrub-oak-production/`. Keep mines separate from E pickup
queries. The domain chain is scrub oak mine → finite bug → square-wave items.

### The mudpit burbler: a clock mine you configure

A **mudpit burbler** is a gurgling spring of mud that **bubbles at a BPM**: it is
the game's clock source. It is an **organically round pool** of mud, and a stream
about 1 m wide leaves it between banked sides. Its bubbles burble on the beat (baked
one-beat glTF clips the game plays at the BPM). It is a resource node in the Satisfactory sense, the
equivalent of an iron ore node, so the player does not pick it up.

- **Configure, Satisfactory-style:** E at the burbler opens a **HUD showing a basic
  Eurorack**; Escape closes it. Nothing is built in the world. The rack has three
  places: the **clock** (built in, provided by the burbler; it cannot be removed; its
  BPM knob sets the BPM, starting at the burbler's seeded tempo), a **VCO** slot (a
  square wave from a lightning bug makes a Square VCO), and a **volume** module (a VCA
  with a built-in envelope). The player **patches the voice with cables**: clock GATE →
  volume TRIG, VCO OUT → volume IN; volume OUT is normalled to the outlet. Patch cables
  are a general asset and **fully elastic, as in VCV Rack** (`assets/cables/`).
- **Every machine gets a rack, and all racks share one implementation**: a common
  `Rack` plus a per-machine `RackMachine` trait. Each machine's configurator is data, a
  configuration recipe of slots by component kind and width; empty slots show labelled
  outlines of what goes there (`add-rack-configurations`, `assets/hud/`).
- **Mud streams carry the music.** Streams of mud flow from a burbler's outlet and
  **pulse at its BPM**, so a stream is the played sequence, laid out in space. The
  water is **sheet music**: a staff ribbon runs along it, its lines a fixed distance
  apart and its wave travelling downstream; each beat's note rides it at the water's
  speed, so note spacing shows velocity, and a bar line every four beats keeps
  measures in time with the clock's BPM. The
  mud is **raw**: muddy on purpose, like ore. It is cleaned later by a **filter
  mini-rack** (direction, not yet designed): **high-pass filters separate out pure
  liquid, low-pass filters separate solids and nutrients**, into separate channels,
  controlled via audio. The stream's water goes **from muddy to clear**.
- **Reeds are the stream's ingredient**: **6 reeds build one stream segment**, and reeds
  outline every laid segment's banks. They are a frequent renewable, as easy to get as
  leaves in Satisfactory: abundant on wet ground, harvested with E (hold E for nearby
  clumps), cut to stubble, and back after a short interval (`openspec/changes/add-reeds/`).
- **Port highlighting.** The burbler's mouth carries an upright halo arch, its stream
  port. When the player extends a stream from it, the arch lights to show the
  connection point, as a generator's port does when you drag a conveyor from it in
  Satisfactory; it stays softly lit once connected.
- **Stream hologram.** While extending, the planned stream shows as a holographic
  preview, like a Satisfactory conveyor before it is built. That is its own asset,
  separate from the burbler.
- Requirements: `openspec/changes/add-mudpit-burbler/`. Art (Claude) builds the
  burbler, the HUD rack's modules and, later, streams and reeds; game logic (Codex)
  builds the interaction, the HUD, the patch, streams and production.

### Shared Jackdaw development fork

Use the shared editor source checkout `/home/buddha/src/jackdaw`, not a per-game
copy. Its development `origin` is **https://github.com/buddha314/jackdaw.git**.
Keep **https://github.com/jbuehler23/jackdaw.git** as `upstream` for reference and
selective integration. Develop and publish fixes to the buddha314 fork; do not
wait for upstream acceptance before using them locally or in this game. Preserve
this remote convention when fetching, opening PRs, or configuring new checkouts.

**We do not change the editor except to fix bugs, and never adapt it to this game.**
Game needs (components such as a player start, prefabs, editor-visible markers) live
in this repository, through Jackdaw's own extension points: reflected components,
`HiddenInGame`, prefabs. The editor is kept current with upstream and runs
system-wide through soft links, `~/.cargo/bin/jackdaw` and `~/.cargo/bin/jd` →
`~/src/jackdaw/target/debug/`; `tools/jackdaw.sh` uses the same build.

### Fluids and stream progression

Muddy water is an initial raw resource. Every fluid needs viscosity as gameplay
data. Cleaning water improves quality and usable nutrient provision and lowers
viscosity so streams move faster, supporting mine productivity such as scrub-oak
lightning bugs. Keep fluid transport deterministic and off the audio callback;
render flow from simulation data and keep musical pulses on the sample clock.
Requirements and implementation guidance are in
`openspec/changes/add-fluid-quality-and-viscosity/`; exact rates, quality curves
and cleaning recipes remain deferred. Coordinate stream visuals with the art
owner and the existing muddy-to-clear stages.
