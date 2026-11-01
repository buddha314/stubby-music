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

### Requirement: The inventory screen lists items from the catalog
The screen SHALL list every item the player holds, and every item in the recipe book
the player holds none of, by its catalog name, with its quantity, description and
signal uses. Discovered items with quantity zero SHALL be shown as known but empty.
No item name or id SHALL be hardcoded in the screen.

#### Scenario: Items held
- **WHEN** the player holds 2 square waves and 6 reeds and opens the inventory
- **THEN** the screen shows "Square wave" × 2 with its description and uses (VCO, LFO, Gate) and "Reed" × 6

#### Scenario: Known but used up
- **WHEN** the player has discovered square wave and holds none
- **THEN** the screen shows "Square wave" as known with quantity 0

#### Scenario: New catalog item
- **WHEN** a new item is added to the catalog and the player collects it
- **THEN** it appears on the screen without code changes

### Requirement: Item counts leave the text HUD
The bottom-left text HUD SHALL no longer list item counts; it SHALL keep the
interaction prompt and action feedback.

#### Scenario: Collecting with the screen closed
- **WHEN** the player collects a lightning bug with the inventory closed
- **THEN** the bottom-left HUD shows the feedback and no item count, and the goals hub updates
