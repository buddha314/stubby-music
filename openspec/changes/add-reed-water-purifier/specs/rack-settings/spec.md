## ADDED Requirements

### Requirement: Pitch knobs click to notes
A module knob that declares semitone quantization SHALL take only equal-tempered semitone
values (A4 = 440 Hz) within its range, and the module's display SHALL show the value as a
note name with its octave.

#### Scenario: Turning a quantized VCO
- **WHEN** the player turns a quantized frequency knob toward 65.6 Hz
- **THEN** it settles on C2 (65.41 Hz), the engine plays 65.41 Hz, and the display reads C2

### Requirement: A machine can require settings
A machine's game rule SHALL apply only while its rack sounds and every target setting its
profile lists (knob values, exactly) is met.

#### Scenario: All targets met
- **WHEN** every targeted knob holds its target value and the rack sounds
- **THEN** the machine works

#### Scenario: One target missed
- **WHEN** one targeted knob is a semitone away from its target
- **THEN** the machine does not work, though the rack still sounds
