# stubby-music

A modular synthesis factory game. The first scene is a silent, seeded desert mudflat.

Run the visual scene:

```sh
cargo run -p stubby_game -- --seed=42
```

WASD walks, the mouse looks freely, Space jumps, C crouches,
and hold Left Shift to sprint. Escape releases the cursor; click to resume.
The player starts standing on the terrain at eye height. Use another seed to
regenerate the mud plates, fine crack texture, and mountain silhouettes.
For faster development linking use `--features dev`; omit it in release builds.

The floor uses clipped Voronoi polygons with raised rims, recessed sidewalls,
subtle height variation, and a generated repeating fine-crack/grain albedo and normal textures on a
rough PBR material. No reference photograph or external textures are required.
The HDR sky is generated as a linear half-float cubemap with an amber low sun,
dusty horizon and slate zenith; the cubemap also supplies filtered environment lighting, alongside matching sunlight.

This is the initial art-study scene: a fixed terrain area roughly 216 m wide,
grounded first-person movement, haze and distant mountains. Streaming, sandstorm particles,
mood transitions, discovery objects and spatial audio are tracked in the opening proposal
and are not implemented yet.

To save a preview after shader warmup, append
`--screenshot=/tmp/stubby-desert-preview.png` to the game arguments.

Open the level in the shared Jackdaw editor with `tools/jackdaw.sh open`.
First-time setup and Play instructions are in [jackdaw/README.md](jackdaw/README.md).

The audio engine is `crates/stubby_audio`; it has no Bevy dependency. One shared
audio thread drives the lightning bugs’ clocks and short click patches. The opening
is silent until a bug is nearby. For a test patch, add `--audio-demo` or press F8.
To hear the engine alone on the default output device:

```sh
cargo run --release -p stubby_audio --example play
```

The first creature asset is the [lightning bug](assets/creatures/lightning_bug/README.md).
Each bug in the opening owns a seeded random audio clock and emits
white flashes independently, remaining invisible and silent between pulses. Nearby
clicks use the existing square VCO, envelope and VCA; spawn is outside hearing
range. An audio output device is required for the live clock.

Lightning bugs are also resource nodes: approach within 3 metres and press E to
recover a square-wave recipe ingredient. The [recipe foundation](crates/stubby_recipes/README.md)
provides validated data definitions, inventory, one-time resource yields and
atomic crafting. Press O for the recipe book: Square wave is a raw building block
for VCO, LFO and gate uses. Other waveform resources and production combinations
will be authored later.

Version selection follows the shared Jackdaw checkout, recorded in
[`jackdaw/compatibility.json`](jackdaw/compatibility.json). Both game entry points
share one Cargo workspace and lockfile. Run `tools/jackdaw.sh check` before editor
work; see [the version and upstream-sync policy](docs/research/jackdaw-version-policy.md).

## Adding and changing game rules (designers)

Items, resources, mines and recipes are JSON rows in `assets/recipes/`, one file per
kind. There is no `catalog.json` any more; it was split into these files.

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

1. Edit the file. In VS Code, field names autocomplete and mistakes are underlined,
   because each file names its schema.
2. Run `cargo run -p stubby_recipes --bin rules-fmt`. It tidies the file, records new
   IDs and lists every problem, e.g. `resources.json [reed_clump] .yeilds: unknown field`.
3. Run `cargo test -p stubby_recipes`, then open a PR and tick **Data** in the template.

For example, a recipe that turns 6 reeds into one stream segment (once the
`stream_segment` item exists in `items.json`):

```json
{
  "id": "stream_segment_from_reeds",
  "name": "Stream segment",
  "inputs": [{ "item": "reed", "quantity": 6 }],
  "outputs": [{ "item": "stream_segment", "quantity": 1 }]
}
```

IDs are lowercase `snake_case`, unique across all the files (so the recipe above is
not called `stream_segment`, which is the item) and never reused; the `name` is what
players see and can change any time. **Renaming or deleting an ID** needs a line in
`assets/recipes/migrations.json` so old saves still load; the formatter and tests say
when. If a rule needs something the files cannot express (a condition, a formula, a
new field), that is not a data edit: ask for a new rule type, which starts as an
OpenSpec change. The full guide is [assets/recipes/README.md](assets/recipes/README.md).
