# stubby-music

A modular synthesis factory game: the music of nature brings light to the world. It
opens on a silent, seeded desert mudflat (`NARRATIVE.md`, `AGENTS.md`).

Launch the game with AudioNimbus spatial audio. The first launch builds the pinned Steam
Audio SDK; later launches only check whether its inputs changed:

```sh
tools/run_game.sh --seed=42
```

This runs `stubby_jackdaw`, the game as Jackdaw builds it: the authored opening scene
(`assets/scene.bsn`) plus the layout generated from the seed (burblers, reed beds, oak
groves, rock formations and props, `assets/world/opening.json`). `tools/run_game.sh --bare`
runs `stubby_game` alone: the terrain, sky and player with no scene, for art studies.
For faster development linking, set `STUBBY_DEV_LINKING=1` before the launcher; omit
that setting for release builds.

WASD walks, the mouse looks freely, Space jumps, C crouches, and hold Left Shift to
sprint. E collects and opens machines, Q builds, H brings up the field recorder, B the
scanner, Tab the inventory, O the recipes. Escape releases the cursor; click to resume.

The floor uses clipped Voronoi polygons with raised rims, recessed sidewalls, subtle
height variation, and generated fine-crack albedo and normal textures; the cracked ground
is about 216 m wide. The sky is Bevy's atmosphere with desert dust, a moon and seeded
stars. AudioNimbus binaural spatial rendering and direct propagation are on by default;
use `--no-default-features` for a temporary build without them.

From the repository root, launch the game with:

```sh
cargo run
```

The workspace defaults to the `stubby_jackdaw` game package. To enable development
features, use `cargo run --features dev`.

It opens at the main menu. To play together (add-multiplayer-and-menus):

```sh
cargo run -- --host                    # host over UDP (port 5888); or Host a Game in the menu
cargo run -- --join 192.168.1.20       # join that host; or type its address in the menu
cargo run -- --steam-host              # host a Steam lobby; the invite dialog opens
cargo run -- --steam                   # play, and join when you accept a friend's invite
cargo run -- --no-steam                # the menu without touching Steam (for checks)
```

A plain launch starts Steam if it is running (as Valve's test app 480, Spacewar, until
stubby has its own app), so the menu's Host on Steam works and invites join.

To save a preview after shader warmup, append
`--screenshot=/tmp/stubby-desert-preview.png` to the game arguments.

Open the level in the shared Jackdaw editor with `tools/jackdaw.sh open`.
First-time setup and Play instructions are in [crates/stubby_jackdaw/README.md](crates/stubby_jackdaw/README.md).

The audio engine is `crates/stubby_audio`; it has no Bevy dependency. One shared
audio thread drives the lightning bugs’ clocks and short click patches. The opening
is silent until a bug is nearby. A patched burbler's voice pans from the nearest point
of the burbler or its streams and fades out at its hearing radius. For a test patch, add `--audio-demo` or press F8.

The default game run uses AudioNimbus/Steam Audio HRTF and direct propagation. From a
terminal at the repository root, launch the game without the sustained audio demo tone:

```sh
tools/build_steam_audio_sdk.sh      # once per checkout, and after the SDK pin changes
cargo run -p stubby_jackdaw --features dev
```

The SDK helper builds Steam Audio from pinned source into this checkout's
`target/native/steam-audio/` and rebuilds only when its pinned inputs change; Cargo never
builds it. The game's build scripts find that cache and record it in the binaries' run
path, so no `STEAMAUDIO_LIB_DIR` or `LD_LIBRARY_PATH` is needed (setting
`STEAMAUDIO_LIB_DIR` still overrides it). `STEAMAUDIO_CACHE_DIR` moves the cache, so
worktrees can share one build (e.g. `~/.cache/stubby-steam-audio`); a symlink at
`target/native/steam-audio` does the same without the variable, which the editor's build
needs. Until the SDK is found, Cargo warns and the link fails (`unable to find library
-lphonon`); after providing it, `touch tools/steam-audio-sdk.lock` so cached build scripts
look again. Jackdaw's editor builds the game into `.jackdaw/target/game`. Add `-- --audio-demo` to the Cargo command only
when you want a sustained 110 Hz square-wave test tone alongside clock blips. The direct path
supports distance, air absorption, geometry occlusion and material transmission.
Reflections/reverb, authored cave or building validation, live device-restart handling,
and multi-emitter budgets remain in progress. Details and evidence:
[`docs/audio/pre-level-validation.md`](docs/audio/pre-level-validation.md).

To hear the engine alone on the default output device:

```sh
cargo run --release -p stubby_audio --example play
```

The first creature asset is the [lightning bug](assets/creatures/lightning_bug/README.md).
Each bug in the opening owns a seeded random audio clock and emits
white flashes independently, remaining invisible and silent between pulses. Nearby
clicks use the existing square VCO, envelope and VCA; spawn is outside hearing
range. An audio output device is required for the live clock.

Lightning bugs are also resource nodes: hold up the field recorder (H), come within
3 metres and press E to record a square wave. Rock openings and hollow trunks yield
high-pass filters and ripple pools along laid streams yield oscilloscopes; any model
socket tagged with a `resource` is collectable (`stubby_game/src/collectibles.rs`). The
[recipe foundation](crates/stubby_recipes/README.md) provides validated data definitions,
inventory, resource yields and atomic crafting. Press O for the recipe book.

