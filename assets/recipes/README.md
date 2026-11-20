# Rules: items, resources, mines, recipes, buildables, goals, machines

The game's rules are rows in these files. **The designer owns the rows**; the schema
and validator are code in `crates/stubby_recipes` (`rules.rs`), and new kinds of rule
go through OpenSpec. See AGENTS.md, "Rules are data; the schema is code", and
`openspec/changes/add-rules-data-conventions/`.

| File | Holds |
| --- | --- |
| `items.json` | Everything that can sit in an inventory: `square_wave`, `reed`, `clock`, `patch_cable`, `snail_shell`, `high_pass_filter`, `oscilloscope`, `reed_water_purifier_kit` |
| `resources.json` | Finite pickups, what they yield and when they return: `lightning_bug`, `reed_clump`, `clock_source`, `empty_snail_shell`, `rock_opening`, `hollow_trunk`, `ripple_pool` |
| `mines.json` | Infinite sources and the resource they produce: `scrub_oak`, `mudpit_burbler` |
| `recipes.json` | Inputs → outputs, optionally only at a bench tempo (`bpm: {min, max}`, on the 24 BPM grid): `craft_reed_water_purifier` (20 `reed`, a `high_pass_filter`, an `oscilloscope`, 96 to 144 BPM → a `reed_water_purifier_kit`) |
| `buildables.json` | What the player builds in the world, its item cost per one built, its `model`, its `placement` (`ground`, `beside_stream`, `across_segment`, `laid`) and an optional `built_hint`: `campfire` (25 `reed`), `stream_segment` (6 `reed`), `reed_water_purifier` (a kit) |
| `goals.json` | What the hub tells the player to do next (format 2). A `major` goal is a set of `minor` steps (`part_of` it), done in file order; it completes with its last step and has no cost of its own. A step is met by a `condition` (`machine_configured`, `tree_watered`, `tree_grown` with a bloom `level`, `item_held` with a `quantity`, `built` with a `buildable`, `fire_lit`) or by `requires` (items delivered). Majors open in file order, or after their `after` |
| `start.json` | The starter inventory a new player gets: 10 patch cables and 25 reeds |
| `machines.json` | Machine settings tuned in play: each machine kind's `targets` (knobs its work needs, at a `note` such as `C2` or a `value` such as `1`), `clock_reach_m` (how far a machine with a borrowed clock reaches) and, for a machine that cleans water, `clarity_lift_s` (seconds to lift the water below it one level, and to let it fall back). `machine` is the placed asset ID (`campfire`; `reed_water_purifier`, whose targets are its low VCO at C2, high VCO at C4 and filter cutoff at C5) |
| `air.json` | Foliage that cleans the air: which placed assets, how far (`radius_m`), how strongly (`strength`), and what their cleaning follows (`growth`, `water_or_light`, `light`) |
| `migrations.json` | Every renamed or removed ID, so saves survive the change |
| `ids.lock` | Every ID ever shipped (maintained by the formatter; never delete lines) |
| `schema/` | JSON Schemas generated from the Rust types (do not edit) |

## The rules designer

The easiest way to change the rules is the designer, a page in your browser. **A dev
build of the game starts it and opens it for you** (unless one is already running, or
`STUBBY_NO_DESIGNER=1` is set; its output goes to `target/rules_designer.log`). To run it
alone:

```bash
cargo run -p stubby_rules_server        # then open http://127.0.0.1:7878/
```

Pick a file and a row, and edit it in the form. ID fields offer only IDs that exist,
problems appear as you type (click one to jump to it), **Used by** shows where an ID is
used, and Undo, Redo and Save work as usual (Ctrl+Z, Ctrl+Shift+Z, Ctrl+S). Saving is
refused while there are problems. A running dev build of the game (standalone or Play in
Jackdaw) picks up a save within a second, and the **Live game** panel shows values from
it, such as how clean the basin's air is. The **Machines** panel lists every placed
machine with a rack: whether it sounds, and whether its targets are met, naming each
missed one and where its knob is (`seq.gate_4 at 0`). Tune a machine by editing its row
in `machines`, saving, and watching the panel. Link to a row with `#registry/id`, e.g.
`http://127.0.0.1:7878/#air/scrub_oak_air`.

Files saved by the designer are exactly what the formatter writes, so hand edits and the
designer can be mixed; the designer picks up hand edits while it has nothing unsaved.

## Changing the rules by hand

1. Edit the JSON in an editor that reads JSON Schema (VS Code does by default: each file
   names its schema in `$schema`, so fields autocomplete and typos are underlined).
2. Run the formatter, which sorts rows, regenerates the schemas, records new IDs and
   validates everything:

   ```bash
   cargo run -p stubby_recipes --bin rules-fmt
   ```

   It prints every problem at once, each as `file [id] .field: message`.
   `-- --check` changes nothing and fails if anything would change.
3. `cargo test -p stubby_recipes` must pass. It loads these exact files and fails if
   any file is not in the formatter's form or a schema is stale.
4. Open a **data PR** (the PR template asks). No code or OpenSpec change is needed for
   a new row or value of a kind that already exists.

If a row seems to need a condition, a formula or a field that does not exist, that is a
new rule type: write an OpenSpec change instead of bending the data.

## Rules the validator enforces

- IDs are `snake_case`: a lowercase letter, then `a-z`, `0-9` or `_`. Each ID is used
  once across all files. `name` is the display name and may change freely.
- Unknown fields are errors (a misspelled `yeilds` fails instead of being ignored).
- Every referenced item, resource and mine exists; quantities are positive; item lists
  are not empty; every recipe's inputs can be obtained from some resource or recipe.
- **IDs are part of the save format.** To rename or remove one, add a row to
  `migrations.json` (`registry`, `from`, `to`, or `to: null` to remove). An ID that
  disappears without one fails the `ids.lock` check.
- Every file carries a `format` (1, except `goals.json` at 2). Only a schema change that old
  files cannot satisfy bumps it.

## What the current rows mean

Lightning bugs are exhaustible: each yields two `square_wave` items and returns at the
next night. Reed clumps yield six reeds and return after 60 seconds. The `scrub_oak`
mine produces finite lightning bugs, scaled later by water and light quality
(`openspec/changes/add-scrub-oak-production/`). The `mudpit_burbler` mine produces the
`clock_source` resource (renamed from `clock`, which is the item's ID), which yields the
`clock` item. Square wave is in development: keep its stable ID and metadata while its
asset is built. Collection feedback plays once per waveform, not per yielded item.

Rock openings and hollow trunks each yield a `high_pass_filter`, and ripple pools an
`oscilloscope`, all returning after 1800 s (half a game day). **A resource's pickups are
its model's sockets**: any leaf node whose glTF extras name the resource ID (`"resource":
"rock_opening"`) is collectable, one source per socket (`stubby_game/src/collectibles.rs`),
so adding a collectible is a tagged socket and a row, no code. Hollow scrub oaks
(variants 6 and 7) tag their `Hollow` socket `hollow_trunk`. The purifier is crafted at a
campfire as a kit and built from it across a laid stream.
