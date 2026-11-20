## ADDED Requirements

### Requirement: Four reviewable settlement phases

The project SHALL provide independent Jackdaw review scenes for Camp, Grove,
Village and City. Each scene SHALL show the same site and preserve its shared
orientation, water route and landmarks. Later scenes SHALL show visible
additions appropriate to their phase.

#### Scenario: A designer compares two phases

- **WHEN** a designer opens Camp and then City
- **THEN** the same burbler, campfire, channel head and window rock provide
  reference points for comparing the more developed layout

### Requirement: Shared asset families and visible placeholders

Phase scenes SHALL reference the shared master prefabs for existing assets.
Unbuilt settlement architecture SHALL be represented by named scene blockouts.
A phase asset index SHALL identify reusable families and outstanding art needs.

#### Scenario: A designer changes the shared campfire

- **WHEN** the campfire master prefab is updated
- **THEN** all four phase scenes resolve the same updated prefab

### Requirement: Bounded phase review

Each phase SHALL be selectable by name in the standalone graphical watchdog so
a designer can inspect it with a common seed.

#### Scenario: Review the grove

- **WHEN** a designer selects Grove in the standalone watchdog
- **THEN** Jackdaw loads the Grove scene with seed 42 under the existing time limit
