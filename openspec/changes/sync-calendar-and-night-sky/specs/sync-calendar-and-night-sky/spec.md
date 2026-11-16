## ADDED Requirements

### Requirement: One solar-time mapping
Sky, sun, moon, stars, direct light, fog and environment lighting SHALL use one calendar mapping: 2700 s day maps 06:00–18:00 and 900 s night maps 18:00–06:00. Opening time SHALL be authored data, defaulting to sunrise. Musical timing SHALL remain sample timed.

#### Scenario: One solar-time mapping
- **WHEN** the calendar runs at 64x
- **THEN** the sun, moon and stars move at 64x while pitch and BPM remain unchanged

### Requirement: Night sky travels coherently
The sky SHALL be drawn by an atmosphere lit by the sun and moon, so its colour follows their positions (warm, dusty dawn and dusk; hazy day; dark night). The moon SHALL be fixed among the stars, trailing the sun so it rises after dusk, crosses the night sky and is still setting in the west when the game opens at sunrise; by day it SHALL be pale. Stars SHALL be seeded from the world seed, turn about the celestial pole with solar time and fade in as the sun sets. Clouds SHALL drift and ridge rings SHALL repeat one seeded waveform with diminishing echoed amplitudes. Exact Godot (wait-for-stubs) positions and colours are a reference, not a requirement.

#### Scenario: Night sky travels coherently
- **WHEN** 225 simulation seconds of night pass
- **THEN** the moon moves more than 20 degrees of arc and its hour angle advances 45 degrees, as the stars do

#### Scenario: The opening moon
- **WHEN** the game opens at sunrise
- **THEN** the moon is low in the west, above the horizon, and the sun is rising in the east

### Requirement: Environment lighting follows visible time
Environment-map illumination and visible sky SHALL transition continuously with day/night and authored lighting mood. A once-baked environment map SHALL NOT leave lighting stuck at its initial time. The environment map SHALL be filtered again when the sun or moon has moved about a degree since its last filtering, not every frame.

#### Scenario: Environment lighting follows visible time
- **WHEN** a scene passes through dusk and dawn
- **THEN** sky, direct light, ambient illumination and fog agree without abrupt cuts or an unbounded per-frame bake

#### Scenario: A still sky is not refiltered
- **WHEN** the sun and moon have moved less than a degree since the last filtering
- **THEN** the environment map is not filtered again

### Requirement: Simulation controls remain separate
Speed controls SHALL be ignored while screens own controls, expose solar HH:MM, and cap each frame advance at 0.1 real seconds times speed. Movement and UI SHALL retain real-time speed.

#### Scenario: Simulation controls remain separate
- **WHEN** a two-second hitch occurs at 64x
- **THEN** calendar advances at most 6.4 seconds and night-boundary events remain unique

### Requirement: Editor preview lights stay out of the game
Lights the level marks `HiddenInGame` SHALL NOT light the game, including under the editor's embedded Play, because the calendar owns the sun and moon.

#### Scenario: Preview sun under Play
- **WHEN** the level's preview "Desert sun" is loaded under embedded Play at midnight
- **THEN** it is hidden and the night sky is lit only by the moon
