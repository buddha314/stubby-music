## ADDED Requirements

### Requirement: Ripple pools yield an oscilloscope
A ripple pool SHALL yield one oscilloscope when collected with E. Collecting SHALL NOT
remove the pool or stop its ripples.

#### Scenario: Collecting
- **WHEN** the player aims at an uncollected ripple pool and presses E
- **THEN** one oscilloscope is added to the inventory and the pool keeps rippling

#### Scenario: Collected already
- **WHEN** the player presses E at a pool already collected
- **THEN** nothing is added

### Requirement: Ripples show the sound reaching the pool
A ripple pool's rings SHALL follow the sound reaching it, timed from the audio clock.
A pool with no sound reaching it SHALL be still.

#### Scenario: Silent start
- **WHEN** nothing sounds within hearing of a pool
- **THEN** its surface is still

#### Scenario: Near a voiced burbler
- **WHEN** a voiced burbler sounds within hearing of a pool
- **THEN** rings leave the pool's centre in time with the burbler's beat

### Requirement: Ripple pools lie on wet ground
Seeded placement SHALL put ripple pools only where the wetness channel marks the ground
wet, and the opening region SHALL hold at least one.

#### Scenario: Seed 17
- **WHEN** the opening is generated from seed 17
- **THEN** at least one ripple pool lies on wet ground in the region
