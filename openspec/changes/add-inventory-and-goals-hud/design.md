## Context
`stubby_recipes` already owns the catalog (items, resources, mines, recipes),
`Inventory` (a quantity per item id), `RecipeBook` (discovered items, kept when the
quantity reaches zero), `Catalog::missing_ingredients` and `Catalog::craft`.
`stubby_game::resources` holds them in the `PlayerInventory` resource, sends
`ResourceCollected`, `CraftRequest` and `CraftCompleted`, and draws one text block,
bottom left (`update_hud`): hardcoded "Square waves / Reeds" counts, the E prompt, and
the recipe book toggled on O as more text. `controls` defines E and O and the cursor
capture (Escape and focus loss release it). Tab is reserved for inventory in AGENTS.md.

Satisfactory (research doc): milestones are short-term goals with an item cost, shown
top right; Space Elevator phases are long-term goals of deep crafted parts, also shown;
the To-Do List totals pinned recipes with progress bars; Tab opens the inventory.

## Goals / Non-Goals
- Goals: one place that says what to make next, always as items; an inventory screen;
  everything data-driven and testable without Bevy; reuse what we and Bevy have.
- Non-goals: an ending, a score, quest dialogue or story text; slot-based inventory
  with capacity, splitting and trash; the hub machine where goals are handed in (not
  designed yet); deciding the actual goal content beyond what exists.

## Decisions

### D1. Goals are item lists, in two horizons
A goal is `{ id, name, horizon: long|short, requires: [Stack], after: [goal id],
unlocks: [item id] }`. `requires` uses the catalog's `Stack`, so a goal is always
"make these". The hub shows one **active long-term goal** and one **active
short-term goal**. Short-term goals are steps toward the long-term one, but that link
is authored through `after`, not implied.

### D2. Prerequisites, not tiers
Satisfactory gates tiers on phases. We gate any goal on `after` (all listed goals
complete), which expresses tiers and phases without a separate concept. Among open
goals of a horizon, the hub shows the first in file order unless the player picks
another (D5). A goal whose `after` is unmet is hidden.

### D3. Delivery completes a goal; where is open
A goal completes when its items are **delivered**: consumed from the inventory in one
atomic transaction, like `Catalog::craft` (all or nothing, through the same
`Inventory` methods). `goals.json` and the model do not care where delivery happens.
Until a hand-in place is designed (see Open questions), the game delivers through a
`DeliverGoal` message, so the place can be a machine, a menu or a test later without
changing the model. Holding the items is not enough: a goal never completes by itself,
so the player can keep using a collected square wave.

### D4. Unlocks feed the recipe book
`unlocks` are item ids discovered into `RecipeBook` on completion. Buildables join
when the build menu (Q) exists. An unlock is never an ending: after the last authored
goal, the hub shows pins only, and the world goes on.

### D5. Pins are the To-Do List
A pin is `(recipe id, count)`. The hub totals the pins' ingredients
(`inputs × count`, summed per item) and shows each against the inventory.
`CraftCompleted` for a pinned recipe lowers its count by one; at zero the pin is
removed. Pins are added and adjusted in the recipe book (O). The player may also
pick which open goal of each horizon is active from the recipe book; the choice is
remembered.

### D6. Progress bars follow Satisfactory's thresholds
Per item: none held → empty; below half → low; half or more → high; all → full, with
a tick. Bars and colours come from the art direction (amber signal light, style
guide), not Satisfactory's orange.

### D7. Screens share one mode
The Tab screen, the recipe book and the rack screen are the same kind of thing: an
overlay with the cursor free, movement and look off, Escape to close. Use one
`ScreenMode` (or the rack-screen input context planned in `add-desert-opening` 4.2c
on `bevy_enhanced_input`), so only one screen is open at a time and closing it
recaptures the cursor. Opening one screen closes another.

### D8. Crates
No goal, quest or inventory crate fits (research doc). Build on Bevy UI with Bevy's
`bevy_ui_widgets` (headless buttons, scroll areas) styled by us, `bevy_picking` for
clicks, `bevy_common_assets` to load `goals.json` with hot reload (bytes go to
`stubby_recipes` for validation, so it stays Bevy-free), and `moonshine-save` for
completed goals, active choices and pins when saving lands. `bevy_egui` stays
developer-only. Optional later: `bevy_nested_tooltips` for item tooltips;
`pyri_tooltip` once it supports Bevy 0.20.

## Risks / Trade-offs
- Hand-in place unknown → D3 keeps it behind a message; the first playable build may
  deliver from the hub itself as a stopgap, which the open question below must settle.
- Goal content is thin (three items, no recipes yet) → ship the mechanism with a
  small `goals.json` using only existing items; real content follows the machines.
- Two text HUDs during migration → the bottom-left HUD keeps prompts and feedback
  only; counts move to Tab in the same task.

## Open questions
- Where goals are handed in: a hub machine in the world (Satisfactory's HUB terminal),
  the burbler, or anywhere. A hub machine is art and design work not yet started.
- Whether the inventory gets slots and capacity later (Satisfactory has 18 to start).
- First goal content: which long-term goal opens the game (e.g. toward the first clean
  water, `add-opening-region`), and which short-term steps lead to it. Do not invent
  recipes or sources for it; AGENTS.md says machine recipes wait for their design.
