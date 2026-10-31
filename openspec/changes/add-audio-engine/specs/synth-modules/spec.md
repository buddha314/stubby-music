## ADDED Requirements

### Requirement: Module and port model
Every synth module SHALL declare named, typed input and output ports and named
parameters with ranges and defaults. A connection SHALL join exactly one output port
to one input port. An output MAY fan out to many inputs, and an input SHALL accept at
most one connection.

#### Scenario: Fan-out
- **WHEN** the Clock output is connected to two different modules' inputs
- **THEN** both receive the same signal

#### Scenario: Second cable to an input
- **WHEN** a connection is made to an input that is already connected
- **THEN** the engine rejects the connection with an error

### Requirement: Eurorack signal conventions
All ports SHALL carry `f32` signals at audio rate. Audio SHALL use a nominal ±1.0
range; pitch CV SHALL use 1.0 per octave (V/oct) relative to a module's base
frequency; gates and triggers SHALL be high when above 0.5.

#### Scenario: V/oct pitch
- **WHEN** a Sine VCO with base frequency 261.63 Hz receives pitch CV of 1.0
- **THEN** it oscillates at 523.25 Hz (±0.1 Hz)

### Requirement: Sine VCO module
The game SHALL provide a Sine VCO with a frequency parameter, a V/oct pitch input,
and a sine audio output.

#### Scenario: Pure tone
- **WHEN** a Sine VCO at 440 Hz is rendered offline for one second at 48 kHz
- **THEN** the dominant spectral peak is at 440 Hz and peak amplitude is 1.0 (±0.01)

### Requirement: Clock module
The game SHALL provide a Clock with a tempo parameter (BPM), a gate output, and a
phase output (0..1 ramp per beat). Each beat SHALL emit a timestamped tick event to
the game.

#### Scenario: Tempo accuracy
- **WHEN** a Clock at 120 BPM is rendered offline for 10 seconds at 48 kHz
- **THEN** exactly 20 ticks occur, spaced 24,000 samples apart

#### Scenario: Clock drives a visual pulse
- **WHEN** the clock emits a tick at sample S
- **THEN** the game's clock light peaks at the display time predicted for sample S
