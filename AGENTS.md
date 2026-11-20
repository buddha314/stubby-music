# AGENTS.md — stubby-music

Guidance for AI agents (Claude Code, Codex, etc.) and humans working in this repo.
`CLAUDE.md` imports this file; keep the real content here.

## Graphical testing and the display flicker

**Current setup (2026-11-03):** the monitors are driven by the **NVIDIA RTX 5070 Ti**
(`nvidia` driver, Vulkan) under **KDE Plasma (Wayland)**. The main monitor, DP-6, is 4K
with 60/95/144/240 Hz modes (no 165 Hz). The old AMD flicker workaround
(`amdgpu.dcdebugmask=0x410`) is no longer on the kernel command line and no longer
applies; do not reapply it. On this setup, bounded 20-second Jackdaw Play tests at 60 Hz
and at 144 Hz (with return to 60 Hz) looked clean to the user. They were repeated
on Jackdaw main / Bevy main (fork 4ff3eddc, Bevy c9c397ba): 60 Hz and 144 Hz Play, the
placed-prefab review and the standalone game all looked clean too. This is bounded
validation, not a claim of indefinite stability.

Use only `tools/jackdaw_timed_test.py` for graphical diagnostics: it has an independent
20-second kill timer. Build `stubby_jackdaw` first so Play has time to run. On KDE it
switches refresh with `kscreen-doctor` and restores each output's original mode; on
GNOME it uses Mutter's DisplayConfig. Do not reboot or log out the user automatically.

Evidence and detailed history: `docs/research/jackdaw-live-listen.md` and
`/home/buddha/.cache/stubby-music-live-listen/evidence/`.

## What we are building

