## ADDED Requirements

### Requirement: Recorder gates sound collection
Sound yields SHALL require the recorder in hand; ordinary material pickups SHALL remain collectable without it. H SHALL toggle the single held item and SHALL be ignored while a screen owns controls. The default opening hand SHALL hold the recorder.

#### Scenario: Recorder gates sound collection
- **WHEN** a player attempts a waveform harvest without the recorder
- **THEN** nothing is taken and the refusal indicates the recorder action

### Requirement: Recorder shows capture and heard level
A successful sound harvest SHALL pulse the record light with 0.05 s attack and 0.4 s decay. Eight meter bars SHALL show world/music level from -48 dBFS in 6 dB steps with 24 dB/s fall, excluding UI confirmation sounds; silent opening SHALL leave them dark.

#### Scenario: Recorder shows capture and heard level
- **WHEN** a nearby bug clicks and then a UI snap plays
- **THEN** the click moves the meter but the UI snap does not