Q builds: streams, a campfire, and a reed water purifier across a laid stream. E at a
burning campfire opens the crafting screen (Satisfactory's Craft Bench), where recipes
are made at the tempo the fire hears; the first is the reed water purifier.

The opening's layout comes from the seed (`assets/world/opening.json`, see
[assets/world/README.md](assets/world/README.md)). `--no-seeded-layout` leaves only the
authored scene; `--world-config=<file>` runs another layout; `--layout-review=<dir>`
saves an overhead and a reed-bed screenshot.

Version selection follows the shared Jackdaw checkout, recorded in
[`compatibility.json`](compatibility.json). Both game entry points
share one Cargo workspace and lockfile. Run `tools/jackdaw.sh check` before editor
work; see [the version and upstream-sync policy](docs/research/jackdaw-version-policy.md).

## Adding and changing game rules (designers)

Game rules (items, resources, mines, recipes, buildables, goals, machines, the start
and air cleaning) are JSON rows in `assets/recipes/`, one file per kind. There is no `catalog.json` any more; it was split into these files.

**The rules designer** edits them in a browser, with forms, ID pickers, live problems,
undo and save, and a panel showing values from the running game:

```bash
cargo run -p stubby_rules_server    # then open http://127.0.0.1:7878/
```

A dev build of the game picks up a save within a second. Link to a row with
`http://127.0.0.1:7878/#<file>/<id>`, e.g. `http://127.0.0.1:7878/#air/scrub_oak_air`.
Details: [assets/recipes/README.md](assets/recipes/README.md#the-rules-designer).

The same page includes a dev-only Spatial Audio panel for live per-emitter reflection
wet-gain tuning. Run the Rules Designer in one terminal and the dev game in another;
a dev build of the game starts the designer itself (unless one is running) and serves Bevy
Remote:

```sh
# Terminal 1
cargo run -p stubby_rules_server

# Terminal 2
cargo run -p stubby_jackdaw --features dev
```

Open `http://127.0.0.1:7878/` and use Spatial Audio → Reflection wet gain. A value of
0.0 is a dry comparison; 0.75 is the current default. The control is session-only and
does not tune the listener-centric reverb bus, which is not routed yet.

**The crafting system is our own, not a community crate.** `crates/stubby_recipes`
holds the rules model, inventory, recipe book and crafting, with no Bevy dependency.
It is used as a Bevy community plugin: the thin `bevy_stubby_recipes` crate, which the
game adds with `add_plugins`, over that Bevy-free core, the structure long-lived Bevy
plugins such as `bevy_rapier3d` use (`openspec/changes/add-recipes-bevy-plugin/`).
We looked at the Bevy crafting and inventory crates (`bevy_inventory`, `bevy_items`,
`bevy-slotted` and others, checked 2026-11-01): they are small, behind our Bevy
version, and built for RPG inventories rather than a factory builder's catalog, so
none was adopted.

For comparison, **no game engine ships a crafting system.** Unity, Unreal and Godot
have none built in. Crafting there comes from third-party plugins written for one
engine (Opsive's Ultimate Inventory System for Unity, Arc Inventory or Easy Survival
RPG for Unreal, the expressobits Inventory System for Godot), built on that engine's
own data types, so they cannot be reused in another engine. We found no mature
crafting library that works independently of an engine, in any language (research,
2026-11-01). Community crates are used around ours instead: `serde` and `schemars`
for the files and their schemas, and, as planned, `bevy_common_assets` for hot
reload and `moonshine-save` for saves. See
[docs/research/crafting-and-inventory-crates.md](docs/research/crafting-and-inventory-crates.md)
and [docs/research/satisfactory-goals-and-inventory.md](docs/research/satisfactory-goals-and-inventory.md).

| To add or change | Edit |
| --- | --- |
| Something that can be held in the inventory | `assets/recipes/items.json` |
| A pickup in the world and what it yields | `assets/recipes/resources.json` |
| An infinite source that produces pickups | `assets/recipes/mines.json` |
| A recipe (inputs → outputs) | `assets/recipes/recipes.json` |
| What something built in the world costs | `assets/recipes/buildables.json` |
| A goal or chore the hub shows the player | `assets/recipes/goals.json` |
| A machine's tuning targets and clock reach | `assets/recipes/machines.json` |
| What the player starts with | `assets/recipes/start.json` |
| How much a plant cleans the air | `assets/recipes/air.json` |

1. Edit the file. In VS Code, field names autocomplete and mistakes are underlined,
   because each file names its schema.
2. Run `cargo run -p stubby_recipes --bin rules-fmt`. It tidies the file, records new
   IDs and lists every problem, e.g. `resources.json [reed_clump] .yeilds: unknown field`.
3. Run `cargo test -p stubby_recipes`, then open a PR and tick **Data** in the template.

For example, the first recipe, made at a campfire whose clock is between 96 and 144 BPM:

```json
{
  "id": "craft_reed_water_purifier",
  "name": "Reed water purifier",
  "inputs": [
    { "item": "reed", "quantity": 20 },
    { "item": "high_pass_filter", "quantity": 1 },
    { "item": "oscilloscope", "quantity": 1 }
  ],
  "outputs": [{ "item": "reed_water_purifier_kit", "quantity": 1 }],
  "bpm": { "min": 96, "max": 144 }
}
```

IDs are lowercase `snake_case`, unique across all the files (so the recipe above is
not called `reed_water_purifier`, which is the buildable machine) and never reused; the `name` is what
players see and can change any time. **Renaming or deleting an ID** needs a line in
`assets/recipes/migrations.json` so old saves still load; the formatter and tests say
when. If a rule needs something the files cannot express (a condition, a formula, a
new field), that is not a data edit: ask for a new rule type, which starts as an
OpenSpec change. The full guide is [assets/recipes/README.md](assets/recipes/README.md).
