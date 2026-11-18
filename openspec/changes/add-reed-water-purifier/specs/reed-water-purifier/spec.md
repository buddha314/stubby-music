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
a VCF, a VCA and a scope, and its comb SHALL suggest the patch: low VCO TRI to mixer 1,
high VCO SAW to mixer 2, mixer OUT to VCF IN, VCF HPF to VCA IN, clock to VCA CV, VCA OUT
to scope IN, scope OUT to the outlet. The clock SHALL be the nearest running burbler's or
flowing stream's, lent as on the campfire.

#### Scenario: Patched as sketched
- **WHEN** every place is filled and the seven cables are patched, with a clock in reach
- **THEN** the purifier sounds: the clock gates the high-passed mix, and the scope shows it

#### Scenario: No clock in reach
- **WHEN** no running burbler or flowing stream is within reach
- **THEN** the clock place is empty and the purifier is silent

### Requirement: A working purifier clears the water downstream
A purifier SHALL raise the clarity of the stream downstream of it toward its cap while
its rack sounds with its targets met (low VCO C2, high VCO C4, cutoff at its target), and
SHALL let the clarity ease back when it stops.

#### Scenario: Cleaning
- **WHEN** a purifier stands in a stream, sounds, and meets its targets
- **THEN** the stream below it grows clearer, and a tree it waters may grow past budding

#### Scenario: Targets not met
- **WHEN** the purifier sounds but a VCO is not on its note
- **THEN** the water is not cleaned
