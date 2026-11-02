## ADDED Requirements

### Requirement: Tab opens the inventory screen
Pressing Tab during play SHALL open the inventory screen; pressing Tab or Escape while
it is open SHALL close it. While it is open the cursor SHALL be free and player
movement and look SHALL stop. Opening it SHALL close any other open screen (recipe
book, rack screen), and closing it SHALL recapture the cursor.

#### Scenario: Opening and closing
- **WHEN** the player presses Tab during play, then Tab again
- **THEN** the inventory screen opens with the cursor free and movement stopped, then closes and the cursor is captured again

#### Scenario: Escape closes
- **WHEN** the inventory screen is open and the player presses Escape
- **THEN** the screen closes

#### Scenario: One screen at a time
- **WHEN** the recipe book is open and the player presses Tab
- **THEN** the recipe book closes and the inventory screen opens

### Requirement: The inventory is sixteen boxes of one hundred
The player's inventory SHALL hold 16 boxes, and a box SHALL hold up to 100 items of one
kind; an item held in larger numbers SHALL fill as many boxes as it needs. Adding items
that would need more than 16 boxes SHALL fail without changing the inventory (all or
nothing), so a collection or craft that does not fit takes nothing.

#### Scenario: More than a box
- **WHEN** the player holds 250 reeds
- **THEN** they fill three boxes: 100, 100 and 50

#### Scenario: Full
- **WHEN** all 16 boxes are in use and the player collects an item that needs another box
- **THEN** the collection fails and the inventory is unchanged

### Requirement: The inventory screen shows the boxes
The screen SHALL show the 16 boxes as a grid, filled in order with the icon of what each
holds and its count; the rest SHALL be empty boxes. Items the player holds none of SHALL
NOT be shown, not even as placeholders. Hovering a filled box SHALL show the item's
catalog name. No item name or id SHALL be hardcoded in the screen.

#### Scenario: Items held
- **WHEN** the player holds 2 square waves and 6 reeds and opens the inventory
- **THEN** two boxes show the square wave and reed icons with 2 and 6, and the other 14 are empty

#### Scenario: Used up
- **WHEN** the player has discovered square wave and holds none
- **THEN** no box shows a square wave

#### Scenario: New catalog item
- **WHEN** a new item is added to the catalog and the player collects it
- **THEN** it appears in a box, named on hover, without code changes

### Requirement: Item counts leave the text HUD
The bottom-left text HUD SHALL no longer list item counts; it SHALL keep the
interaction prompt and action feedback.

#### Scenario: Collecting with the screen closed
- **WHEN** the player collects a lightning bug with the inventory closed
- **THEN** the bottom-left HUD shows the feedback and no item count, and the goals hub updates

### Requirement: A new player starts with the starter inventory
A new game SHALL start the player's inventory from `assets/recipes/start.json`, validated
like the other rules files; it holds 10 patch cables.

#### Scenario: First Tab
- **WHEN** a new game starts and the player presses Tab
- **THEN** the inventory shows "Patch cable x 10"
