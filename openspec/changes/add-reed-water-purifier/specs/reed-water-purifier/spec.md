## ADDED Requirements

### Requirement: The reed water purifier is made at the campfire
The `reed_water_purifier` recipe SHALL be made at a lit campfire, and only while the
campfire's clock is inside the recipe's BPM range.

#### Scenario: Made at tempo
- **WHEN** the player crafts the purifier at a lit camp whose clock is in the recipe's range and holds its inputs
- **THEN** the inputs are taken and the player holds a reed water purifier

#### Scenario: Off tempo
- **WHEN** the camp's clock is outside the recipe's range
- **THEN** nothing is taken and the screen shows the range beside the clock's BPM

### Requirement: The purifier's rack is the high-pass patch
A placed purifier SHALL have a rack with a borrowed clock, a low VCO, a high VCO, a mixer,
a VCF, a VCA and a scope, and its screen SHALL suggest the patch: low VCO SAW to mixer 1,
high VCO TRI to mixer 2, mixer OUT to VCF IN, VCF HPF to VCA IN, clock to VCA CV, VCA OUT
to scope IN, scope OUT to the outlet. The clock SHALL be the nearest running burbler's or
flowing stream's, lent as on the campfire.

#### Scenario: Patched as sketched
- **WHEN** every place is filled and the seven cables are patched, with a clock in reach
- **THEN** the purifier sounds: the clock gates the high-passed mix, and the scope shows it

#### Scenario: No clock in reach
- **WHEN** no running burbler or flowing stream is within reach
- **THEN** the clock place is empty and the purifier is silent

### Requirement: Water has four levels of clarity
Stream water SHALL have four levels of clarity, from dirtiest to cleanest: muddy, silty,
cloudy and clear. A stream SHALL leave its burbler muddy.

#### Scenario: A new stream
- **WHEN** a stream is laid from a burbler with no filter on it
- **THEN** its water is muddy along its whole length

### Requirement: The purifier is a check dam in the stream
A purifier SHALL be placed onto a laid stream segment, across its bed and square to its
flow, as a small check dam the water pools behind and spills over. It SHALL NOT be
placeable off a stream, and it SHALL NOT stop the stream's flow or music.

#### Scenario: Placing on a stream
- **WHEN** the player aims the purifier's hologram at a laid stream segment
- **THEN** it snaps across that segment's bed, and a click places it there

#### Scenario: Off the stream
- **WHEN** the hologram is aimed at bare ground
- **THEN** it shows red and cannot be placed

### Requirement: A working purifier lifts the water one level
A purifier SHALL raise the stream below it one clarity level above the water reaching it
while its rack sounds with its targets met (low VCO C2, high VCO C4, cutoff at its target),
and SHALL let the clarity ease back when it stops.

#### Scenario: Cleaning
- **WHEN** a purifier stands in a muddy stream, sounds, and meets its targets
- **THEN** the stream below it rises to silty, and a tree it waters may grow past budding

#### Scenario: Targets not met
- **WHEN** the purifier sounds but a VCO is not on its note
- **THEN** the water is not cleaned

#### Scenario: Upstream unchanged
- **WHEN** a purifier is working
- **THEN** the water above the dam keeps its clarity
