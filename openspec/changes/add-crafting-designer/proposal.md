## Why

Game rules (items, resources, mines, recipes, buildables, goals, the start) are
designed today by hand-editing JSON under `assets/recipes/` and running `rules-fmt`.
That is fine for an agent, but it ties design to a file format: a designer has to know
the schema, type IDs exactly, keep `ids.lock` and `migrations.json` in mind, and read
errors from a terminal. `recipes.json` is still empty, so recipe design has barely
started. The right time to give it a real tool is now, before it does.

## What Changes

- A **crafting designer**: a Jackdaw editor extension, built in this repository, that
  shows the rules as items, recipes and goals instead of files. It has a browser of every
  registry, a form for the selected row with pickers for IDs, a live problems list, a
  where-used view, and a recipe graph of items flowing through recipes.
- A Bevy-free **editing model** in `stubby_recipes` (`rules::edit`): typed edits with
  undo, renames that write their own migration row, live validation, and saving through
  the same canonical writer and `ids.lock` update as `rules-fmt`. Both the designer and
  any later front end are thin over it.
- **Forms come from the generated JSON Schema**, with ID fields annotated by the
  registry they reference, so a new rule type shows up in the designer without
  designer code.
- **JSON stays the storage format**, as a save format the designer never asks anyone to
  open: diffs stay reviewable, the validator and `cargo test -p stubby_recipes` stay
  the gate, and a Data PR made with the tool is identical to one made by hand.
- **Live playtesting:** a dev-build Play reloads rules from `assets/recipes/` on save,
  so a change in the designer is in the running game within a second.
- An **in-game designer** over the same editing model is a later step, not part of this
  change.

## Capabilities

### New Capabilities
- `crafting-designer`: designing game rules in Jackdaw through forms, pickers, a
  problems list and a recipe graph, saved as the canonical rules files.
- `rules-editing`: the Bevy-free editing model: typed edits, undo, renames with
  migrations, live validation and canonical saving.

### Modified Capabilities
None in `openspec/specs/` (it is empty). `add-rules-data-conventions` is unchanged:
the designer follows its files, IDs, `ids.lock` and migrations rules exactly.

## Impact

- `crates/stubby_recipes`: new `rules::edit` module; schema annotations on ID fields
  (a Rule PR in the AGENTS.md sense: schema code plus regenerated `schema/`). Still
  no Bevy.
- New crate `crates/stubby_crafting_designer` (`cdylib` + `rlib`): depends on
  `stubby_recipes`, Bevy and the Jackdaw extension crates only, never on `stubby_game`.
  It is a tool, outside the game's layers (`split-game-into-systems`).
- `bevy_stubby_recipes`: dev-build hot reload of the rules from disk; release builds
  keep the embedded rules.
- No change to the Jackdaw editor itself (editor policy: extension points only). A
  blocking editor bug is fixed in the buddha314 fork.
- `assets/recipes/README.md` and AGENTS.md: the designer is the normal way to author
  rules; hand edits still work.
