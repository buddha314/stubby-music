## Why
The game's rules (items, resources, mines, recipes, and soon goals) are JSON in git,
checked by `stubby_recipes`. That is the setup shipped systemic games converge on:
engineers own a typed schema and its interpreter, designers own the rows, and a
validator rejects bad rows before the game runs (Unreal Data Tables, RimWorld Defs,
Minecraft data packs, Factorio prototypes, Satisfactory recipes and schematics). What
we lack are the conventions that make it safe to keep editing: today a typo in a field
name is silently ignored, IDs may contain uppercase and `-`, the item `clock` and the
resource `clock` share an ID, errors are one string with no file or field, and nothing
protects a save when an ID is renamed. Those are the failure modes the research found
in shipped games.

Research: `docs/research/crafting-and-goal-tracking.md` (and its sourced notes).

## What Changes
- **Who edits what, and two flows.** The designer owns rows; Codex owns the schema and
  validator; OpenSpec owns rule types and invariants. A new row of an existing type is
  a data-only PR. A new rule type is an OpenSpec change, then code, then example rows.
  Rows never grow logic. Recorded in AGENTS.md.
- **Strict loading.** Unknown fields are an error. IDs are `snake_case` `[a-z0-9_]`,
  unique across every registry. Every reference must resolve.
- **Structured errors** naming file, ID, field and problem; every error is reported,
  not just the first.
- **Files by domain**, each with a `format` version: `items`, `resources`, `mines`,
  `recipes` (and `goals` from `add-inventory-and-goals-hud`) under `assets/recipes/`,
  one object per entry, sorted by `id`, in canonical formatting checked by a test.
- **IDs are save API.** Renaming or removing an ID needs a row in
  `assets/recipes/migrations.json`; saves record the migrations applied.
- **JSON Schema** generated from the Rust types for editor autocomplete.
- **Three test layers:** the real shipped files load clean; golden tests for canonical
  rows; a seeded economy simulation proving every authored goal can be completed.
- **Hot reload keeps the old rules** when new ones fail validation, and shows the error.
- **Fix the one collision:** resource `clock` becomes `clock_source` (with a migration).

## Capabilities
### New Capabilities
- `rules-data`: how game rules are authored, validated, versioned and migrated.

## Impact
`stubby_recipes` (definitions, validation, errors, migrations), `assets/recipes/`
(split files, `migrations.json`, schema), `stubby_game` catalog loading (and the
`bevy_common_assets` loader planned in `add-inventory-and-goals-hud` 2.1),
`assets/machines/mudpit_burbler/asset.json` if it names the resource, AGENTS.md, and
a PR template asking which flow a change is. No gameplay change.
