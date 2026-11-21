## ADDED Requirements

### Requirement: A step can wait for an item to have been found
A goal step's condition MAY be `item_found` with an item; it SHALL be met once the player's
recipe book knows that item, whether collected or crafted, and SHALL stay met when the item is
later consumed. Crafting SHALL add its outputs to the recipe book.

#### Scenario: Found, then used up
- **WHEN** the player collects a high-pass filter and crafts it into a purifier kit before the step opens
- **THEN** the step "Find a high-pass filter" completes when it opens, although no filter is held

#### Scenario: Never found
- **WHEN** the player has never collected or crafted the item
- **THEN** the step stays open
