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

### Requirement: Master output
A reserved output module SHALL have one input that accepts any number of cables,
and SHALL send their sum to both speakers.

#### Scenario: Mono source reaches both speakers
- **WHEN** a Sine VCO's output is cabled to the master output
- **THEN** the left and right channels are identical

### Requirement: Constant module
The game SHALL provide a Constant module with a value parameter and one output
carrying that value, for use as a fixed CV source.

#### Scenario: Fixed pitch offset
- **WHEN** a Constant of 1.0 is cabled to a Sine VCO's V/oct input
- **THEN** the sine sounds one octave above its base frequency

### Requirement: Sine VCO module
The game SHALL provide a Sine VCO with a frequency parameter, a V/oct pitch input,
and a sine audio output.

#### Scenario: Pure tone
- **WHEN** a Sine VCO at 440 Hz is rendered offline for one second at 48 kHz
- **THEN** the dominant spectral peak is at 440 Hz and peak amplitude is 1.0 (±0.01)

### Requirement: Square VCO module
The game SHALL provide a band-limited Square VCO with the same frequency parameter,
V/oct input and output as the Sine VCO.

#### Scenario: Square tone
- **WHEN** a Square VCO at 220 Hz is rendered offline
- **THEN** its fundamental is 220 Hz (±0.1 Hz), its RMS is above 0.85 of its peak, and no sample exceeds 1.25

### Requirement: Envelope module
The game SHALL provide an attack/decay Envelope with a gate input and one output
from 0 to 1. Each rising gate edge SHALL start a linear attack to 1 from the current
level, so a retrigger never jumps. An exponential decay SHALL follow, reaching -60 dB
after the decay time. The attack and decay times SHALL be parameters.

#### Scenario: Shape
- **WHEN** a 2 ms / 200 ms envelope is gated
- **THEN** it reaches 1.0 (±0.02) after 2 ms and 0.001 (±20%) a further 200 ms later

#### Scenario: Retrigger
- **WHEN** the gate rises again before the decay has finished
- **THEN** the output climbs from its current level with no step larger than one attack increment

### Requirement: VCA module
The game SHALL provide a VCA whose output is its audio input times its CV (negative
CV treated as 0) times a gain parameter. With no CV cabled it SHALL be closed. Gain
changes SHALL be smoothed so they do not zipper. As an audio source, it SHALL fade
on add and remove.

#### Scenario: Scaling
- **WHEN** a full-scale sine passes through a VCA with CV 0.5 and gain 0.5
- **THEN** the output peaks at 0.25

#### Scenario: Clock blip
- **WHEN** a Clock gates an Envelope that drives a VCA on an oscillator
- **THEN** a blip sounds starting at each tick's sample, with a click-free onset, and dies away before the next tick

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
