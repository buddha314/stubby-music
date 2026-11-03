## ADDED Requirements

### Requirement: Buildables have item costs
`assets/recipes/buildables.json` SHALL list what the player can build, each row an ID, a
name and a cost in item stacks. Cost items SHALL be known items. Building SHALL take the
cost times the count from the inventory all or nothing.

#### Scenario: Building with enough items
- **WHEN** the inventory holds 13 reeds and the player builds 2 `stream_segment` (6 reeds each)
- **THEN** 12 reeds are taken and 1 remains

#### Scenario: Unknown cost item
- **WHEN** a buildable's cost names an item that does not exist
- **THEN** loading fails, naming `buildables.json`, the buildable's ID and the field
