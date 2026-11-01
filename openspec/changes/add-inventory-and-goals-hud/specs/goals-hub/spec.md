## ADDED Requirements

### Requirement: Goals are lists of items, in two horizons
Goals SHALL be defined as data in `assets/recipes/goals.json`, each with an id, a name,
a horizon (`long` or `short`), either the item stacks it requires or a condition that
meets it, the goals it comes after, and the items it unlocks. Loading SHALL reject a
goal that names an unknown item, goal or condition, has neither required items nor a
condition (or both), or forms a prerequisite cycle.

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
amount already paid into the goal, the amount held, the amount required and a progress
bar of paid plus held against required. A goal is open when every goal it
comes after is complete; the active goal of a horizon SHALL be the player's choice if
they made one, otherwise the first open goal of that horizon in file order. A horizon
with no open goal SHALL show nothing.

#### Scenario: Progress
- **WHEN** the active short-term goal needs 4 square waves, none is paid and the player holds 1
- **THEN** the hub shows square wave at 1 of 4 with a low progress bar

#### Scenario: Paid and held
- **WHEN** 2 square waves are paid into a goal needing 4 and the player holds 1
- **THEN** the hub shows 2 paid and 1 held of 4, with a high progress bar

#### Scenario: Enough held
- **WHEN** paid plus held covers all of a goal's items
- **THEN** each item shows a full bar with a tick, and the goal is not complete until delivered

#### Scenario: Goals not yet open
- **WHEN** a goal comes after a goal that is not complete
- **THEN** the hub does not show it

### Requirement: Goals fill by delivery
Each goal SHALL keep a paid amount per required item. Delivering to a goal SHALL, for
each required item, consume from the inventory the lesser of what the goal still needs
and what the player holds, add it to the goal's paid amount, all in one transaction.
The goal SHALL complete on the delivery that leaves nothing unpaid, and only then
SHALL its unlocks be discovered in the recipe book and the goals after it open.
Completion SHALL be reported exactly once. Holding items SHALL NOT complete a goal,
and paid amounts SHALL be kept when the player switches the active goal.

#### Scenario: Partial delivery
- **WHEN** a goal needs 4 square waves and 6 reeds, nothing is paid, and the player holds 1 square wave and 10 reeds and delivers
- **THEN** 1 square wave and 6 reeds are consumed, the goal shows 1 of 4 and 6 of 6 paid, it stays open, and the player keeps 4 reeds

#### Scenario: Completing delivery
- **WHEN** the same goal has 3 square waves left to pay and the player holds 5 and delivers
- **THEN** 3 are consumed, 2 remain, the goal completes once, its unlocks are in the recipe book and the next goals open

#### Scenario: Nothing to give
- **WHEN** the player delivers while holding none of the items the goal still needs
- **THEN** nothing changes and feedback names what the goal still needs

#### Scenario: Switching goals
- **WHEN** 2 of 4 square waves are paid into a goal and the player makes another goal active, then switches back
- **THEN** the first goal still shows 2 of 4 paid

#### Scenario: Repeated event
- **WHEN** the completing delivery's event is applied twice
- **THEN** the goal is reported complete once and nothing more is consumed

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

### Requirement: Chores are met by a condition
A goal with a `condition` SHALL complete, once, the first time the game reports that its
condition holds, and SHALL stay complete if the condition later stops holding. The hub
SHALL show a condition goal as its name and a one-line description of what meets it,
with no item rows.

#### Scenario: Build a heartbeat
- **WHEN** the "Build a heartbeat" chore is active and the player patches the burbler's last cable so it sounds
- **THEN** the chore completes and "Water the tree" opens

#### Scenario: Unpatching later
- **WHEN** the player unplugs a cable from a burbler after "Build a heartbeat" completed
- **THEN** the chore stays complete

#### Scenario: Unknown condition
- **WHEN** a goal names a condition the schema does not define
- **THEN** loading fails naming the goal and the condition
