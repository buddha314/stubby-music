## ADDED Requirements
### Requirement: Lightning bug resource yield
A lightning bug SHALL provide one square_wave item when collected. A world instance SHALL yield at most once. Holding a square-wave item SHALL NOT automatically make sound or change other bugs' clocks.
#### Scenario: Collect a bug
- **WHEN** the player collects an uncollected lightning bug in interaction range
- **THEN** inventory gains one square_wave item and the instance is marked collected
- **THEN** repeating collection of that instance adds nothing
### Requirement: General data-defined recipes
Recipes SHALL have stable IDs, declared input item quantities and declared output item quantities. Catalog validation SHALL reject duplicate IDs, unknown items and zero quantities. Inventory crafting SHALL aggregate repeated ingredient lines and apply input consumption and output production atomically.
#### Scenario: Insufficient ingredients
- **WHEN** a recipe requires more of any item than inventory contains
- **THEN** crafting returns the missing ingredient and inventory remains unchanged
#### Scenario: Successful recipe
- **WHEN** all inputs are available and output quantities fit inventory capacity
- **THEN** crafting consumes exactly the total declared inputs and adds exactly the total declared outputs
### Requirement: Waveform building blocks without invented progression
An empty production recipe catalog SHALL be valid while production combinations have not been authored. Recipe mechanics SHALL be independent of Bevy and the realtime audio engine.
#### Scenario: Empty initial catalog
- **WHEN** the game loads item and resource definitions with no production recipes
- **THEN** lightning bug collection and inventory still work and no invented craft output is presented

### Requirement: Square-wave resource in the recipe book
The lightning bug SHALL represent a mineable raw resource analogous to coal in a factory builder. Its discovered recipe-book entry SHALL be Square wave, with VCO, LFO and gate uses. Waveform knowledge SHALL survive consuming its inventory quantity. Other waveform sources SHALL be assigned to other world assets when designed.
#### Scenario: Waveform learned and used
- **WHEN** a player mines a lightning bug and later consumes its square-wave quantity in a valid recipe
- **THEN** the recipe book still lists Square wave and its VCO, LFO and gate roles

### Requirement: Familiar builder interaction keys
Collection SHALL use E. Holding E SHALL allow successive nearby resource pickups while each instance still yields at most once. Collection SHALL NOT move the camera vertically. The recipe book SHALL use O, keeping Q and F reserved for future building and dismantling.
#### Scenario: Hold interaction while moving
- **WHEN** E remains held as an uncollected resource comes within interaction range
- **THEN** it can be collected without a new key press and cannot yield twice
- **THEN** F does not collect resources and E does not raise the inspection camera
