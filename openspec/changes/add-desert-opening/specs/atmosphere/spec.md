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

### Requirement: Day/night cycle
The world SHALL run a global day/night cycle through dawn, day, dusk and night, with
the cycle length configurable. Lighting SHALL be interpolated between the time keys
in `assets/lighting/lighting_keys.json` and SHALL never hard-cut, including when
night wraps to dawn.

#### Scenario: Continuous through the cycle
- **WHEN** time of day advances across a whole cycle
- **THEN** sun colour, illuminance, elevation, ambient, fog and exposure change continuously, matching each key exactly at its time

### Requirement: Light quality field
The world SHALL hold a light quality value in [0, 1] for each 10 m cubic cell,
0 meaning bleak and 1 meaning bloomed, defaulting to 0. Each cell's target SHALL be
set from local game conditions, chiefly the density of plants and animals (which the
player grows by playing music to them), and its value SHALL ease
toward the target over several seconds. Cells SHALL be stored sparsely, keyed by
integer cell coordinates, so an untouched desert costs nothing.

#### Scenario: Plants raise light quality locally
- **WHEN** plants grow densely within a cell
- **THEN** that cell's light quality eases upward over at least 3 seconds, and a cell 50 m away is unchanged

#### Scenario: Conditions can lower it
- **WHEN** the plants in a cell die or are removed
- **THEN** its light quality eases back toward bleak

### Requirement: Bleak to bloomed look
Lighting SHALL blend between the bleak and bloomed keys by the light quality sampled
at the camera, smoothly across cell borders. At 0 the scene SHALL be hazy and
desaturated; at 1 it SHALL be solarpunk by day (clear, saturated, lush) and lunarpunk
by night (violet sky, luminous accent flora). Values that vary across the view, such
as ground tint and flora emission, SHALL use the light quality of the cell they are
in, so a bloomed area reads distinctly from the bleak plain around it.

#### Scenario: Walking into a grove
- **WHEN** the player walks from bleak desert into a cell at light quality 1 at night
- **THEN** the lighting eases from the bleak night key to the bloomed (lunarpunk) night key without a visible step at the cell border

#### Scenario: Seeing a grove from outside
- **WHEN** the camera stands in bleak desert looking at a bloomed area
- **THEN** that area's ground and flora show bloomed colour while the light around the camera stays bleak

### Requirement: Visual style
Rendering SHALL be full 3D with a stylised, muted look: strong silhouettes, limited
palette, heavy atmospheric perspective at low light quality, and readable industrial forms.

#### Scenario: Silhouette readability
- **WHEN** a discoverable is within visibility range during the storm
- **THEN** it reads as a distinct silhouette against the haze

### Requirement: Desert scene art study
The initial scene SHALL provide a generated HDR desert sky with warm amber near
the low sun, dusty neutral haze at the horizon and muted slate overhead. It SHALL
include distant seeded mountain silhouettes and remain silent.

#### Scenario: Initial scene preview
- **WHEN** the standalone game is launched
- **THEN** cracked mud plates extend toward hazy mountain silhouettes under a warm desert sky
- **THEN** no audio is produced
