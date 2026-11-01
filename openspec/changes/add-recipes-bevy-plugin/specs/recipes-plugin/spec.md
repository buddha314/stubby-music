## ADDED Requirements

### Requirement: Crafting installs as one Bevy plugin
The crafting system SHALL be provided to the game by a plugin crate,
`bevy_stubby_recipes`, whose `RecipesPlugin` is added with `app.add_plugins`. The game
SHALL NOT load rules files or apply harvests or crafts itself; it sends requests and
reads results.

#### Scenario: Adding the plugin
- **WHEN** a Bevy app adds `RecipesPlugin` and sends a harvest request for a lightning bug
- **THEN** the shipped rules are loaded, the player's inventory gains the yield, and a collected message names the instance

### Requirement: The rules core never depends on Bevy
`stubby_recipes` SHALL have no dependency on any Bevy crate, required or optional, and
its tests SHALL run without Bevy. Every rule (validation, inventory, crafting,
replenishment, goals) SHALL live in the core; the plugin SHALL only translate between
Bevy and the core.

#### Scenario: Dependency check
- **WHEN** the core's dependency tree is listed
- **THEN** it contains no `bevy` crate

### Requirement: One player inventory, for now
The plugin SHALL hold the player's items, harvest ledger and recipe book as one
resource. Inventories owned by other entities (machines, storage) are out of scope
until machines exist.

#### Scenario: Harvest and craft share the inventory
- **WHEN** a harvest request is applied and then a craft request
- **THEN** both act on the same player inventory

### Requirement: Game presentation stays in the game
HUD, prompts, input bindings, reach checks and screens SHALL remain in `stubby_game`;
the plugin SHALL expose messages and a public system set for the game to drive and
order against.

#### Scenario: Collecting a bug
- **WHEN** the player presses E within reach of a lightning bug
- **THEN** the game sends a harvest request, the plugin applies it, and the game shows the feedback
