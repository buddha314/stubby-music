## ADDED Requirements

### Requirement: Ordered first discoveries
The first module the player can discover SHALL be the Square VCO, and the second SHALL
be the Clock. The Clock SHALL NOT be collectable before the Square VCO.

#### Scenario: Clock before square
- **WHEN** the player reaches the Clock before discovering the Square VCO
- **THEN** the Clock cannot be collected and gives a cue pointing toward the square

### Requirement: The square is the first sound, heard only in range
Before it is discovered, the Square VCO SHALL emit its tone, spatialized, from its
world position. The tone SHALL be audible only within a hearing radius (default
35 m) and SHALL be the only sound in the game. Spawn SHALL always be outside that
radius, so the game starts silent.

#### Scenario: Silent at spawn
- **WHEN** a new game starts
- **THEN** the player is outside the square's hearing radius and the output is silent

#### Scenario: Caught on the wind
- **WHEN** the player first comes within the hearing radius
- **THEN** the square fades in from the direction of its position, and turning the camera changes where it is heard

#### Scenario: Found by ear
- **WHEN** the player walks toward where the square is heard
- **THEN** it grows louder and brighter-toned until the player reaches it

### Requirement: Square discovery joins the patch
Discovering the Square VCO SHALL add it to the player's patch, where it keeps
sounding.

#### Scenario: Collect the square
- **WHEN** the player discovers the Square VCO
- **THEN** it joins the patch without a gap or click in its tone

### Requirement: Visual lead as a fallback
The undiscovered Square VCO SHALL also have a visual cue (for example a square-wave
ripple in the sand) for players who cannot rely on audio.

#### Scenario: Visual lead
- **WHEN** the player is within 30 m of the undiscovered Square VCO
- **THEN** a visual cue is visible through the storm

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
