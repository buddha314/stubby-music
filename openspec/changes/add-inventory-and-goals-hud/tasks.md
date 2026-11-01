Owners follow AGENTS.md: Codex builds `crates/`; Claude builds HUD art and mocks; the
designer owns goal rows. Order: design "Implementation plan". `add-rules-data-conventions`
lands first, so `goals.json` follows its conventions (format, IDs, strict loading).

## 0. Decide (user)
- [ ] 0.1 Where goals are handed in (design, Open questions): a hub machine, the burbler, or anywhere
- [x] 0.2a First chores (design D11): "Build a heartbeat" (burbler configured), then "Water the tree" (needs `add-digging`)
- [ ] 0.2 The first long-term goal the chores lead toward

## 1. Goal model in `stubby_recipes` (Codex)
- [ ] 1.1 `GoalDefinition` (`id`, `name`, `horizon`, `requires: Vec<Stack>`, `after`, `unlocks`) and `Goals::from_json` validated against `Catalog`: unknown items or goals, empty `requires`, cycles
- [ ] 1.2 `GoalProgress` (complete goals, paid per goal and item, active choice per horizon) and the D9 reducer `apply(&Rules, &mut GoalProgress, &GoalEvent) -> Vec<GoalOutcome>`: open goals, active goal per horizon, partial delivery (D3) through the same consumption path as `Catalog::craft` (factor it out of `craft` rather than writing a second one), completion latched and reported once
- [ ] 1.3 `Pins` (`recipe → count`): totals from `RecipeDefinition::inputs`, decrement on craft, remove at zero
- [ ] 1.4 `ItemProgress { paid, held, required, state: Empty|Low|High|Full }` (state from `paid + held`) with the D6 thresholds, shared by goals (pins have `paid = 0`)
- [ ] 1.5 Event-table tests (events in, outcomes and progress out) for every scenario in `specs/goals-hub/spec.md` that needs no Bevy
- [x] 1.5b Goal definitions in the rules files: `goals.json` (format 1, schema, `ids.lock`), `GoalDefinition`/`Condition`/`Horizon` in `stubby_recipes::goals`, validated (IDs, exactly one of `requires`/`condition`, known items/machines/mines/goals, cycles) by `Rules::load` and `rules-fmt` (built by Claude at the user's request)
- [ ] 1.5a Condition goals (D11), the progress side: `condition` beside `requires` (exactly one), the closed condition list (`machine_configured`, `tree_watered`), `ConditionMet` event, latched completion; reject unknown conditions
- [x] 1.6a `assets/recipes/goals.json` holds the two chores of D11
- [ ] 1.6 Further goals in `goals.json`: with the content from 0.2 (until then, a minimal file using only existing items, marked provisional)

## 2. Game (Codex)
- [ ] 2.1 Load `goals.json` with `bevy_common_assets` (JSON, hot reload), handing the bytes to `Goals::from_json`; check its Bevy 0.20 release and pin the minor version
- [x] 2.2 One screen mode for overlays (`controls::ScreenMode`; the rack screen joins it when it lands) (design D7), shared with the rack screen (`add-desert-opening` 4.2c, `add-mudpit-burbler` 2.3): cursor free, movement/look off, Escape closes, one screen at a time
- [x] 2.3 Tab inventory screen (`inventory_screen.rs`; built ahead of the 3.1 mock, at the user's request) (`controls::INVENTORY = KeyCode::Tab`) in Bevy UI with `bevy_ui_widgets` (scroll area), listing items from `PlayerInventory` + `RecipeCatalog`; follow the 3.x mock
- [x] 2.4 Remove item counts from `update_hud`; keep prompts and feedback
- [ ] 2.5 Goals hub, upper right: active long-term and short-term goals and pin totals, as `ItemProgress` rows; update on `ResourceCollected`, `CraftCompleted` and delivery, not every frame
- [ ] 2.5a Report `machine_configured` when a machine's last `configure.cables` patch is made (with `add-mudpit-burbler` 2.5), and `tree_watered` once `add-digging` delivers water to a tree
- [ ] 2.6 `DeliverGoal` message → `GoalProgress::deliver`, with feedback; the trigger waits on 0.1
- [ ] 2.7 Recipe book (O) as a screen in the same mode: pin/unpin with a count, choose the active goal per horizon
- [ ] 2.8 Bevy tests for the Tab and screen-mode scenarios in `specs/inventory-screen/spec.md`
- [ ] 2.9 Save the D10 shape (completed, paid, active choices, pins, recipe book, formats, migrations) with `moonshine-save` when saving lands; apply migrations first and drop unknown IDs with a log line

## 3. Art (Claude)
- [ ] 3.1 Mock of the goals hub and the inventory screen in the HUD style of `docs/art/hud_mock.png` (`docs/art/goals_hub_mock.png`, `docs/art/inventory_mock.png`), with the four progress-bar states in signal amber
- [ ] 3.2 Item icons rendered from the models (square wave from the lightning bug, reed, clock) into `assets/hud/icons/`, named by item id, with a fallback icon; list them in a data file Codex can read
- [ ] 3.3 Style guide section for HUD panels and progress bars
