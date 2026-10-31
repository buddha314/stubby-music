## ADDED Requirements

### Requirement: Silent windstorm
At game start the world SHALL show a sandstorm — GPU-simulated blowing sand,
ground-hugging drift, and volumetric haze — while producing no sound.

#### Scenario: Storm at start
- **WHEN** a new game begins
- **THEN** blowing sand and haze are visible and the audio output is silent

### Requirement: Gusting visibility
Storm intensity SHALL vary over time in seeded gusts, changing fog density and
particle rate so that visibility alternates between near-zero and partial.

#### Scenario: Gust reveals landmark
- **WHEN** a gust trough occurs
- **THEN** visibility increases enough to silhouette objects at least 60 m away

### Requirement: Mood progression
A single `Mood` value in [0, 1] SHALL drive colour grading (saturation, temperature),
fog density, storm intensity, and sky light. It SHALL start at 0 (desaturated, cold,
dense) and rise with discoveries. Changes SHALL ease over several seconds and never
hard-cut.

#### Scenario: Mood rises on discovery
- **WHEN** the player discovers the Square VCO
- **THEN** `Mood` increases and saturation and warmth ease upward over at least 3 seconds

### Requirement: Visual style
Rendering SHALL be full 3D with a stylised, muted look: strong silhouettes, limited
palette, heavy atmospheric perspective at low Mood, and readable industrial forms.

#### Scenario: Silhouette readability
- **WHEN** a discoverable is within visibility range during the storm
- **THEN** it reads as a distinct silhouette against the haze
