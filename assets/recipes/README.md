# Rules: items, resources, mines, recipes

The game's rules are rows in these files. **The designer owns the rows**; the schema
and validator are code in `crates/stubby_recipes` (`rules.rs`), and new kinds of rule
go through OpenSpec. See AGENTS.md, "Rules are data; the schema is code", and
`openspec/changes/add-rules-data-conventions/`.

| File | Holds |
| --- | --- |
| `items.json` | Everything that can sit in an inventory: `square_wave`, `reed`, `clock` |
| `resources.json` | Finite pickups and what they yield: `lightning_bug`, `reed_clump`, `clock_source` |
| `mines.json` | Infinite sources and the resource they produce: `scrub_oak`, `mudpit_burbler` |
| `recipes.json` | Inputs → outputs. Empty: no production recipe is designed yet |
| `migrations.json` | Every renamed or removed ID, so saves survive the change |
| `ids.lock` | Every ID ever shipped (maintained by the formatter; never delete lines) |
| `schema/` | JSON Schemas generated from the Rust types (do not edit) |

## Changing the rules

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
- Every file carries `"format": 1`. Only a schema change that old files cannot satisfy
  bumps it.

## What the current rows mean

Lightning bugs are exhaustible: each yields two `square_wave` items and returns at the
next night. Reed clumps yield six reeds and return after 60 seconds. The `scrub_oak`
mine produces finite lightning bugs, scaled later by water and light quality
(`openspec/changes/add-scrub-oak-production/`). The `mudpit_burbler` mine produces the
`clock_source` resource (renamed from `clock`, which is the item's ID), which yields the
`clock` item. Square wave is in development: keep its stable ID and metadata while its
asset is built. Collection feedback plays once per waveform, not per yielded item.
