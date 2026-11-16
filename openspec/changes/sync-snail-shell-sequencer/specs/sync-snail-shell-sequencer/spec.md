## ADDED Requirements

### Requirement: Shell collection and mounting
Seeded snail shells SHALL yield the stable snail_shell item and mount in an optional 8 HP burbler place without being required for the original two-cable voice.

#### Scenario: Shell collection and mounting
- **WHEN** a shell is collected and mounted
- **THEN** one item is spent and four initially disabled steps appear

### Requirement: Four sample-timed steps
CLOCK rising edges SHALL cycle steps 1–4; RESET SHALL make the next clock play step 1. GATE SHALL pass the clock pulse only on enabled steps, CV SHALL hold the current knob value from 0 to 2 V/oct, snapped to semitones (1/12 V), and the step light SHALL invert while its heard step is active.

#### Scenario: Four sample-timed steps
- **WHEN** only steps 2 and 4 are enabled
- **THEN** only those beats gate the volume, with visual playhead timing derived from audio samples

### Requirement: Visible routing completes the rack
Configuration SHALL accept clock to sequencer CLOCK to volume TRIG when the VCO audio path is complete; every processor and connection SHALL remain visible or explicitly normalled.

#### Scenario: Visible routing completes the rack
- **WHEN** the player routes the gate through the optional sequencer
- **THEN** the burbler sounds selected steps and its stream flows without a hidden patch

### Requirement: Shells to find in the opening
About five shells SHALL lie by the reed beds of the opening scene, each with its own instance, collected with E into one `snail_shell` and never returning. The aimed-pickup outline SHALL show on a shell within reach, and the compass SHALL be shown for orientation.

#### Scenario: Collect a shell
- **WHEN** the player aims at a shell within reach and presses E
- **THEN** it is outlined before, one `snail_shell` is added, and the shell is gone for good
