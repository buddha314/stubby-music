## ADDED Requirements

### Requirement: Ordered first discoveries
The first module the player can discover SHALL be the Sine VCO, and the second SHALL
be the Clock. The Clock SHALL NOT be collectable before the Sine VCO.

#### Scenario: Clock before sine
- **WHEN** the player reaches the Clock before discovering the Sine VCO
- **THEN** the Clock cannot be collected and gives a cue pointing toward the sine

### Requirement: Sine wave is the first sound
Discovering the Sine VCO SHALL add it to the audio patch, and its tone SHALL be the
first sound the game produces.

#### Scenario: First sound
- **WHEN** the player discovers the Sine VCO
- **THEN** a sine tone fades in over at least 1 second and the world is no longer silent

### Requirement: Sine is findable in silence
Before it is discovered, the Sine VCO SHALL be locatable by a visual cue alone (for
example a faint oscillating shimmer or disturbance in the sand), because no sound
exists yet.

#### Scenario: Visual-only lead
- **WHEN** the player is within 30 m of the undiscovered Sine VCO
- **THEN** a visual cue is visible through the storm without any audio

### Requirement: Clock manifests as a pulsating light
The Clock SHALL appear in the world as a light whose brightness pulses on each clock
tick, timed from audio clock events rather than frame time. Once discovered it SHALL
be added to the patch and visibly pulse in time with what the player hears.

#### Scenario: Light on the beat
- **WHEN** the Clock is running at 90 BPM
- **THEN** the light peaks once per beat, within 10 ms of the corresponding audio tick at the display

#### Scenario: Pulse visible through storm
- **WHEN** the undiscovered Clock is within 100 m during a gust trough
- **THEN** its pulse is visible as a rhythmic glow in the haze
