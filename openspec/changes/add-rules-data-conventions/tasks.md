Sections 2–4.2 were built by Claude at the user's request (2026-11-01; `rules.rs`,
`rules-fmt`); Codex owns them from here. Owners follow AGENTS.md: Codex builds `crates/`; the designer owns rule rows; Claude
writes docs and art-adjacent rows. This change lands **before** the goal model in
`add-inventory-and-goals-hud` (see that change's implementation plan), so goals are
born strict.

## 1. Conventions on record (Claude)
- [x] 1.1 Research: `docs/research/crafting-and-goal-tracking.md` and its notes
- [x] 1.2 AGENTS.md "Rules data" section: ownership, the data-PR and rule-PR flows, ID rules, rows never hold logic
- [x] 1.3 `.github/pull_request_template.md` asking which flow a PR is (data / rule / other), and for rule PRs the OpenSpec change

## 2. Strict loading in `stubby_recipes` (Codex)
- [x] 2.1 `#[serde(deny_unknown_fields)]` on every definition struct (allowing `$schema`), with a test per struct that an unknown field fails
- [x] 2.2 IDs `[a-z][a-z0-9_]*`, unique across all registries (replace `insert_unique`'s per-map check)
- [x] 2.3 `RulesError { file, id, field, message }`, collected so one load reports every error; keep `Catalog::from_json` for tests
- [x] 2.4 Reachability: every recipe input is produced by some resource, mine or recipe output
- [x] 2.5 Rename resource `clock` → `clock_source` (the first migration; update `mudpit_burbler`'s `produces_resource` and any profile that names the resource, not the `clock` item)

## 3. Files, format and schema (Codex)
- [x] 3.1 Split `catalog.json` into `items.json`, `resources.json`, `mines.json`, `recipes.json` (`{ "format": 1, ... }`), content unchanged; `Rules::load(&RulesFiles)` loads and cross-validates them; the game loads `shipped_catalog()`
- [x] 3.2 Canonical-format test (sorted by `id`, pretty-printed, trailing newline) and a fix command (`cargo run -p stubby_recipes --bin rules-fmt`)
- [x] 3.3 `schemars` schemas written to `assets/recipes/schema/`, a staleness test, and `$schema` in each file
- [x] 3.4 `migrations.json` and `ids.lock` (design D5) with their validation tests

## 4. Tests on the shipped rules (Codex)
- [x] 4.1 `shipped_rules_load`: the real `assets/recipes/` loads with no errors
- [x] 4.2 Golden rows: lightning bug yields 2 square waves; reed clump timing; burbler produces the clock source
- [ ] 4.3 `economy_simulation`: seeded, Bevy-free, N nights of greedy collection under replenishment policies; asserts every goal in `goals.json` completes within a bound (vacuous until goals exist)

## 5. Game (Codex)
- [ ] 5.1 A failed hot reload keeps the previous rules and shows the errors (with `add-inventory-and-goals-hud` 2.1's `bevy_common_assets` loader)
- [ ] 5.2 Saves record each file's `format` and the applied migration IDs; apply pending migrations before restoring progress (with `moonshine-save`, `add-inventory-and-goals-hud` 2.9)
