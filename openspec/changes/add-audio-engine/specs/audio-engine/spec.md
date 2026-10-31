## ADDED Requirements

### Requirement: Engine is independent of the game engine
The audio engine SHALL live in a crate that does not depend on `bevy`, and SHALL be
usable headlessly (offline rendering to a buffer) without an audio device.

#### Scenario: Crate builds without Bevy
- **WHEN** `cargo tree -p stubby_audio` is inspected
- **THEN** no `bevy` crate appears in its dependency tree

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
lock-free snapshots. Sending a command SHALL never block the caller; a full queue
SHALL return an error to the caller instead.

#### Scenario: Command applied
- **WHEN** the game sends `SetParam { module, param: "freq", value: 440.0 }`
- **THEN** the audio thread applies it within the next audio block

#### Scenario: Queue full
- **WHEN** the command queue is full and the game sends another command
- **THEN** the send returns an error immediately and the audio thread is unaffected

### Requirement: Realtime safety on the audio thread
Code on the audio thread SHALL NOT allocate, free, lock, perform I/O, or panic.
Debug builds MUST abort on audio-thread allocation.

#### Scenario: Allocation is detected
- **WHEN** a debug build performs a heap allocation inside the audio callback
- **THEN** the process aborts with an allocation diagnostic

#### Scenario: Removed nodes are freed elsewhere
- **WHEN** a module is removed from the patch
- **THEN** its DSP node is returned off the audio thread and dropped on a non-realtime thread

### Requirement: Glitch-free graph edits
Adding, removing, connecting, and disconnecting modules SHALL take effect at an audio
block boundary without clicks, dropouts, or interruption to unaffected modules.

#### Scenario: Add a module while playing
- **WHEN** a Sine VCO is already playing and the game adds a Clock module
- **THEN** the sine continues uninterrupted and the clock starts at the next block

### Requirement: Bounded work per block
The audio thread SHALL apply at most a fixed number of commands per audio block and
defer the rest to later blocks.

#### Scenario: Command flood
- **WHEN** the game sends 10,000 parameter changes in one frame
- **THEN** no underrun occurs and all changes are applied over subsequent blocks

### Requirement: Audio clock is the timing source of truth
The engine SHALL maintain a monotonic sample counter and SHALL publish timestamped
events (in samples) and a periodic `(sample_position, wall_instant)` sync point, so
the game can schedule visuals against audio time.

#### Scenario: Clock tick carries sample time
- **WHEN** a Clock module crosses a tick
- **THEN** the game receives an event whose sample timestamp equals the exact sample of the tick

### Requirement: Silent until sound is discovered
The engine SHALL output silence when the patch contains no sound-producing modules,
and the game SHALL NOT play any ambient or UI audio before the first oscillator is
discovered.

#### Scenario: Fresh game is silent
- **WHEN** a new game starts and no module has been discovered
- **THEN** the master output is digital silence
