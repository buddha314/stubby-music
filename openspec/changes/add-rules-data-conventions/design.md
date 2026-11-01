## Context
`assets/recipes/catalog.json` holds three items, three resources, two mines and no
recipes. `stubby_recipes::Catalog::new` checks unique IDs per registry against
`[A-Za-z0-9_-]`, resolves stack items and mine outputs, rejects zero quantities and
empty lists, and returns the first problem as `RecipeError::InvalidCatalog(String)`.
Definitions derive `Deserialize` without `deny_unknown_fields`. The research
(`docs/research/crafting-and-goal-tracking.md`) found five failure modes in shipped
games: unknown fields ignored (Factorio), well-formed but wrong data (Satisfactory's
quantity above stack size stalls a machine), content invisible until registered
(Satisfactory schematics), inheritance and load-order surprises (RimWorld, Dwarf
Fortress), and renamed IDs that drop player progress (Minecraft 1.16).

## Goals / Non-Goals
- Goals: rules a designer can add and change without an engineer, safely; mistakes
  caught at test time with a message that says where; saves that survive renames.
- Non-goals: a custom editor or spreadsheet pipeline; inheritance or templates between
  rows; any expression language in data; mod support.

## Decisions

### D1. Ownership and the two flows
| Change | Flow | Needs |
| --- | --- | --- |
| A new row or value of an existing type (a recipe, a goal, a yield, a replenishment interval) | **Data PR** | edits to `assets/recipes/*.json` only; `cargo test -p stubby_recipes` passes; no OpenSpec change |
| A new rule type (a field, a trigger or unlock kind, an accounting mode) | **Rule PR** | an OpenSpec change (SHALL + scenarios), then schema and validator code (Codex), then example rows |

The spec states invariants, not values, so values never need a spec change. Rows stay
declarative: when a row seems to need a condition or formula, that is a new rule type.

### D2. Files and format
One JSON file per domain under `assets/recipes/`, each `{ "format": 1, "<domain>":
[...] }`, loaded together and validated as one set. Entries are objects sorted by `id`,
printed by `serde_json`'s pretty printer with a trailing newline; a test fails on any
file not in that form and prints the fix command. One row per object keeps diffs and
merges to the rows that changed. `catalog.json` is split in one step, with no content
change, so the split PR is reviewable as a move.

### D3. IDs
`snake_case`, ASCII `[a-z0-9_]`, starting with a letter, never reused, separate from
the display `name`. One namespace across items, resources, mines, recipes and goals,
so a bare ID is never ambiguous. Instance IDs (placed nodes such as a bug's
`instance`) are not rule IDs and are out of scope.

### D4. Validation in three layers, all errors at once
1. **Shape:** serde with `deny_unknown_fields` on every definition struct; type and
   required-field errors carry the file and the serde path.
2. **Meaning:** IDs valid and unique; every reference resolves (stack items, mine
   outputs, goal `requires`/`after`/`unlocks`, pins); no cycles in `after`; no zero
   quantities or empty lists; every recipe's inputs are obtainable from some resource,
   mine or recipe output (no unreachable recipe).
3. **Simulation:** a seeded test (D7).

Errors become `RulesError { file, id, field, message }`, collected into a list so one
run reports every problem. `Catalog::from_json` keeps working for tests.

### D5. Migrations
`assets/recipes/migrations.json`: `{ "format": 1, "migrations": [{ "id":
"2026_11_clock_source", "from": "clock", "to": "clock_source", "registry":
"resources" }] }`; `to: null` removes. The validator checks that every `from` is gone
and every `to` exists. To notice an ID that vanished with no migration, a test keeps
`assets/recipes/ids.lock`, the sorted list of every rule ID ever shipped: a new ID is
appended by the test's fix command, and a listed ID missing from the rules must be
named by a migration. Saves store the `format` of each file and the migration IDs
applied; on load, pending migrations rename first, then progress naming unknown IDs
is dropped with a log line. Saves themselves land with `moonshine-save`; until then
migrations are validated and tested but have nothing to rewrite.

### D6. Schema for editors
Derive `schemars::JsonSchema` on the definition types and write
`assets/recipes/schema/<domain>.schema.json` from a test (or `cargo run` helper),
failing when they are stale. Each data file names its schema in `$schema`, so VS Code
autocompletes and underlines unknown fields while typing. `$schema` is the one
allowed extra key.

### D7. Tests
- `shipped_rules_load`: loads the real files from `assets/recipes/` and asserts no
  errors (not fixtures).
- Golden rows: a few canonical facts as assertions, e.g. a lightning bug yields 2
  square waves; reeds return after their interval.
- `economy_simulation`: seeded, deterministic; replays N in-game nights of collection
  under the replenishment policies with a simple greedy player and asserts every
  authored goal completes within a bound. It runs without Bevy.

### D8. Hot reload
When the game reloads rules (the `bevy_common_assets` loader), a set that fails
validation leaves the previous rules in place and shows the errors on screen, as
Minecraft's `/reload` does.

## Risks / Trade-offs
- Strictness slows a quick experiment → the error says exactly what to fix, and the
  schema autocompletes; that is cheaper than a silent typo.
- Splitting files adds cross-file loading → done once, behind one `Rules::load`.
- The simulation can be brittle as content grows → it asserts reachability within a
  generous bound, not balance targets.

## Open questions
- Whether a PR template is enough to mark data vs rule PRs, or a CI label check.
- Stack or capacity limits: none exist yet; the "quantity within stack limit" check
  arrives with them.
