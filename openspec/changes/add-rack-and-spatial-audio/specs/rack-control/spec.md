## ADDED Requirements

### Requirement: Smoothed parameters
Parameters SHALL be owned by the rack. A new value set by the game SHALL be reached
by a smoothing ramp of 5–20 ms rather than a step, so turning a knob never produces
zipper noise.

#### Scenario: Knob sweep
- **WHEN** a VCA's gain parameter jumps from 0 to full in one frame while a tone plays
- **THEN** the output reaches full level over 5–20 ms with no single-sample step

### Requirement: Lights, meters and scopes
Modules SHALL publish light, meter and scope values that the game reads every frame
without locks. Lights SHALL be smoothed in the rack. A scope SHALL expose a ring of
recent samples.

#### Scenario: Clock light
- **WHEN** a clock's gate is high
- **THEN** its gate light reads at or near full brightness on the next frame

#### Scenario: Scope
- **WHEN** the game reads a scope on a 440 Hz oscillator
- **THEN** the samples show a 440 Hz waveform

### Requirement: Cable edits
Connecting or disconnecting a cable SHALL take effect at a sample boundary. A cable
whose signal reaches an emitter as audio SHALL be faded in or out. A cable carrying
a gate or CV SHALL switch instantly, so gate edges are not moved.

#### Scenario: Pull an audio cable
- **WHEN** the cable from an oscillator to an emitting VCA is removed while it plays
- **THEN** the sound fades out without a click

#### Scenario: Patch a clock
- **WHEN** a clock's gate cable is connected to an envelope
- **THEN** the first gate edge reaches the envelope with no fade or delay beyond the 1-sample cable delay

### Requirement: Patches are data
A patch SHALL round-trip through a serialized form: module kinds, parameters,
cables, emitter settings and world positions. It serves save games, undo and test
fixtures.

#### Scenario: Round trip
- **WHEN** a patch is serialized, deserialized and rendered
- **THEN** its output is bit-identical to the original patch's
