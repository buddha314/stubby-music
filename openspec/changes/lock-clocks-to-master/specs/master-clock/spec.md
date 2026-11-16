## ADDED Requirements

### Requirement: Tempo clocks are whole multiples of the master
The engine SHALL derive every tempo clock's beat from its sample counter as a whole multiple of the 1 BPM master clock, without accumulating error, so that all tempo clocks begin a beat together at every master beat. Seeded random clocks SHALL remain free-running.

#### Scenario: Two clocks share the master's downbeat
- **WHEN** a 90 BPM and a 116 BPM clock are added at different times and the engine renders past a master beat
- **THEN** both tick on the first sample of that master beat

#### Scenario: No drift
- **WHEN** a 116 BPM clock runs for ten master beats
- **THEN** it has ticked exactly 1160 times, each on the sample its index predicts

### Requirement: Tempo is a whole number of BPM
A clock's tempo SHALL be a whole number from 1 to 999 BPM. The engine SHALL round a fractional tempo it receives, and the rack's tempo knob SHALL step in whole numbers.

#### Scenario: A fractional tempo
- **WHEN** a clock is set to 116.4 BPM
- **THEN** it runs at 116 BPM and the rack displays 116

### Requirement: Changing tempo keeps the grid
Changing a clock's tempo SHALL place it at the new multiple's position on the master grid without emitting a tick.

#### Scenario: Turning the knob
- **WHEN** a clock changes from 116 to 117 BPM between beats
- **THEN** no tick is emitted at the change, and its next tick falls on the 117 BPM grid
