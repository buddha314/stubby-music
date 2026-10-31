## ADDED Requirements

### Requirement: Per-sample processing with one-sample cables
The rack SHALL run every module once per sample. Every cable SHALL deliver the value
its source output had on the previous sample, so the result does not depend on the
order modules run in.

#### Scenario: Cable delay
- **WHEN** a clock's gate reaches a module directly and also through one utility module
- **THEN** the path through the utility arrives exactly one sample later than the direct path

#### Scenario: Order independence
- **WHEN** the same patch is built with its modules added in two different orders
- **THEN** both render bit-identical output

### Requirement: Feedback patches are valid
The rack SHALL accept any cable topology, including loops and a module patched into
itself. It SHALL NOT reject a patch for containing a cycle.

#### Scenario: Self-modulating oscillator
- **WHEN** a VCO's output is cabled through a VCA back into its own FM input
- **THEN** the cable is accepted and the output stays finite

### Requirement: VCV voltage standards
All ports SHALL carry `f32` volts at audio rate, following VCV Rack's standards:
audio ±5 V; unipolar CV 0–10 V; bipolar CV ±5 V; pitch 1 V/oct with f = f₀·2^V,
where f₀ = 261.6256 Hz for oscillators and 2 Hz (120 BPM) for clocks and LFOs; gates
10 V; triggers 10 V for 1 ms. Gate and trigger inputs SHALL use a Schmitt trigger
(low 0.1 V, high 1.0 V). Modules SHALL NOT clamp outputs to ±1 V, and SHALL output
0 V in place of NaN or infinity. At an emission stage, ±10 V SHALL map to full scale.

#### Scenario: Pitch
- **WHEN** an oscillator with no frequency offset receives +1 V at its V/oct input
- **THEN** it oscillates at 523.25 Hz (±0.1 Hz)

#### Scenario: Schmitt trigger
- **WHEN** a gate input sees a signal rising slowly from 0 V to 0.9 V and back
- **THEN** no trigger fires, and one fires as soon as the signal reaches 1.0 V

#### Scenario: Non-finite output
- **WHEN** a module's computation produces NaN
- **THEN** its output port carries 0 V

### Requirement: Polyphony-ready cables
A cable SHALL carry a channel count of 1 to 16 and that many values, so polyphonic
modules can be added later without changing the cable type. Monophonic modules SHALL
read channel 0 and write one channel.

#### Scenario: Mono default
- **WHEN** a monophonic output is cabled to an input
- **THEN** the input reports a channel count of 1

### Requirement: Deterministic rendering
The rack SHALL render bit-identical output for the same patch, parameters and seed.

#### Scenario: Repeat render
- **WHEN** a 200-module patch is rendered offline twice
- **THEN** both renders are identical

### Requirement: Hosted in the game-audio callback
The rack SHALL render inside a single Firewheel node in the same audio callback as
all other game audio, so rack time and the audio clock are the same clock. Its
outputs SHALL be emission stems, one per emitting module.

#### Scenario: Shared clock
- **WHEN** a rack clock ticks at rack sample S
- **THEN** the tick event's `sample_time` equals S on the engine's audio clock

### Requirement: Realtime safety and budget
Rack rendering SHALL NOT allocate, lock, or panic. On the reference machine, a
200-module patch SHALL render a 256-frame block at 48 kHz in under 15% of the block's
duration, measured as a mean over 1,000 blocks.

#### Scenario: Budget
- **WHEN** the 200-module benchmark patch renders 1,000 blocks offline under an allocation guard
- **THEN** no allocation occurs and the mean block time is under 0.8 ms
