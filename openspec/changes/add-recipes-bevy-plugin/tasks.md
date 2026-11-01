Thin plugin built by Claude at the user's request (2026-11-01); Codex owns it from here.
Owners follow AGENTS.md: Codex builds `crates/`; Claude writes docs. Lands after
`add-rules-data-conventions` sections 2–4 (done) and before `add-inventory-and-goals-hud`
2.x, whose game systems then go into the plugin.

## 1. Record (Claude)
- [x] 1.1 Research: `docs/research/crafting-system-library-maturity.md`
- [x] 1.2 AGENTS.md repo map and "Rules are data" section: the core/plugin/game split, and "no Bevy in the core, ever"

## 2. Plugin crate (Codex)
- [x] 2.1 `crates/bevy_stubby_recipes` in the workspace, `bevy = "~0.20"` with `default-features = false`; README with the Bevy compatibility table
- [ ] 2.2 (deferred) `RulesAsset` + loader for `assets/recipes/` with hot reload; keep the last valid rules on failure; `RulesInvalid` message (`add-rules-data-conventions` 5.1)
- [ ] 2.3 (deferred) `Inventory`, `RecipeBook`, `HarvestLedger` as reflected newtype components over the core types (design D4); `ResourceNode`, `MineNode` moved in, without game constructors
- [x] 2.4 `HarvestRequest`, `ResourceCollected`, `CraftRequest`, `CraftCompleted` (one player inventory, not owner entities); plugin systems in a public `RecipesSystems` set
- [x] 2.5 One headless plugin test (harvest request → inventory, recipe book, collected message)

## 3. Game moves onto the plugin (Codex)
- [x] 3.1 `stubby_game` adds `RecipesPlugin` (through `ResourceRecipesPlugin`); the game sends `HarvestRequest` and marks nodes from `ResourceCollected`; HUD, popup, E reach and replenishment stay in the game
- [x] 3.2 The game re-exports the plugin's types from `resources`, so `prefabs.rs`, `review.rs`, `prefab_review.rs`, `waveform_audio.rs` are unchanged; Jackdaw embeds the plugin source (`embedded_recipes_plugin`)
- [ ] 3.3 (deferred) Calendar feeds `ResourceClock`

## 4. Generic item metadata (Codex, rule PR)
- [ ] 4.1 (deferred) `ItemDefinition.extensions` (design D5); the core drops `waveform` and `uses`; `format` 2 with `rules-fmt` rewriting `items.json` and a test for the format change
- [ ] 4.2 (deferred) The game's typed `stubby` extension (waveform, uses) with its schema fragment, validated in `rules-fmt` and tests

## 5. Ready to extract (later, on a decision)
- [ ] 5.1 Decide whether and when to publish (design D7's conditions) and the name (Open questions)
- [ ] 5.2 `#[non_exhaustive]`, private fields, a non-Bevy and a Bevy example, README non-goals, MIT OR Apache-2.0
- [ ] 5.3 `git filter-repo --subdirectory-filter` into its own repository with both crates; the game depends on the published versions with `[patch.crates-io]` paths for local work
