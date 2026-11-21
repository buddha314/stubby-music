## ADDED Requirements

### Requirement: Foliage cleans the air within a radius
Each air-cleaning asset SHALL clean the air within its authored radius, with a strength that follows its growth and bloom. Overlapping cleaners SHALL define the undepleted air quality at a point, up to 1.
A point beyond every cleaner's radius SHALL be 0 (bleak). Plants and algae with
authored air-cleaner rules SHALL replenish local air quality toward that undepleted
level. Dancer consumption SHALL lower the same local value sampled by lights,
materials and mines. Air quality SHALL stay within 0 to 1.

#### Scenario: A blooming tree clears its ground
- **WHEN** a scrub oak with an 18 m radius reaches full bloom
- **THEN** light quality at its trunk eases to its strength, and 50 m away it stays 0

#### Scenario: A grove adds up
- **WHEN** several cleaners overlap at a point
- **THEN** light quality there is higher than any one alone and never above 1

#### Scenario: A dying grove fouls the air
- **WHEN** the plants of a grove die or are removed
- **THEN** light quality there eases back toward 0

### Requirement: Cleaning is data
Each air-cleaning asset kind SHALL declare its cleaning radius and strength as a rule (`assets/recipes/air.json`), and code SHALL NOT name asset kinds to decide what cleans. The Rules Designer SHALL expose
the cleaner's radius, strength and growth binding, and its replenishment rate
when shared air depletion is enabled.

#### Scenario: A new plant cleans
- **WHEN** a designer adds an `air` rule for a new flora asset
- **THEN** placed instances of it clean the air with no code change

### Requirement: Everything samples one light quality
The global lights, materials that vary by place and mine productivity SHALL all read light quality from the same sampler, at the camera, at the surface and at the mine respectively.

#### Scenario: Walking into a grove
- **WHEN** the player walks from bare desert into a blooming grove at night
- **THEN** the lighting eases from the bleak toward the bloomed night key without a visible step

### Requirement: The windstorm follows the basin
The windstorm's strength SHALL follow how clean the basin's air is on average, not the light quality where the camera stands. Until the windstorm is built, the dust in the sky SHALL thin as the basin is cleaned.

#### Scenario: Cleaning the basin calms the storm
- **WHEN** foliage cleans more of the basin
- **THEN** the dust in the sky thins everywhere, also where the player is not standing
