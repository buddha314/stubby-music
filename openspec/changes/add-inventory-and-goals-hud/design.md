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

### D3. Goals fill by partial delivery; where is open
Each goal keeps a **paid** amount per required item (Satisfactory's
`mPaidOffSchematic`, Mindustry's `finishedRequirements`, Stardew's bundle slots). A
delivery accepts, per item, up to what the goal still needs and what the player holds,
consumes exactly that through the same path as `Catalog::craft`, and adds it to
`paid`. Each delivery is one atomic transaction; a goal completes on the delivery that
leaves nothing unpaid. Paid amounts belong to the goal, so switching the active goal
loses nothing, and a deep long-term goal can be fed over many trips. Delivered items
are gone (a sink), so nothing counts twice. Holding items never completes a goal, so a
square wave stays usable as an instrument until the player chooses to hand it in.

The hub shows, per item, `paid + held / required`, distinguishing paid from held.
Where delivery happens is open (Open questions); the game delivers through a
`DeliverGoal` message, so the place can be a machine, a menu or a test later without
changing the model.

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

### D9. A pure reducer over a snapshot
Goal logic is one Bevy-free function, `apply(&Rules, &mut GoalProgress, &GoalEvent) ->
Vec<GoalOutcome>`: no clock, RNG or I/O. Events are `Deliver(goal)`,
`Crafted(recipe)`, `Choose(horizon, goal)`, `Pin(recipe, count)`. Outcomes are
`Paid{goal, item, n}`, `GoalCompleted(goal)` (emitted exactly once), `Opened(goal)`,
`Discovered(item)`, `PinDone(recipe)`. Duplicate events are harmless: completion is
latched and paid amounts only rise. Progress is a snapshot, not an event log; no
shipped game surveyed event-sources quest progress, and tests become tables of events
and expected outcomes. The game's Bevy systems only translate messages into events
and outcomes into feedback and HUD updates.

Future goal kinds get an explicit **accounting mode** each, in a rule PR
(`add-rules-data-conventions` D1): delivered-and-consumed (this change), a
produced/consumed ledger for throughput goals ("a stream carries N beats"), or a state
query re-evaluated on change for reversible conditions ("light quality ≥ x here").
A goal that opens late is evaluated against current state when it opens.

### D10. Save shape
The save holds only IDs and numbers: completed goals, paid amounts per goal and item,
the active choice per horizon, pins as `(recipe, count)`, recipe-book items, each rules
file's `format` and the applied migrations. On load, migrations run first
(`add-rules-data-conventions` D5); paid amounts or pins naming an ID that no longer
exists are dropped with a log line and, for pins, player feedback. Pins drop only by
crafting, never by delivery.

## Implementation plan
Each step is one PR, small enough to review, in this order. Steps 1–4 need no Bevy.

1. **Rules files and strict loading** (`add-rules-data-conventions` 2–4): domain files
   with `format`, strict validation with structured errors, canonical format, schema,
   migrations and `ids.lock`, shipped-rules and golden tests.
2. **Goal model** (this change 1.1–1.5): `goals.json` definitions and validation, the
   D9 reducer with partial delivery (D3), pins, `ItemProgress`; event-table tests for
   every goals-hub scenario. Codex.
3. **Provisional goals and simulation** (1.6, `add-rules-data-conventions` 4.3): a
   small `goals.json` from existing items, marked provisional until task 0.2, and the
   seeded economy simulation that proves it completable. Designer rows; Codex test.
4. **Mocks** (3.1–3.3), in parallel with 1–3: goals hub and inventory screen mocks,
   item icons, style guide. Claude.
5. **Screen mode and Tab inventory** (2.2–2.4, 2.8): the shared overlay mode with the
   rack screen, the inventory screen, counts out of the text HUD. Codex.
6. **Goals hub on screen** (2.1, 2.5, 2.6): rules loaded with `bevy_common_assets` and
   kept on a failed reload, the hub, `DeliverGoal` from a stopgap trigger until 0.1 is
   decided. Codex.
7. **Recipe book screen** (2.7): pins and the active-goal choice. Codex.
8. **Saving** (2.9, `add-rules-data-conventions` 5.2) when `moonshine-save` lands.

The first playable slice is steps 1–6: collect bugs and reeds, see the goal fill in the
upper right, hand items in, watch the next goal open.

## Risks / Trade-offs
- Hand-in place unknown → D3 keeps it behind a message; the first playable build
  delivers from a stopgap trigger (step 6), which the open question below must settle.
- Partial delivery means a goal can sit half paid → that is the point (deep goals over
  many trips); the hub shows paid and held apart so the player sees both.
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
