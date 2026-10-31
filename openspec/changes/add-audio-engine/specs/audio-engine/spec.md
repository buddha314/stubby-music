## ADDED Requirements

### Requirement: Engine is independent of the game engine
The audio engine SHALL live in a crate that does not depend on the Bevy engine, and
SHALL be usable headlessly (offline rendering to a buffer) without an audio device.
Utility crates pulled in by the audio library (`bevy_platform`,
`bevy_macro_utils`) are permitted; ECS, app, render and asset crates are not.

#### Scenario: Crate builds without the Bevy engine
- **WHEN** `cargo tree -p stubby_audio` is inspected
- **THEN** none of `bevy`, `bevy_app`, `bevy_ecs`, `bevy_render`, or `bevy_asset` appears in its dependency tree

#### Scenario: Offline render
- **WHEN** a test builds a patch and requests N samples of offline rendering
- **THEN** the engine returns N samples without opening an audio device

### Requirement: DSP runs on the realtime audio thread
The engine SHALL render audio on the audio device's callback thread and SHALL NOT
render audio inside any Bevy system or schedule.

#### Scenario: Game frame hitch does not interrupt audio
- **WHEN** the Bevy main thread stalls for 200 ms while a patch is playing
- **THEN** audio output continues without underrun

### Requirement: Lock-free message boundary
The game SHALL communicate with the audio thread only through wait-free queues and
lock-free shared state. The game sends commands, then publishes them once per frame
with a flush. Neither call SHALL block; a full queue SHALL make the flush return an
error, with the edits kept for the next flush.

#### Scenario: Command applied
- **WHEN** the game sends `SetParam { module, param: "freq", value: 440.0 }` and flushes
- **THEN** the audio thread applies it within the next audio block

#### Scenario: Queue full
- **WHEN** the queue to the audio thread is full and the game flushes
- **THEN** the flush returns an error immediately and the audio thread is unaffected

### Requirement: Realtime safety on the audio thread
Code on the audio thread SHALL NOT allocate, free, lock, perform I/O, or panic.
The test suite MUST run every offline render under an allocation guard, and the
engine crate MUST deny `unwrap`, `expect` and `panic!`.

#### Scenario: Allocation is detected
- **WHEN** the audio code path allocates during an offline test render
- **THEN** the test fails with an allocation diagnostic

#### Scenario: Removed nodes are freed elsewhere
- **WHEN** a module is removed from the patch
- **THEN** its DSP node is returned off the audio thread and dropped on a non-realtime thread

### Requirement: Glitch-free graph edits
Adding and removing modules and changing parameters SHALL take effect at an audio
block boundary without clicks, dropouts, or interruption to unaffected modules.
Sound sources SHALL fade in when added and fade out before they are deleted.

#### Scenario: Add a module while playing
- **WHEN** a Sine VCO is already playing and the game adds a Clock module
- **THEN** the sine continues uninterrupted and the clock starts at the next block

#### Scenario: Remove a sound source while playing
- **WHEN** a playing Sine VCO is removed at an arbitrary frame
- **THEN** it fades out without a click and its node is deleted on a later flush

### Requirement: Bounded work per flush
Parameter changes sent within one frame SHALL coalesce so that only the last value
per parameter reaches the audio thread. Per-flush work is then bounded by the number
of distinct parameters, not by the number of sends.

#### Scenario: Command flood
- **WHEN** the game sends 10,000 changes to one parameter in one frame
- **THEN** the flush succeeds and the audio thread applies only the last value

### Requirement: Audio clock is the timing source of truth
The engine SHALL maintain a monotonic sample counter and SHALL publish timestamped
events (in samples) and a periodic `(sample_position, wall_instant)` sync point, so
the game can schedule visuals against audio time.

#### Scenario: Clock tick carries sample time
- **WHEN** a Clock module crosses a tick
- **THEN** the game receives an event whose sample timestamp equals the exact sample of the tick

### Requirement: Silent until sound is discovered
The engine SHALL output silence when nothing audible is in range. Before the first
oscillator is discovered, its own emission SHALL be the only sound in the game: no
ambient, UI or effects audio.

#### Scenario: Fresh engine is silent
- **WHEN** an engine starts with an empty patch
- **THEN** the master output is digital silence
