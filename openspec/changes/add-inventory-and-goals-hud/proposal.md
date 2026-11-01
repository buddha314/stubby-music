## Why
The player has no way to see what they carry, beyond one line of text in the bottom
left, and no way to know what to do next. Satisfactory answers both, and its answers
fit a factory builder: an inventory screen on Tab, and goals in the upper right that
are always **lists of items to make**. The short-term goal (a milestone) and the
long-term goal (a Space Elevator phase) each show the items they need against what the
player holds, and the player's own pinned recipes (the To-Do List) sit with them.

This is a simulator, not a level game: goals point the way and open new things; they
never end the game. When the authored goals run out, the hub shows only what the
player pinned.

Research: `docs/research/satisfactory-goals-and-inventory.md`.

## What Changes
- **Inventory screen on Tab.** A full-screen panel listing every item the player holds
  or has discovered, by catalog name, with quantity, description and signal uses.
  Tab or Escape closes it. While open, the cursor is free and movement and look stop,
  through the same screen mode the burbler rack screen uses. The bottom-left text HUD
  stops listing item counts.
- **Goals hub in the upper right.** Always visible during play:
  - the **long-term goal** (title, and the deep crafted items it needs),
  - the **short-term goal** (title, and the items it needs),
  - the player's **pinned recipes** (the To-Do List), as total ingredients,
  each item with `owned / required` and a Satisfactory-style progress bar.
- **Goals are data.** Long-term goals and short-term goals live in
  `assets/recipes/goals.json`, validated against the recipe catalog: item costs,
  prerequisites (which goals open which), and unlocks (recipe-book entries, later
  buildables). Delivering a goal's items completes it and opens the goals that follow.
- **Pinning.** In the recipe book (O), the player pins a recipe with a quantity; the
  hub totals its ingredients. Crafting one lowers the pin by one; a finished pin
  disappears.
- **Model in `stubby_recipes`.** No goal or quest crate fits (research), so the goal
  model is ours and Bevy-free, beside the catalog. Around it we use Bevy UI with Bevy's
  `bevy_ui_widgets`, `bevy_picking`, `bevy_common_assets` (goals as a hot-reloading
  asset) and, when saving lands, `moonshine-save`.

## Capabilities
### New Capabilities
- `inventory-screen`: the Tab inventory screen.
- `goals-hub`: long-term and short-term goals and pinned recipes, as items, in the
  upper right; the data-driven goal model behind them.

## Impact
`stubby_recipes` (goal definitions, validation, progress, delivery, pins),
`stubby_game::resources` (the HUD split: counts leave the text HUD), `controls`
(Tab), the shared screen mode with the rack screen (`add-desert-opening` 4.2c,
`add-mudpit-burbler` 2.3), new `assets/recipes/goals.json`, and HUD art (panels, item
icons) in `assets/hud/` and `docs/art/`. No editor change.
