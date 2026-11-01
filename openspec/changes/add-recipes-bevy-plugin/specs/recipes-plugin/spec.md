## ADDED Requirements

### Requirement: Crafting installs as one Bevy plugin
The crafting system SHALL be provided to the game by a plugin crate,
`bevy_stubby_recipes`, whose `RecipesPlugin` is added with `app.add_plugins`. The game
SHALL NOT load rules files or apply harvests, crafts or replenishment itself.

#### Scenario: Adding the plugin
- **WHEN** a Bevy app adds `RecipesPlugin` and spawns an entity with an `Inventory`
- **THEN** the rules load, and harvest and craft messages for that entity are applied

### Requirement: The rules core never depends on Bevy
`stubby_recipes` SHALL have no dependency on any Bevy crate, required or optional, and
its tests SHALL run without Bevy. Every rule (validation, inventory, crafting,
replenishment, goals) SHALL live in the core; the plugin SHALL only translate between
Bevy and the core.

#### Scenario: Dependency check
- **WHEN** the core's dependency tree is listed
- **THEN** it contains no `bevy` crate

### Requirement: Inventories belong to entities
Inventory, recipe book and harvest ledger SHALL be components on an owner entity, and
the harvest, craft and completion messages SHALL name that entity.

#### Scenario: Two owners
- **WHEN** a craft request names the player and another names a storage entity
- **THEN** each consumes from and adds to its own inventory only

### Requirement: Rules are a hot-reloaded asset that keeps the last valid rules
The plugin SHALL load the rules files as a Bevy asset and reload them when they
change. A reload that fails validation SHALL keep the previous rules and send a message
with the errors.

#### Scenario: Bad edit during play
- **WHEN** a rules file is saved with an unknown field while the game runs
- **THEN** play continues on the previous rules and an error message names the file, row and field

### Requirement: Game-specific item data is an extension
Item definitions in the core SHALL carry only generic fields plus an extension map the
core passes through unchanged; the game's waveform and signal-use data SHALL live in
the game's extension and be validated by the game's schema fragment.

#### Scenario: Square wave metadata
- **WHEN** the game reads the `square_wave` item
- **THEN** its waveform and uses come from the item's `stubby` extension, and the core's types name neither

### Requirement: Game presentation stays in the game
HUD, prompts, input bindings, reach checks and screens SHALL remain in `stubby_game`;
the plugin SHALL expose messages and a public system set for the game to drive and
order against.

#### Scenario: Collecting a bug
- **WHEN** the player presses E within reach of a lightning bug
- **THEN** the game sends a harvest request, the plugin applies it, and the game shows the feedback
