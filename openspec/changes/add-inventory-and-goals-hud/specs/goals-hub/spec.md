## ADDED Requirements

### Requirement: Goals are lists of items, in two horizons
Goals SHALL be defined as data in `assets/recipes/goals.json`, each with an id, a name,
a horizon (`long` or `short`), the item stacks it requires, the goals it comes after,
and the items it unlocks. Loading SHALL reject a goal that names an unknown item or
goal, has no required items, or forms a prerequisite cycle.

#### Scenario: Valid goals
- **WHEN** `goals.json` lists a long-term goal requiring catalog items and a short-term goal after nothing
- **THEN** both load and the short-term goal is open

#### Scenario: Unknown item
- **WHEN** a goal requires an item id the catalog does not define
- **THEN** loading fails naming the goal and the item

#### Scenario: Cycle
- **WHEN** goal A comes after B and B after A
- **THEN** loading fails naming the cycle

### Requirement: The hub shows the active goals as items, upper right
During play the upper right of the screen SHALL show the active long-term goal and the
active short-term goal, each as its name and its required items, every item with the
amount held, the amount required and a progress bar. A goal is open when every goal it
comes after is complete; the active goal of a horizon SHALL be the player's choice if
they made one, otherwise the first open goal of that horizon in file order. A horizon
with no open goal SHALL show nothing.

#### Scenario: Progress
- **WHEN** the active short-term goal needs 4 square waves and the player holds 1
- **THEN** the hub shows "Square wave 1 / 4" with a low progress bar

#### Scenario: Enough held
- **WHEN** the player holds all of a goal's items
- **THEN** each item shows a full bar with a tick, and the goal is not complete until delivered

#### Scenario: Goals not yet open
- **WHEN** a goal comes after a goal that is not complete
- **THEN** the hub does not show it

### Requirement: Delivering a goal completes it
Delivering a goal SHALL consume all of its required items from the inventory in one
transaction, or none if any is short, then mark the goal complete, discover its
unlocks in the recipe book and open the goals that come after it. Holding the items
SHALL NOT complete a goal by itself.

#### Scenario: Delivery
- **WHEN** the player holds 5 square waves and delivers a goal requiring 4
- **THEN** 1 square wave remains, the goal is complete, its unlocks are in the recipe book and the next goals open

#### Scenario: Short delivery
- **WHEN** the player delivers a goal while holding fewer items than it requires
- **THEN** nothing is consumed, the goal stays open, and feedback names the missing items

### Requirement: Goals never end the game
Completing the last authored goal SHALL NOT end, pause or reset the game. The hub
SHALL then show only the player's pinned recipes, and play continues.

#### Scenario: Last goal
- **WHEN** the player delivers the last authored goal
- **THEN** no ending appears, the goals section of the hub is empty, and the player keeps playing

### Requirement: Pinned recipes act as a To-Do List
The player SHALL be able to pin a recipe from the recipe book with a count, and change
or remove it there. The hub SHALL show the pins' total ingredients (each recipe input
times its pin count, summed per item) against the amount held, with progress bars.
Each completed craft of a pinned recipe SHALL lower its count by one, and a pin at zero
SHALL be removed.

#### Scenario: Totals
- **WHEN** the player pins a recipe needing 6 reeds with a count of 3 and holds 10 reeds
- **THEN** the hub shows "Reed 10 / 18"

#### Scenario: Crafting lowers the pin
- **WHEN** the player crafts a pinned recipe whose count is 1
- **THEN** the pin is removed from the hub

### Requirement: Progress bars have four states
Each item's progress bar SHALL show empty when none is held, low below half the
required amount, high at half or more, and full with a tick at the full amount.

#### Scenario: Thresholds
- **WHEN** a goal requires 4 of an item and the player holds 0, 1, 2 or 4
- **THEN** the bar is empty, low, high and full with a tick respectively
