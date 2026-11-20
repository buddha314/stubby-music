## ADDED Requirements

### Requirement: Ripple pools form along streams
Ripple pools SHALL form at random places along each laid stream segment, as a Poisson
process in arc length with a mean spacing set in data, seeded from the world seed and the
segment's identity. The same segment in the same world SHALL get the same pools.

#### Scenario: Laying a long stream
- **WHEN** the player lays a stream many times the mean spacing long
- **THEN** ripple pools appear along it at irregular gaps, about one per mean spacing

#### Scenario: Same segment, same pools
- **WHEN** a segment is dismantled and laid again in the same place in the same world
- **THEN** its pools return at the same places

#### Scenario: Dismantled
- **WHEN** a segment is dismantled
- **THEN** its pools are removed with it

### Requirement: Pools ripple to the beat
A ripple pool on a sounding stream SHALL send rings from its centre on each beat of the
stream's clock, timed from the audio clock. A pool on a silent stream SHALL be still.

#### Scenario: Sounding stream
- **WHEN** a burbler's stream sounds at 96 BPM
- **THEN** each of its pools ripples 96 times a minute, on the beat

#### Scenario: Silent stream
- **WHEN** a stream carries no sound
- **THEN** its pools are still

### Requirement: Pools yield oscilloscopes semi-daily
A ripple pool SHALL yield one oscilloscope when collected with E, and SHALL yield again
half a day of simulation time later. Collecting SHALL NOT stop its ripples.

#### Scenario: Collecting
- **WHEN** the player aims at an uncollected pool and presses E
- **THEN** one oscilloscope is added to the inventory and the pool keeps rippling

#### Scenario: Back half a day later
- **WHEN** half a day of simulation time has passed since a pool was collected
- **THEN** it yields an oscilloscope again
