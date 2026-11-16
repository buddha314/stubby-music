## Why

Game rules (items, resources, mines, recipes, buildables, goals, the start) are
designed today by hand-editing JSON under `assets/recipes/` and running `rules-fmt`.
That is fine for an agent, but it ties design to a file format: a designer has to know
the schema, type IDs exactly, keep `ids.lock` and `migrations.json` in mind, and read
errors from a terminal. `recipes.json` is still empty, so recipe design has barely
started. The right time to give it a real tool is now, before it does.

## What Changes

- A **crafting designer** as a **local web tool** (amended 2026-11-16, user decision):
  `cargo run -p stubby_rules_server` serves one page on localhost that shows the rules
  as registries and rows, with a form for the selected row built from the JSON Schema,
  pickers for IDs, a live problems list, undo and redo, where-used, and save. The game's
  dev builds reload saved rules within a second, so you tune while it runs, in Jackdaw's
  Play or standalone.
- **Live game values beside the rules:** dev builds of the game serve Bevy's own remote
  protocol (`bevy_remote`) on localhost, and the page reads chosen resources from the
  running game (the basin's clean air, light quality at the camera, the time of day).
- A Jackdaw extension front end is **deferred**: its UI kit has no text input for
  extensions, and an extension loads beside the game project only as a signed, installed
  bundle (task 3.1, findings in design.md). It can come later over the same model.
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
- `crafting-designer`: designing game rules in a local web designer through forms,
  pickers, a problems list and where-used, saved as the canonical rules files, with live
  game values beside them.
- `rules-editing`: the Bevy-free editing model: typed edits, undo, renames with
  migrations, live validation and canonical saving.

### Modified Capabilities
None in `openspec/specs/` (it is empty). `add-rules-data-conventions` is unchanged:
the designer follows its files, IDs, `ids.lock` and migrations rules exactly.

## Impact

- `crates/stubby_recipes`: new `rules::edit` module; schema annotations on ID fields
  (a Rule PR in the AGENTS.md sense: schema code plus regenerated `schema/`). Still
  no Bevy.
- New tool crate `crates/stubby_rules_server` (binary): depends on `stubby_recipes`,
  `tiny_http`, serde and `serde_json` only; no Bevy, never the game. It is a tool,
  outside the game's layers (`split-game-into-systems`). The page is static HTML in the
  crate with `json-editor` (MIT) vendored, so it works offline.
- `stubby_game` dev builds add Bevy's `RemotePlugin` and `RemoteHttpPlugin` (localhost,
  CORS for the page); the resources the page shows derive `Reflect`.
- `bevy_stubby_recipes`: dev-build hot reload of the rules from disk; release builds
  keep the embedded rules.
- No change to the Jackdaw editor.
- `assets/recipes/README.md` and AGENTS.md: the designer is the normal way to author
  rules; hand edits still work.
