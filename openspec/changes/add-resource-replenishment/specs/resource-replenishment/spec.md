## ADDED Requirements

### Requirement: Finite replenishment
Finite resources SHALL support never, elapsed game-time and next-night replenishment policies. A successful harvest SHALL yield once per availability period. Infinite mines SHALL remain separate from pickup replenishment.

#### Scenario: Next night
- **WHEN** a lightning bug is collected during day or night
- **THEN** it becomes collectible at the next night boundary strictly after collection, with its original yield and effects restored

#### Scenario: Timed return
- **WHEN** an elapsed-time resource reaches its configured deadline
- **THEN** it becomes available once without changing inventory or recipe discovery

### Requirement: Simulation calendar and speed
The game SHALL use a 3600 second cycle with 2700 seconds of daylight and 900 seconds of night at normal speed, and SHALL expose faster and slower simulation controls with a visible speed indicator. Simulation speed SHALL NOT change audio pitch or musical clock tempo.

#### Scenario: Speed changes
- **WHEN** the player changes simulation speed
- **THEN** movement, calendar and replenishment follow virtual time while synthesis retains its audio sample timing

### Requirement: Spring sun path
The sun SHALL follow an early-spring equinox arc at 37 degrees north, rising east, peaking south at approximately 53 degrees elevation and setting west. Its visible position and directional illumination SHALL share the simulation calendar and transition continuously through night. Sunlight, sky intensity, ambient illumination and fog color SHALL darken continuously at dusk.

#### Scenario: Solar noon
- **WHEN** daylight reaches its midpoint
- **THEN** the sun is south at 53 degrees elevation, never directly overhead

#### Scenario: Dusk lighting
- **WHEN** the sun approaches and passes below the horizon
- **THEN** directional light, sky and ambient brightness decrease and the fog shifts to a dark night color without retaining daylight haze
