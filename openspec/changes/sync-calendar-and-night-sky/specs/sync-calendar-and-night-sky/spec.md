## ADDED Requirements

### Requirement: One solar-time mapping
Sky, direct light, fog and environment lighting SHALL use one calendar mapping: 2700 s day maps 06:00–18:00 and 900 s night maps 18:00–06:00. Opening time SHALL be authored data; the parity preset SHALL start at 3150 s, solar midnight, with dawn 450 s later. Musical timing SHALL remain sample timed.

#### Scenario: One solar-time mapping
- **WHEN** the parity preset runs at 64x
- **THEN** dawn arrives after approximately 7.03 real seconds while pitch and BPM remain unchanged

### Requirement: Night sky travels coherently
Moon and stars SHALL move with simulation solar time; the parity moon SHALL start at bearing 60 degrees and altitude 20 degrees, advancing 45 degrees of hour angle in three solar hours. Clouds SHALL drift and ridge rings SHALL repeat one seeded waveform with diminishing echoed amplitudes.

#### Scenario: Night sky travels coherently
- **WHEN** 225 simulation seconds of night pass
- **THEN** the moon moves more than 20 degrees of arc and its hour angle advances 45 degrees

### Requirement: Environment lighting follows visible time
Environment-map illumination and visible sky SHALL transition continuously with day/night and authored lighting mood. A once-baked environment map SHALL NOT leave lighting stuck at its initial time. The implementation SHALL document whether intensity-only modulation matches the intended visual result or requires measured scheduled rebakes/blends.

#### Scenario: Environment lighting follows visible time
- **WHEN** a scene passes through dusk and dawn
- **THEN** sky, direct light, ambient illumination and fog agree without abrupt cuts or an unbounded per-frame bake

### Requirement: Simulation controls remain separate
Speed controls SHALL be ignored while screens own controls, expose solar HH:MM, and cap each frame advance at 0.1 real seconds times speed. Movement and UI SHALL retain real-time speed.

#### Scenario: Simulation controls remain separate
- **WHEN** a two-second hitch occurs at 64x
- **THEN** calendar advances at most 6.4 seconds and night-boundary events remain unique