The story is `NARRATIVE.md` (the user's: from desolate mudflat to solarpunk city); design
notes that serve it (mythic regions, the vardo base, styling) are `docs/design/design-guide.md`.
The player character is **Omar** (`docs/art/omar.md`); for now only his gloves (the
first-person hands) and his shadow are built (`add-omar-gloves-and-shadow`): the gloves
(`assets/characters/omar_gloves/`) are modelled in the held item's frame and mount with it
(`stubby_player::hand`); the shadow body (`assets/characters/omar_shadow/`) follows the
player on `stubby_core::SHADOW_ONLY_LAYER`, which the sun sees and no camera does
(`stubby_player::shadow`). Changing the recorder's hold pose means regenerating the gloves.

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
  Each flash a placed bug appears at a fresh seeded height in `hover_metres`
  (`assets/creatures/lightning_bug/asset.json`, 1.3-1.7 m, still being tuned), and it
  always turns broadside to the player's camera: end-on, its thin body vanishes.
- Reuse **Firewheel** for graph/device/scheduling and existing **fundsp** DSP units
  and module wrappers. Compose the bug click from the existing square VCO,
  envelope and VCA. Do not hand-write replacement DSP when the installed crates
  already provide it. Small game-specific timing, metadata and recipe glue are
  appropriate; they are separate from DSP.

### Interaction controls

Follow Satisfactory's interaction conventions where applicable (research:
`docs/research/satisfactory-controls.md`). Use **E** for use/interact/collection;
holding E may collect successive nearby pickups without duplicating an instance.
Use **O** for the recipe book/codex. **What E acts on comes from avian3d, not hand-written distance checks**
(`interaction.rs`): every pickup and machine `Interact` point has a small kinematic sensor
marker that follows it, one `SpatialQuery` cylinder (3 m, horizontal reach) at the player
finds them, and the nearest becomes `InteractTarget`, which collecting, prompts and racks read.
Never put a physics body on a visual whose scale animates (bugs flash from scale 0). **H** brings up the field recorder and puts it away; the hand holds one item at a time (`hand.rs`), and each future hand item gets its own key. The game opens with the recorder in hand, and **collecting a sound needs it** (reeds need nothing): its record light answers each capture, and its eight bars meter the engine's **world bus** (`WorldLevel`; world and music only; interface sounds go to `PortRef::ui_output()` so they never move it) (`recorder.rs`, `sync-recorder-feedback`). **Q** is build: it opens the **build menu** (`stubby_game::build`, Satisfactory's: a tile per `buildables.json` row with icon and cost). **Campfire** places a hologram at the aim (amber, red when short of its cost or out of place) and a click builds it; every buildable places this way, by its row's `placement` (a purifier snaps across a laid stream). **Stream** enters stream build mode, laid like a Satisfactory conveyor (`stubby_streams`, `openspec/changes/add-stream-building/`): aim by a free port (burbler mouth, stream end) and click to anchor, the hologram curves to the aim, click lays it (6 reeds a segment, from `buildables.json`) and chains from its end; right click drops the anchor; Escape or Q leaves. Reserve **F** for dismantle,
**Tab** for inventory and **B** for the disco light (labelled **Scanner** in the key hints, so the mirror ball is a surprise; a toggle, not a hand item: the mirror ball floats beside you and its 48 small beams reach out to 30 m (16 more shoot upward, for lighting what is above later) and every pickup they pass shows as a hologram with a beacon, and bugs stay lit between flashes; `flashlight.rs`); do not assign these to unrelated
player actions. **Tab** opens the inventory screen and the upper right shows
the goals hub: long- and short-term goals as the items to make, as Satisfactory's
milestones and Space Elevator phases (`openspec/changes/add-inventory-and-goals-hud/`).
Goals guide; they never end the game. A goal done is announced as Satisfactory
announces a milestone: a banner drops in at the top, STEP COMPLETE or GOAL COMPLETE over its
name, with a short chime (`stubby_game::goal_banner`; chime from `tools/audio/make_goal_chime.py`). The player starts with 25 reeds, one campfire's worth
(`start.json`), and the first chore is **Build a campfire** (Q; condition `built`); the
opening scene has none placed. The first chores (short-term goals met by a condition, not items) go on with
**Build a heartbeat** (configure the burbler so it sounds), then **Water the tree** (lead
its stream to a scrub oak; needs digging, `openspec/changes/add-digging/`), and later
**Light the fire**: a campfire within 5 m of a sounding burbler or its stream borrows that
clock (a `borrowed` rack place), and a whole bar with the snail sequencer's gates on 2 and 4
lights it; it dies to embers when the clock stops (`stubby_game::campfire`). **Streams
accumulate a composition** (`add-stream-layers`): the heartbeat puts a note on the staff on
every beat. The player builds a campfire from reeds (Q, 25 reeds, `buildables.json`), places
it, then configures it (E): placed within 5 m of a stream it snaps beside it and joins it there,
its rack inherits the stream's voice (the burbler's VCO, fixed and patched; the player adds
only the sequencer), a ghostly staff runs from the join to the fire and rises, and its 2
and 4 notes (ember) ride the staff downstream of the join, where its voice is heard too.
**A join updates the chain, it does not add to it** (user, 2026-11-18): while the campfire
plays, the burbler's own voice is silent (`stubby_streams::ChainUpdated`) and the
heartbeat's notes leave the staff at the join, drawn along the ghost staff into the fire and
up; only the campfire's version plays downstream. Its rack shows the inherited modules (the
burbler's clock and VCO), then a 2HP **stream blank** divider (`assets/modules/stream_blank/`),
then its own (sequencer and VCA). Anything downstream inherits the latest
version; splitting a stream comes later. WASD and Shift retain movement/sprint roles. Space/C are jump/hold-to-crouch. The grounded player uses free mouse look;
Escape or focus loss releases the cursor, and left click recaptures it; a second Escape opens the pause menu (Satisfactory's escape menu: Resume Game, Exit to Desktop; `stubby_game::pause_menu`), and Escape closes any open screen. E opens a machine's rack and E again closes it.

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
  lunarpunk by night. The blend is local, not global: **light quality**, how clean the
  air is. **Foliage cleaning the air is a key mechanic**: each plant or tree cleans the
  air within a radius, more as it grows and blooms (`clean-air-by-foliage`; this replaced
  the old 10 m cell field). Time of day is global. Lighting for every
  combination is art-authored in `assets/lighting/lighting_keys.json`; see
  `docs/art/style-guide.md`. Never hard-cut between looks.
- **Three lighting stages (decided 2026-11-06):** bleak haze, then `clearing`, where the air clears
  to a burning sky with hard silhouettes, then bloomed (`add-clearing-air-stage`; not authored yet).
- **Outside nature assets:** check `docs/research/nature-asset-sourcing.md` first. This repository
  is public, so only CC0 assets may be committed; record each asset's licence beside it.
- The environment is **procedurally generated** from a seed. The same seed always
  produces the same world. **The opening too** (user, 2026-11-19): burblers, reed beds,
  rock formations, the oak grove and props come from ordered seeded rules
  (`add-hybrid-world-placement` 2b); hand placement is an overlay only where that fails.
  The rules are `assets/world/opening.json`; `stubby_jackdaw/src/seeded_layout.rs` spawns them
  when the scene spawns (Play or file), as model sources the prefab bridge and binders bring
  alive, around the authored objects (kept for now: spawn composition, bugs, grotto). Only
  Play shows them, not the editor's edit view. `--no-seeded-layout` skips it;
  `--layout-review=<dir>` saves an overhead and a reed-bed view. The ground is ±108 m.

## Repo map (target layout — create as the work lands)

```
AGENTS.md                 this file
CLAUDE.md                 imports AGENTS.md
openspec/                 requirements and change proposals (see below)
docs/research/            research notes behind decisions (docs/art, docs/audio, docs/design beside it)
crates/
  stubby_audio/           audio engine — NO bevy dependency
  bevy_stubby_audio/      service: the engine in Bevy (Synth, ClockTick, module IDs, pulse bus)
  stubby_core/            service: the few contracts systems share (screens, keys, HUD camera, sets,
                          GroundHeight, Interactable)
  stubby_player/          system: movement and look, the hand, what E reaches (Interactable)
  stubby_life/            system: plants and animals; today the lightning bug (Harvested, GroundHeight)
  bevy_stubby_rack/       system: the rack screen, cables, knobs, module assets; takes items via Stock
  stubby_world/           system: the desert, time, world generation, placed-prefab lifecycle (binders)
  stubby_streams/         system: laying mud streams; bar lines and voices ride them (MachineVoice)
  stubby_ui/              service: shared HUD widgets (item icons, slots, tooltips)
  stubby_game/            the Bevy app; `tools/run_game.sh --bare` runs it alone, with no scene
  stubby_rack/            rack modules as data (module.json) and the engine patch they plan (no Bevy, ever)
  stubby_recipes/         rules files, inventory, recipe book, crafting, goals (no Bevy, ever)
  bevy_stubby_recipes/    the crafting system as a thin Bevy plugin over stubby_recipes
  stubby_rules_server/    tool: the rules designer, a web page on 127.0.0.1 (no Bevy, never the game)
  stubby_jackdaw/         the game: stubby_game plus the scene, the seeded layout and the editor bridge
                          (`cargo run`, `tools/run_game.sh`)
assets/                   the one asset folder: scene.bsn, asset families (models + prefabs + profiles),
                          recipes/ (the rules), world/ (the seeded layout's rules), shared shaders, ui, audio
jackdaw.toml              the Jackdaw project is the repo root (package = stubby_jackdaw)
assets-src/blender/       source .blend files — never loaded at runtime
tools/blender/            headless Blender export scripts
tools/                    run_game.sh, jackdaw.sh, build_steam_audio_sdk.sh, ci_local.sh,
                          jackdaw_timed_test.py (the 20 s watchdog), icons/, audio/, assets/
previews/                 asset previews, mirroring assets/ (never loaded by the game)
spikes/                   throwaway experiments, their own Cargo workspaces
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

### The game is independent systems in layers

Each part of the game is its own crate, and dependencies only point down
(`split-game-into-systems`; `cargo deny check` enforces it from `deny.toml`, in CI and
`tools/ci_local.sh`, along with licenses, advisories and sources as Bevy does):

| Layer | Crates | May depend on |
| --- | --- | --- |
| cores | `stubby_audio`, `stubby_rack`, `stubby_recipes` | other cores; never Bevy |
| services | `stubby_core`, `stubby_ui`, `bevy_stubby_audio` | cores |
| systems | `bevy_stubby_rack`, `bevy_stubby_recipes`, `stubby_player`, `stubby_life`, `stubby_world`, `stubby_streams` | services, shared cores, their own core |
| game | `stubby_game`, `stubby_jackdaw` | anything |

- **Systems never depend on each other**, nor on another system's core. They talk
  through contracts in `stubby_core` (`Stock`, `Interactable`, `Harvested`,
  `GroundHeight`, `PlacedAsset`, `AssetClock`, set labels) and their own messages.
- **A rule that joins two systems lives in the game** (`stubby_game/src/glue.rs`,
  `resources.rs`, `waveform_audio.rs`), never inside either system: an item mounting as a
  module, the bug's pitch as a knob default, a collected source being `Harvested`.
- **`stubby_core` holds contracts only**: an item goes there if at least two systems need
  it and it has no behaviour. Shared behaviour is a service (`stubby_ui`,
  `bevy_stubby_audio`).
- **A new crate gets a line in `deny.toml`** (its `wrappers`: who may depend on it). A new
  dependency must have an allowed license (or a named, reasoned exception) and come from
  crates.io, Bevy's repository or a buddha314 fork. Cores
  never reaching Bevy is a `cargo tree` check in the same scripts.
- **Placed prefabs come alive through binders.** `stubby_world`'s lifecycle names no
  asset kind; whoever owns a kind registers an `AssetBinder` (the game's glue does for
  the bug, modules, mines, reeds). A new placeable kind adds a binder, not a branch.
- `stubby_game` is now only assembly, glue, the inventory screen and review drivers.


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

**The master clock is 1 BPM** (user decision, 2026-11-16, as the Godot game decided
2026-11-13): `bevy_stubby_audio::MasterClock`, read from the engine's sample counter. The
day cycle runs on it: the calendar advances by master-clock time times the speed keys'
setting, so at 1x a day is 45 master beats and a night 15. **Every tempo clock is a whole
multiple of it** (`stubby_audio::MasterLockedClock`, `lock-clocks-to-master`): BPM is a
whole number, a 90 BPM burbler is x90, and its beats are derived from the sample counter,
so all clocks land together on every master beat and never drift. A newly patched clock
starts mid-beat on that grid. Seeded random clocks (bugs) stay free. Player movement and
UI stay on real time.

**Tempos are on a 24 BPM grid for now** (user, 2026-11-17: 24, 48, ... 120, 144, ... 240):
a game rule, not the engine's. The burbler's BPM knob clicks through multiples of 24
(`step` in its module data), new burblers seed on it, and recipe BPM ranges are multiples
of 24 (`stubby_recipes::BPM_STEP`). **Recipes are made at a tempo**
(`add-campfire-craft-bench` D9): the bench (the campfire, a bundle of sticks,
`assets/structures/campfire/`) takes the tempo of the nearest running clock within
`clock_reach_m` (5 m), and a recipe with a `bpm` range is made only inside it, so the
player sets their streams to what they want to build.

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

**Designers use the rules designer** (dev builds of the game start it; or `cargo run -p
stubby_rules_server`, then http://127.0.0.1:7878/; `add-crafting-designer`): schema-driven forms over the editing
model, live problems, undo, canonical save, and a live panel reading the running game over
Bevy's remote protocol (dev builds serve it on 127.0.0.1:15702). Dev builds of the game
reload `assets/recipes/` within a second of a save.
The designer also serves the **lore guide** at http://127.0.0.1:7878/lore: the biomes, the being
atlas (filter, map), the design and research documents and the reference images under
`docs/art/`, read from the repository on every load, so it stays current as files change.

How to add or change a rule by hand (designer or agent, same steps):

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
- **Previews live in `previews/`, mirroring `assets/`** (`previews/flora/reeds/states.png`), so
  they stay out of the Jackdaw asset browser and the shipped assets. Only PNGs the game loads
  belong in `assets/`: today `assets/ui/` and the generated floor textures.
- Sources live in `assets-src/blender/<category>/`. Export to **glTF binary (`.glb`)** into
  `assets/<category>/<id>/<id>.glb`, beside its profile and master prefab. Variant
  models sit beneath their family (`assets/flora/scrub_oak/scrub_oak_0/`). Bevy
  loads `.glb`, never `.blend`. See `docs/bevy-asset-guide.md`.
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
  prefabs. Every placeable world asset has one beside its model in its asset folder.
  Each authored variant keeps its own master prefab; LODs never get prefabs. Make
  new ones in Jackdaw (place the glb, add components, **Save as Prefab**; see
  `crates/stubby_jackdaw/README.md`) and edit them there. `cargo test -p stubby_jackdaw --test prefabs`
  validates them.
- **Audit acoustics when authoring levels or large geometry.** Read
  `docs/audio/acoustic-level-authoring.md`; mark sound-critical static `Mesh3d`
  children with `stubby_core::AcousticOccluder` and choose its absorption,
  scattering, and transmission values. The marker belongs on the mesh child, not
  just the prefab root. Record representative source/listener paths and the material
  choice in the PR. Do not tag all decorative meshes or invent a second occlusion
  filter alongside AudioNimbus.
- Ship each glb in its starting pose using **node scale** (Blender's hidden-in-render
  flag does not export).
- **A new placeable kind needs three lines of game code, or it is hidden or inert:** its file
  stem in `stubby_jackdaw/src/prefab_bridge.rs` (so it becomes a `PlacedAsset`), an
  `AssetBinder` in `stubby_game/src/glue.rs` (scenery binds with a no-op; without one it
  fails as "unknown placed asset" and is hidden), and, if the game reads its node names or
  glTF extras, its kind in `model_part_bindings.rs`. Jackdaw's LOD runtime flattens a placed
  model into bare meshes, so names and extras (`start_scale`, `"acoustic": "rock"`) reach
  the game only through that binding, from the nearest ancestor node that carries them
  (2026-11-17: the grotto's boulders were never acoustic, and the campfire burned unlit).
  Empties are flattened away too: the binding recreates a model's **sockets**, childless
  empties outside its `_LOD` nodes (`Interact`, `Sparks`), as named children of the placed
  asset, and `module_kit.lod_levels` keeps sockets out of the levels (2026-11-18: the
  campfire had no `Interact`, so E configured the burbler behind it). Ring and label follow
  E's target.
- **Placed prefabs need unique instance IDs.** A prefab with a fixed `MineNode` or
  `ResourceNode` `instance` (each `scrub_oak_<k>` is `scrub_oak-<k>`) fails with "duplicate
  or empty instance ID" when placed twice, unless its placement gets its own `instance`.
  Reeds carry none: each placement takes an ID from its name and position.
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
| Engine | `bevy` | Bevy main as Jackdaw main locks it; `compatibility.json` |
| Audio I/O | `cpal` (through `firewheel-cpal`) | 0.18.2 |
| Audio engine (graph, device, scheduling) | `firewheel` | 0.14.0 |
| DSP inside modules | `fundsp` | 0.23.0 |
| RT messaging | `rtrb`, `triple_buffer` | 0.4.1 / 9.0.0 |
| RT safety | `assert_no_alloc` (debug) | 1.1.2 |
| Spatial audio | `audionimbus` over Steam Audio | =0.17.0 / 4.8.1 (`tools/steam-audio-sdk.lock`) |
| Physics | `avian3d` | 0.8.0, through our `bevy-main` fork (`[patch.crates-io]`) |
| Editor crates | Jackdaw (`jackdaw_runtime`, `_prefab`, `_terrain`, `_bsn`, `_scene_types`) | git, buddha314 fork, pinned `rev` |
| Seeded RNG | `rand_chacha` | 0.9.0 |
| WAV decoding | `hound` | ~3.5 |

Considered and **not used** (no crate depends on them; 2026-11-20): `noise` (the ground is
our own Voronoi), `bevy_hanabi` (sand particles), `bevy-tnua` / `bevy-tnua-avian3d` (the
player controller is our own, `stubby_player::controls`), `bevy_enhanced_input` (input is
Bevy's).

Check a crate's Bevy compatibility before adding it, since Bevy plugins lag Bevy
releases. Pin exact minor versions in `Cargo.toml`.

## Conventions

- Rust 2024 edition. Format changed Rust files and use the smallest relevant build or check; do not run all-target lint sweeps by default.
- **CI costs money.** Run `tools/ci_local.sh` (the GitHub workflow's steps) and the relevant `cargo test -p` before pushing; push finished work once, and merge only after CI is green.
- Tests: focused checks only during this prototype. DSP modules get offline render tests (render N
  samples, assert frequency/amplitude/phase) — they need no audio device.
- Commits to our own repositories, and merging their pull requests, are authorized as part of completing requested work (user direction, 2026-11-06); do not ask for separate permission. This covers buddha314 only. Branch off `main` for feature work. Publication remains restricted to buddha314; jbuehler23 is read-only.

### Working alongside other agents

Several agents work in this repo at once. To avoid stepping on each other:

- **Use your own git worktree** off `main` (`git worktree add ../stubby-music-<topic>
  -b <topic> origin/main`). Never edit, switch branches in, or pull into another
  session's checkout. The main checkout at `~/src/stubby-music` is usually someone's
  live workspace, and often has the Jackdaw editor open on it.
- **Claim tasks in OpenSpec:** add `(claimed: <agent>)` to the task's line in
  `tasks.md`, in the PR with your first commit for it. Check claims before starting.
- **Shared files** (`Cargo.lock`, `crates/*/src/lib.rs`, `crates/stubby_jackdaw/Cargo.toml`) conflict
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
    `tools/blender/`, `assets-src/blender/`, asset-family models/profiles, `assets/modules/`,
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
  **Claude, by the user's direct request (2026-11-01..06):** crate work on the rack and
  burbler UX, performance (sky environment map refiltered only as the sun and moon move, ground chunked for culling,
  two shadow cascades; `docs/research/performance-profiling.md`), stream laying
  (`stubby_streams`, the `buildables` rule type, `add-stream-building`), bar lines on streams
  and placed machine voices (`add-localized-machine-voices`).
  **Claude (2026-11-16):** the moving sky, `sync-calendar-and-night-sky`: Bevy's atmosphere
  with desert dust, the moon and seeded stars (`stubby_world/src/sky.rs`, `calendar.rs`);
  `tools/jackdaw_timed_test.py --sky` reviews it.
  Codex implemented `organize-model-owned-lod-assets` (2026-11-16), including
  exporter path updates and the asset-layout documentation; geometry and artistic
  settings are preserved.
  **Claude (2026-11-18):** the amphitheatre's art and scene (`add-amphitheatre` 1–3), the being
  atlas (`add-being-atlas`) and the mythology research; Codex owns its seat behaviour (4).
  **Claude (2026-11-20, by the user's request):** collecting filter sources (`add-high-pass-sources`
  2-3, `add-ripple-pool` 2-3).
  **Claude (2026-11-20, by the user's request):** seeded placement (`add-hybrid-world-placement`
  2.2, 2.4, 2b, 3.1).
  **Claude (2026-11-19):** the filter sources' art (`add-high-pass-sources` 1.x, `add-ripple-pool`
  1.x) and the purifier's check dam (`add-reed-water-purifier` 3.x).
  Update this line when you claim work.
- **Opening composition (2026-11-16):** spawn at x=0, z=6, bearing 75 degrees,
  so the eastward sunrise is 15 degrees to the right (about 20 degrees at the
  authored 06:30 opening, when the sun clears the mountain skyline). Scene placements and ridge
  rings share that yaw around spawn; keep the physical sky/cardinal frame fixed.
- **The echo grotto (2026-11-17, `add-echo-grotto`):** a cluster of rounded Mojave-style
  granite boulders (`assets/props/rocks/`, `tools/blender/create_boulders.py`, Blender's
  Rock Generator add-on, so run it without `--factory-startup`) with a small grotto at its
  base, placed in the opening scene at world (70.5, −64.5), 95 m from spawn, with a second
  burbler at its mouth. It is the first authored acoustic space: its boulders' near meshes
  are acoustic rock through glTF extras `"acoustic": "rock"`. **Rock formations**
  (`rock_formation_<k>`, `tools/blender/create_rock_formations.py`, 2026-11-19) are the same
  granite as whole clusters for seeded placement; their `Opening_<k>` sockets are the
  high-pass filter sources (`add-high-pass-sources`). Reflections, reverb and the
  listening review there belong to the audio work stream (`add-rack-and-spatial-audio`
  4.6/4.8).
- **The game opens at 06:30**, just after astronomical sunrise, so the low sun
  clears the mountain skyline. Reviews set the calendar to midday for screenshots;
  if the opening reads too dark, start the clock later rather than brightening the night.
- **LOD tuning (Codex, 2026-11-16):** model metadata is generated from
  `tools/assets/lod_policy.json` via `tools/jackdaw_lods.py --switch`; `--check`
  detects drift. Bounded `--lods flora` / `--lods machines` diagnostics and known
  Jackdaw scale/generated-geometry limitations are documented in
  `docs/research/lod-tuning-first-pass.md`.
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
  opened by TRIG while the gate is high; no envelope, so the voice stays square). The player **patches the voice with cables**: clock GATE →
  volume TRIG, VCO OUT → volume IN; volume OUT is normalled to the outlet. **A rack offers
  only the places its machine's work needs** (user, 2026-11-18): the burbler has no
  sequencer place; the campfire, which needs one, does. Patch cables
  are a general asset and **fully elastic, as in VCV Rack** (`assets/cables/`). **Every
  cable is one of the player's patch cables, from the inventory** (no cable comb;
  `simplify-rack-cables`): patching takes one, unplugging gives it back. The player starts
  with 10 (`start.json`) and later makes them from discarded bird nests. Each jack's halo
  (patch pod) **pulses** in its patch's colour while it waits for a cable; once a cable's
  first end is in one end of a known patch, only the other end shows, **blinking**. A cable
  making a known patch takes that patch's colour (others are bone), and **hovering a
  patched cable lights it end to end**. The rack screen shows a
  **Layout follows Satisfactory's craft bench** (`machine_config::layout`): a framed window
  with a title bar; on the left a **Relevant** list of the items an open slot accepts that
  the player **holds** (nothing held, nothing shown); the rack in the middle panel; the
  inventory's 16 boxes on the right (an item that fits an open slot drags from its box
  too); key prompts under the window. Each item shows its icon and count
  (`assets/ui/icons/<item_id>.png`, from `tools/icons/make_item_icons.py`; hover names the
  item), **dragged into its slot** (or clicked, or keys 1-0). Cables follow **VCV Rack**:
  drag from a jack and drop on the other, or click each. **Any cable joins any output to
  any free input**; the profile's patches are only suggestions (the ghosts and pods), and
  the voice completes when those patches exist by whichever cables. A cable starts on a
  jack; within 40 px of a jack it can enter, its far end **snaps**: the plug
  shows in the jack, and letting go plugs it in. The cable in
  hand is **shown, animated, never described**: with one end in a jack it runs to the
  pointer with VCV Rack's elastic sag, springing after each move (a damped spring); plugs
  take the cable's colour. **Sound begins only when every cable is patched**, and the screen guides it
  visually: ghost cables show the patches still to make and pods pulse in their colour.
  **Satisfactory is the baseline for hints** (user, 2026-11-20): short guidance lines are
  fine where Satisfactory would show them. What stays visual is the *state*: the cable in
  hand is drawn, never described ("this is not a text adventure", 2026-11-01).
  Plugging a cable in **snaps**: three CC0 clicks (Kenney Interface Sounds, licence in
  `assets/audio/sfx/patch_snap/`) played in turn, through the engine's one-shot
  `Command::PlaySample` (load once with `AudioThread::load_sample`; sound effects only,
  never patch modules). The
  stream's bar lines and notes start with the sound.
- **Inventory: 16 boxes, 100 of one kind per box** (`stubby_recipes::INVENTORY_BOXES`,
  `BOX_STACK`); adds that do not fit fail all or nothing. The Tab screen is the 16 boxes,
  filled in order; empty boxes stay blank, and nothing held at zero is shown. Anything
  that cannot fit sends `InventoryFull` (`bevy_stubby_recipes`) and shows Satisfactory's
  **INVENTORY FULL** banner (red, with the item's icon, under the reticle or under an
  open machine window), fading after about 2 s; nothing is taken or lost.
- **Knobs can click to a grid** (`add-reed-water-purifier`): `step` (the 24 BPM grid) or
  `quantize: "semitone"` in a module's knob data (equal-tempered notes; the square VCO's
  frequency). Such knobs turn by whole clicks, start on the grid, and the knob's readout
  beside the pointer names the note (`C4`). **VCOs start straight up** (user, 2026-11-18):
  their frequency knob's default is the middle of its log travel, snapped to C#3, not the
  bug's pitch.
- **Machines can require settings**: a machine's `targets` (knobs at a `note` or a
  `value`; the campfire: gates on 2 and 4 only) and its `clock_reach_m` are **rules**, rows
  in `assets/recipes/machines.json` (`add-machine-rules`), not profile data. The glue copies
  them into the rack's profiles on every rules reload; the rack publishes `RackTargets
  { met, missed, sounding }` (reflected); the game's rule for the machine reads it. Tune
  them in the rules designer: its **Machines** panel shows each placed machine live, and a
  save reaches the running game within a second. The log names missed targets too.
- **Dev builds start the rules designer** (`stubby_game::designer`): running the game
  starts `cargo run -p stubby_rules_server` and opens http://127.0.0.1:7878/, unless one
  answers there already or `STUBBY_NO_DESIGNER=1` (bounded runs set it). It outlives the
  game; its output is `target/rules_designer.log`.
- **Every machine gets a rack, and all racks share one implementation** (the
  `bevy_stubby_rack` crate): this is the general pattern. A machine is data, its profile's
  `configure` block: places (fixed modules, or what an open place accepts from the
  inventory), spare cables (colour, the suggested patch, a hint), the normalled output. A
  module is data too: `assets/modules/<id>/module.json` (its engine parts, internal
  wiring, jack → engine port, knob → engine parameter, default), validated against its
  panel `asset.json` by the Bevy-free `stubby_rack` crate, which also plans the engine
  patch. `stubby_rack` never names items, creatures or machines; values that join the
  rack to the world (a machine's seeded tempo, the bug's pitch) are passed in by
  the game's glue (`stubby_game/src/glue.rs`, `KnobDefaults`) (`refactor-rack-modules-as-data`). The engine patch is exactly the patched cables; completing them sends
  `MachineConfigured`. New machines add data, not systems (`add-rack-configurations`,
  design "The rack pattern as built").
- **A rack plays only what it shows** (user rule, until the user gives an override).
  Every engine module is a module the player sees in the rack, and every connection is a
  cable the player patched or an output normalled on the panel. No hidden modules (no
  built-in envelope, filter, LFO, or anything else off the panel), and tests, reviews
  and renders never add patches the player did not make (e.g. PHASE → V/OCT, which
  sweeps the pitch into a siren). The burbler's voice is therefore a plain gated square
  at the lightning bug's pitch.
- **Mud streams carry the music.** Streams of mud flow from a burbler's outlet and
  **pulse at its BPM**, so a stream is the played sequence, laid out in space. The
  water is **sheet music**: a staff ribbon runs along it, its lines a fixed distance
  apart and its wave travelling downstream; each beat's note rides it at the water's
  speed, so note spacing shows velocity, and a bar line every bar (4 beats by
  default, configurable later) keeps measures in time with the clock's BPM. The
  mud is **raw**: muddy on purpose, like ore. It is cleaned later by a **filter
  mini-rack** (direction, not yet designed): **high-pass filters separate out pure
  liquid, low-pass filters separate solids and nutrients**, into separate channels,
  controlled via audio. The stream's water goes **from muddy to clear**. **High-pass
  filters come from the world** (user, 2026-11-17): openings in rock formations and
  hollow tree trunks, which pass short waves and stop long ones, raising the pitch, like
  sound through a narrow duct. They are found like other waveform sources (the grotto's
  boulders are the first rock formation), not invented as machines. **Low-pass and
  band-pass filters matter more** (they separate the solids and nutrients) but their
  sources are **not decided yet**: do not invent them. Both need an **oscilloscope**
  asset to see what they do, from **ripple pools** that form at seeded Poisson-random places
  along laid streams and ripple on the beat (`add-ripple-pool`).
  Each rock opening and each hollow trunk (only some oaks are hollow) gives **one**
  high-pass filter; filters and oscilloscopes return semi-daily (`add-high-pass-sources`,
  user, 2026-11-19). **Water has
  four levels: muddy, silty, cloudy, clear**; the first filter, the reed water purifier,
  is a small check dam across a laid stream and lifts the water below it one level, so
  dams in sequence reach clear. **Collection is generic** (user, 2026-11-20): a leaf socket of
  any placed model whose glTF extras name a `resource` from the rules is a pickup of it
  (`stubby_game::collectibles`); a new collectible is art and a rules row, never code.
  Things that form along laid streams are data too (`stream_features` in
  `assets/world/opening.json`), and a `*_Beat` clip plays on the placement's clock.
  **Building is generic too** (2026-11-20): a `buildables.json` row names its `model`,
  `placement` (`ground`, `beside_stream`, `across_segment`, `laid`) and an optional
  `built_hint`; `stubby_game::build` places them all. A thing built across a stream
  segment keeps it (`OnSegment`) and goes with it, refunded. **Crafting**: E at a burning campfire opens
  the crafting screen (Satisfactory's Craft Bench, `stubby_game::crafting_screen`): any
  machine carrying `stubby_core::CraftBench` is a bench; its RACK tab opens the rack, and
  each `CraftRequest` carries the tempo the bench hears
  (`add-reed-water-purifier` D4, D6). Until it exists every stream stays mud
  (`LaidStream.clarity` 0).
- **Reeds are the stream's ingredient**: **6 reeds build one stream segment**, and reeds
  outline every laid segment's banks. They are a frequent renewable, as easy to get as
  leaves in Satisfactory: abundant on wet ground, harvested with E (hold E for nearby
  clumps), cut to stubble, and back after a short interval (`openspec/changes/add-reeds/`).
- **Port highlighting.** Every stream port (the burbler's mouth, each end of a stream
  segment) carries a halo in the **stream's cross-section** (bed and both banks). It
  appears **only while the player is attempting a connection** there, as a generator's
  port does when you drag a conveyor from it in Satisfactory; otherwise it is hidden
  (shipped at node scale 0; shown while `AssetState::port_highlight`).
- **Stream hologram.** While extending, the planned stream shows as a holographic
  preview, like a Satisfactory conveyor before it is built. That is its own asset,
  separate from the burbler.
- **Laying streams works (2026-11-03, `stubby_streams`, `add-stream-building`).** Q build
  mode, a cubic Bézier from the port's flow direction to the aim, whole 4 m segments
  bent from the glTF meshes, scrolling hologram chevrons, port halos, reed `need / have`
  under the reticle; cost from `buildables.json`. Bar lines ride a sounding burbler's
  streams, from the audio clock. Its voice is heard from the nearest point of the
  burbler or its streams, within `hearing_radius_metres`
  (`add-localized-machine-voices`). **F dismantles** the aimed segment
  (`stubby_streams::dismantle`, 2026-11-17): pressing F turns the segment in the crosshair
  red, holding F (0.6 s, a bar under the reticle) removes it; a tap, Escape or Q leaves
  (user, 2026-11-18). A stream aimed within 2.5 m of another **joins it**: it arrives along
  that stream's flow (head on at a free end) and laying ends there (`snap_to_stream`).
  Dismantling refunds its reeds all or nothing and splits
  the stream; the part cut off downstream, and anything laid on from it, lies quiet.
  Notes ride sounding streams. Still to do: the staff's flow (2.7), bubbles (2.6), saving
  laid streams, collision checks.
  `tools/jackdaw_timed_test.py --streams` runs the script (`--stream-review=<dir>`)
  under the watchdog. **Streams water trees** (`stubby_game::watering`, 2026-11-17): a
  flowing stream within `water_reach_m` (3 m) of a scrub oak waters it (aimed at an oak, a stream
  being laid snaps to its inlet ring and runs alongside the trunk; `stubby_core::StreamInlet`,
  `add-tree-inlets`), and a watered tree shows a small trickle over the ground from the stream to
  its trunk while the water flows (`watering::show_trickles`), and growth rises
  toward the cap its water's clarity allows (`mines.json` `growth`: mud reaches budding,
  silty leafed, clear full bloom) and eases back without water, in simulation time.
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

### Jackdaw is the version authority

**The repo root is the Jackdaw project**, the layout Jackdaw scaffolds: `jackdaw.toml` and
`.jackdaw/` at the root, one `assets/` as Bevy expects, and `package` naming the workspace
member Jackdaw builds. The project takes its name from the folder. In the launcher, add
the checkout (`~/src/stubby-music`), not a subfolder.

The standalone game and editor adapter share the root Cargo workspace, its one
Cargo.lock, the workspace Bevy dependency and rust-toolchain.toml. The adapter depends on
`stubby_game` like any crate; saved prefabs name a component by the crate that owns it
(`stubby_game::resources::MineNode`), and a change that moves a reflected type rewrites
the `.bsn` files that name it (`split-game-into-systems`). Do not independently upgrade Bevy
or restore a separate adapter lockfile. `compatibility.json` records the
shared editor source revision, upstream base, Bevy revision and compiler.

**The game takes Jackdaw's crates as git dependencies on the fork, pinned by `rev`** in the
root `[workspace.dependencies]` (`use-jackdaw-git-dependencies`), so the repository builds
from a fresh clone. Members inherit them (`jackdaw_prefab.workspace = true`); never add a
Jackdaw `path` dependency. To build against an unpushed editor fix in `~/src/jackdaw`, run
`tools/jackdaw.sh link` (an untracked `.cargo/config.toml` `[patch]`) and `tools/jackdaw.sh
unlink` before committing; the check refuses a linked `Cargo.lock`. The shared editor's
checkout must still equal the pin to launch, because Play talks to the game over Jackdaw's
PIE protocol.

Run `python3 tools/check_jackdaw_versions.py` before work involving the editor (it checks
the repository; `--editor` also checks the shared checkout, as `tools/jackdaw.sh` does).

**We follow the chain Bevy main → jbuehler23/jackdaw main → our fork's main → this game**
(user direction, 2026-11-03: use the latest Jackdaw features as they are published).
**`tools/jackdaw.sh sync` reconciles the whole chain** (`tools/jackdaw_sync.py`): it merges
upstream main into the fork (temporary worktree, `sync/<date>` PR on buddha314; stops on a
conflict), fast-forwards the shared checkout, rebuilds our Bevy-main forks of third-party
crates, re-pins `compatibility.json`, `jackdaw.toml` pins, `rust-toolchain.toml`, the
Jackdaw `rev` in `Cargo.toml` and Cargo.lock's Jackdaw and Bevy revisions, then builds the
game and adapter and the shared editor (`prepare`). Commit the pins, `Cargo.toml` and lock
on a branch. If the game stops building on a new Bevy, fix the game (or a fork) and rerun.
CI warns when the pin lags the fork.

**Physics colliders on prefabs are planned in `add-prefab-colliders`:** Avian already runs on
our fork, and `jackdaw_avian_integration` compiles once a one-line Jackdaw workspace bug is
fixed. Meanwhile **boulders are solid** (2026-11-18): each gets static trimesh colliders from
its own LOD0 meshes (`model_part_bindings::solid_boulders`), marked `stubby_core::Solid`, and
the player's movement slides along `Solid` colliders (`controls::slide`); nothing else blocks yet.

**Crates without a Bevy-main release are generated forks** (`bevy_main_forks.json`,
`tools/bevy_main_forks.py`): avian3d and its bevy_heavy, bevy_transform_interpolation and
glam_matrix_extras, each a `bevy-main` branch on buddha314 = the published release plus one
commit pointing its Bevy dependencies at Bevy's git main. The root `[patch.crates-io]` uses
them. Add a crate there (not by hand) when a new dependency needs Bevy main.

`tools/jackdaw.sh prepare` builds the selected shared editor/CLI and records local
binary hashes; managed launches verify that receipt. This does not install a
per-game editor. Our editor fixes go on fork main (`fix/<issue>` branches); `dev/0.20`
is retired and kept only for rollback. Never force-sync/reset the fork to discard fixes.
Promote only while the shared editor is idle and its checkout clean. Follow
`docs/research/jackdaw-version-policy.md` for promotion and rollback.

### Prototype testing: keep it minimal

**User direction (2026-11-01): keep testing minimal; we are prototyping. No testing busywork.**

- Use the smallest relevant check that gives useful confidence in the change. A focused build, one regression check, or one brief smoke test is usually enough.
- Do not run blanket workspace tests, all-target lint sweeps, full upstream fast/integration suites, documentation builds or doctests by default, including for upstream syncs. Broaden only when the user asks or a concrete failure directly requires it.
- Do not install extra testing infrastructure just to run broad suites. Do not repeat or expand passing checks speculatively.
- **Test our code, not third-party packages.** Do not test what fundsp, Firewheel, Bevy, avian, Jackdaw or serde already promise (a sine's purity, a sampler playing, a graph fanning out, Bevy's virtual time pausing); test the wiring and the logic we own. If an area of our own code needs a large suite, look for an established crate that does the job before adding more tests (the rack's WAV decoding is `hound`, not a hand-written parser).
- **Spend build cycles carefully.** Work sequentially in one worktree (switch branches) rather than a fresh worktree per step, which cold-compiles Bevy each time; run `cargo test -p` only for the crates you changed, and `tools/ci_local.sh` once before pushing.
- Once the relevant check passes, continue the actual work. Prefer iteration speed over exhaustive validation during this prototype.
- When a graphical check is relevant, retain the existing independent 20-second watchdog.
- This policy supersedes earlier blanket instructions to run full workspace tests and all-target Clippy before every commit. Upstream release-check lists are not our default prototype workflow.

## Hard rule: jbuehler23 is read-only

Never push to jbuehler23, whether through a named remote, a direct URL, GitHub CLI,
an API or any other route. Never create issues or pull requests there, and never
publish code changes to that account (user direction, 2026-11-06; this replaces the
earlier allowance for filing bug reports as issues). Fetching and reading are allowed.
All pushes and PRs go only to buddha314. This rule overrides any older instruction to
offer fixes or reports upstream. The upstream remote's push URL is deliberately
disabled; do not restore or bypass it.
