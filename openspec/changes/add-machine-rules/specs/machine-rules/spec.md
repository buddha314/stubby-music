## ADDED Requirements

### Requirement: Machine settings are rules
A machine's tuned settings (its target settings and its clock reach) SHALL be rows in the
`machines` registry, keyed by the machine's placed asset ID, validated with the other rules
and edited in the rules designer.

#### Scenario: A malformed target
- **WHEN** a target gives both a note and a value, or a note that is not a note
- **THEN** loading the rules reports the error with the file, the machine's ID and the field

### Requirement: A saved setting reaches the running game
In a dev build, saving a machine's settings SHALL change the running game's racks within
the rules reload interval, without a rebuild.

#### Scenario: Retuning a target
- **WHEN** the designer saves a new target for the campfire while the game runs
- **THEN** the camp's rack checks the new target, and its log and the live panel show the result

### Requirement: The live panel shows each machine
The designer's live panel SHALL list each placed machine that has a rack, with whether it
sounds and whether its targets are met, naming every missed target and its current value.

#### Scenario: A missed gate
- **WHEN** the campfire's gate 4 is off
- **THEN** the panel shows the camp's targets missed, naming `seq.gate_4` at 0

### Requirement: Dev builds launch the designer
Starting a dev build of the game SHALL start the rules designer and open it, unless a
designer already answers on its port or `STUBBY_NO_DESIGNER` is set. Release builds SHALL
NOT start it.

#### Scenario: First launch
- **WHEN** a dev build starts and no designer is running
- **THEN** the designer starts and its page opens

#### Scenario: Already running
- **WHEN** a dev build starts while a designer is running
- **THEN** no second designer starts, and its unsaved edits are untouched
