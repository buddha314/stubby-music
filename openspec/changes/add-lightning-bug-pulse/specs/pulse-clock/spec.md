## ADDED Requirements
### Requirement: Seeded random sample clock
A RandomClock SHALL choose each interval from a configured finite range using its explicit seed, emit ClockTick with the exact sample timestamp, and output a 5 ms trigger. It SHALL begin with a resting interval. Its sequence SHALL be independent of callback block size and frame rate. The callback SHALL allocate nothing, block on nothing and never panic.
#### Scenario: Repeatable irregular pulse
- **WHEN** two offline renders use the same seed, sample rate and interval bounds with different callback sizes
- **THEN** their tick timestamps are identical, intervals remain within the configured range and intervals vary
### Requirement: Shareable pulse timing
The game SHALL broadcast timestamped pulses through one shared clock boundary. Followers SHALL evaluate a common flash envelope from the tick's predicted instant. The clock gate SHALL NOT connect directly to the master output; it MAY drive existing sound modules.
#### Scenario: Two followers synchronize
- **WHEN** two assets follow the same clock and receive a tick
- **THEN** their envelopes use the same timestamp and agree at any display instant
- **THEN** no frame timer independently determines their pulse times

### Requirement: Clock abstraction and independent instances
Regular and random clocks SHALL implement one common sample-stepping clock interface. Each lightning bug SHALL own a separately seeded RandomClock module; bugs SHALL NOT share a clock by default. Followers MAY explicitly select the same clock to synchronize.
#### Scenario: Independent bugs
- **WHEN** three lightning bugs are created with a world seed
- **THEN** each has a distinct clock ID and reproducible random sequence
- **THEN** changing one clock's followers does not alter another clock's state

### Requirement: Click voice uses existing modules
Each bug SHALL make a short audio-rate click from the existing square VCO, envelope and VCA modules, scheduled by its random gate. The click SHALL be silent outside the bug's hearing range and the initial spawn SHALL be outside all bug hearing ranges. No custom DSP replacement SHALL be added for this patch.
#### Scenario: Resource click
- **WHEN** a bug pulses while the listener is in hearing range
- **THEN** its audio-rate square-wave click and white flash share the same sample-clock trigger
- **THEN** the click decays rapidly instead of becoming a sustained tone
