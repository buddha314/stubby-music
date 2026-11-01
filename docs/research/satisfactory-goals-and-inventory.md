# Satisfactory's goals, To-Do List and inventory, and the crates for ours

Research, 2026-11-01, for `add-inventory-and-goals-hud`: an inventory screen on Tab and
a goals hub in the upper right that tells the player what to make next. Question: how
does Satisfactory do it, and what can we take from crates.io instead of writing it?

## How Satisfactory shows what to do next

Sources: the wiki's [Milestone](https://satisfactory.wiki.gg/wiki/Milestone),
[Tiers](https://satisfactory.wiki.gg/wiki/Tiers),
[Progression](https://satisfactory.wiki.gg/wiki/Progression),
[To-Do List](https://satisfactory.wiki.gg/wiki/To_Do_List),
[HUD](https://satisfactory.wiki.gg/wiki/HUD) and
[Inventory](https://satisfactory.wiki.gg/wiki/Inventory) pages.

**Two horizons, both expressed as items.** Every goal in Satisfactory is a list of
item stacks to hand in. The game never says "build a factory"; it says "50 Iron Rods,
10 Rotors".

- **Short term: milestones.** Grouped in tiers. The player picks one milestone at the
  HUB Terminal; its cost (e.g. `200 × Concrete`) is shown at the **top right of the
  HUD** with the amount carried. The player hands the items in at the Terminal (E,
  then Ctrl+click the material); handed-in items cannot be taken back. Completing a
  milestone unlocks buildings, recipes, equipment or upgrades (e.g. inventory slots).
  Within an unlocked tier, milestones can be done in any order. Tier 0 (onboarding,
  the HUB upgrades) is done in order.
- **Long term: Space Elevator phases.** Each phase asks for "project parts", which are
  deep crafted items. A delivered phase unlocks the next tiers (phase 1 → tiers 3–4,
  2 → 5–6, 3 → 7–8, 4 → 9). While a phase is under construction its phase and
  required parts also show on the HUD. You do not need to finish a tier before
  delivering the next phase.

**The To-Do List** sits on the right of the screen and is the player's own goal list.
They add a recipe from the Codex (O), the build menu (Q) or a craft bench, with a
quantity (Shift for ±10). It shows how many buildings and components were added, then
the **total** components required against the amount in inventory. Each line has a
progress bar: grey with none, white below half, orange above half, solid orange with a
tick when the full amount is held. Crafting or building one subtracts one from the
list; when everything is done the list closes.

**The inventory** opens on Tab (unlocked once the Codex is first opened): a grid of
slots (18 at start, up to 78 with upgrades), hand and equipment slots, a trash slot,
sort, shift-click to move a stack, right-click to split, and right-click to open the
item's Codex page.

## What we take, and what we don't

| Satisfactory | stubby-music |
| --- | --- |
| Milestone with an item cost, shown top right | **Short-term goal**: an item list with owned/required and a progress bar, top right |
| Space Elevator phase with project parts | **Long-term goal**: deep crafted items, shown above the short-term goal; delivering it opens the next goals |
| Tiers gated by phases | Goals gated by prerequisites (earlier goals), all data |
| HUB Terminal hand-in | Hand-in place is **open**; no hub machine is designed yet |
| Rewards: recipes, buildings, slots | Unlocks: recipe-book entries and (later) buildables |
| To-Do List of pinned recipes | Pin a recipe from the recipe book (O); totals and bars in the goals hub |
| Inventory on Tab, slot grid | Tab screen; our inventory is a quantity per item, no slots (yet) |
| The game ends at the final phase | **It does not.** A simulator: when the authored goals run out the hub shows only the player's pins |

## Crates

Checked on crates.io 2026-11-01. We are on Bevy 0.20.1.

| Need | Crate | Bevy | Verdict |
| --- | --- | --- | --- |
| Goals/quests | [`bevy_quests`](https://crates.io/crates/bevy_quests) 0.1.0 | `^0.19` | 37 downloads, protobuf-defined RPG quests. No. |
| Goals/quests | [`micro_quest`](https://crates.io/crates/micro_quest) 0.1.0 | old | Stale (2023). No. |
| Goals/quests | [`mmorpg`](https://crates.io/crates/mmorpg) 0.1.0 | | Whole MMORPG toolkit. No. |
| Inventory | `bevy_inventory`, `bevy_items`, `bevy-slotted` | 0.18/0.19 | Already rejected in `crafting-and-inventory-crates.md`; keep `stubby_recipes`. |
| Widgets (buttons, scroll, sliders) | [`bevy_ui_widgets`](https://crates.io/crates/bevy_ui_widgets) 0.20.1 | Bevy's own | **Use.** Unstyled headless widgets, part of Bevy (Bevy's `bevy_ui_widgets` feature); we style them. Not `bevy_feathers`, Bevy's editor-tool look. |
| Drag and drop, clicks | `bevy_picking` | built in | **Use**, as already decided for the rack screen. |
| Goals data as a hot-reloading asset | [`bevy_common_assets`](https://crates.io/crates/bevy_common_assets) 0.18 | `^0.20` | **Use**, already recommended for the catalog. |
| Saving goal progress and pins | [`moonshine-save`](https://crates.io/crates/moonshine-save) 0.8 | `^0.20` | **Use** when saving lands, already recommended. |
| Tooltips | [`pyri_tooltip`](https://crates.io/crates/pyri_tooltip) 0.7.0 | `^0.19` | The common choice, but no 0.20 release yet. Wait, or a plain hover panel. |
| Tooltips | [`bevy_nested_tooltips`](https://crates.io/crates/bevy_nested_tooltips) 0.10.1 | `^0.20` | Nested (tooltip-in-tooltip) tooltips, e.g. an item's uses inside its tooltip. 189 recent downloads; optional for item tooltips if a plain hover panel is not enough. |
| Whole UI kits | `bevy_gooey` 0.14, `bevy_ui_kit` 0.2 | `^0.20` | ~100 downloads each, own skins. No; style is ours. |
| Immediate-mode UI | `bevy_egui` 0.43 | `^0.20` | Developer panels only, as already decided. The player HUD stays Bevy UI. |

**Conclusion.** No goal or quest crate fits, so goals are ours: a small, Bevy-free
model in `stubby_recipes` beside the catalog it validates against (the same reason we
kept our inventory). Everything around it comes from Bevy and the crates already
chosen: Bevy UI and `bevy_ui_widgets` for the screens, `bevy_picking` for clicks and
drags, `bevy_common_assets` for the goals data, `moonshine-save` for progress.
