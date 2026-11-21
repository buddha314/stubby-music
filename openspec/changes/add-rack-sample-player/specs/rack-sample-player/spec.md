## ADDED Requirements

### Requirement: Collected clips appear in the sampler library
Collecting an authored sound clip SHALL unlock its stable catalog ID. The rack
sampler picker SHALL list unlocked clips by name and allow the player to search
and audition them without assigning them. Audition audio SHALL use the
interface bus.

#### Scenario: A newly collected clip
- **WHEN** the player collects a clip and opens a sampler's picker
- **THEN** the clip is available to audition or assign

### Requirement: Each sampler holds its own clip selection
The sampler SHALL show its selected clip and allow the player to change or
clear the selection without changing cables. Rack configurations SHALL persist
the selected clip ID per module. A missing clip SHALL be shown as unavailable
and leave the sampler silent.

#### Scenario: Two samplers
- **WHEN** the player assigns different clips to two samplers and reopens the rack
- **THEN** each sampler still shows its own clip and retains its connections

### Requirement: A trigger plays the selected clip into the patch
A sampler SHALL start its selected clip on a rising gate edge using the
shared trigger threshold and emit that clip on its patchable audio output.
A held gate SHALL start the clip only once. Another rising edge SHALL
restart playback without an audible click.

#### Scenario: Play through an effect
- **WHEN** a clock triggers a sampler with its output patched to a filter
- **THEN** the selected clip starts on the trigger sample and passes through the filter
