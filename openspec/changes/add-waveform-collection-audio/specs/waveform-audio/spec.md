## ADDED Requirements
### Requirement: Shared waveform cues
World pulses and waveform collection confirmations SHALL use existing synthesis modules through a reusable waveform voice composition.
#### Scenario: Square bug flash
- **WHEN** a bug clock fires
- **THEN** its square voice uses a 15ms attack and 285ms decay, with a 300ms visual flash
### Requirement: Collection confirmation
Each successful waveform collection SHALL play one listener-level cue per distinct waveform type in its yields, independent of yield quantity and the removed world voice.
#### Scenario: Multi-item collection
- **WHEN** a bug grants two square-wave items
- **THEN** inventory gains two items and exactly one square confirmation voice plays
#### Scenario: No successful collection
- **WHEN** collection fails or E is held at an already depleted location
- **THEN** no confirmation voice is played
#### Scenario: Cue cleanup
- **WHEN** a confirmation envelope has finished
- **THEN** its temporary graph nodes are removed without affecting other world or confirmation voices
