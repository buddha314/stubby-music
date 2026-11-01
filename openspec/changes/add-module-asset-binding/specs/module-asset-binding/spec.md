## ADDED Requirements

### Requirement: Module profiles
Every module asset SHALL ship an `asset.json` profile, generated with its model, naming
the engine `module_kind` and listing the module's ports, parameters (range, taper,
knob rotation), lights, and any lamp and display. The game SHALL embed the profiles at
build time, so the game and the Jackdaw adapter read identical data.

#### Scenario: Shipped profiles load
- **WHEN** the game starts
- **THEN** the profiles for the square, saw, triangle and sine VCOs and the clock all parse

### Requirement: Placing a module adds its engine module
Spawning a module asset with a module ID and parameters SHALL add the matching
`stubby_audio` module with those parameters. Despawning it SHALL remove the engine
module. A module kind the engine does not have yet SHALL still be placed and shown,
with a warning that it makes no sound.

#### Scenario: Place a clock
- **WHEN** a clock asset is placed with `bpm` 120
- **THEN** a `Clock` module with 120 BPM is added to the engine under the placed module's ID

#### Scenario: Not yet in the engine
- **WHEN** a saw VCO asset is placed
- **THEN** the model appears and the game warns that `SawVco` has no engine module yet

### Requirement: Parameters drive the engine
Changing a placed module's parameter value SHALL send the change to the engine module.
Values that have not changed SHALL NOT be resent.

#### Scenario: Retempo a clock
- **WHEN** a placed clock's `bpm` changes from 120 to 240
- **THEN** the engine clock runs at 240 BPM

### Requirement: Lights keep time with the engine
A clock module's lamp and GATE light SHALL flash on the engine's sample-stamped ticks,
scheduled from each tick's predicted instant through the shared pulse bus, with the
profile's attack and flash envelope. The PHASE light SHALL ramp once per beat. Frame
timing SHALL NOT shift when a flash happens.

#### Scenario: Flashes one beat apart
- **WHEN** a placed clock runs at 120 BPM, then at 240 BPM
- **THEN** its flashes are scheduled 0.5 s apart, then 0.25 s apart (within 15 ms), with one flash per tick

#### Scenario: On screen
- **WHEN** the clock light is rendered in the running game
- **THEN** each flash peaks within 10 ms of the corresponding audio tick's predicted instant

### Requirement: Panel follows parameters
A module's display SHALL show its parameter value (the clock's BPM, right-aligned on
three seven-segment digits, clamped to 999), and each knob SHALL rotate to its
parameter's value using the profile's taper and rotation range.

#### Scenario: BPM readout
- **WHEN** a placed clock's `bpm` is 120
- **THEN** its display lights the segments for "120" and no others

#### Scenario: Knob taper
- **WHEN** a log-taper knob's value is the geometric mean of its range
- **THEN** the knob points straight up

### Requirement: Independent instances
Each placed module SHALL have its own copies of its glowing materials, so two placed
modules flash and glow independently.

#### Scenario: Two clocks
- **WHEN** two clocks are placed at different tempos
- **THEN** each lamp flashes on its own clock's ticks

### Requirement: Engine module IDs are allocated, not picked
Every system that adds engine modules SHALL take module IDs from one shared allocator,
so no two modules are given the same ID.

#### Scenario: No collisions
- **WHEN** module assets, lightning bugs, the audio demo and collection cues all add engine modules
- **THEN** every engine module ID is distinct and no `AddModule` is rejected as a duplicate

### Requirement: Optional live listening showcase
The module showcase SHALL patch its placed square VCO through an envelope and VCA driven by its placed clock when --module-demo is enabled. Normal startup SHALL retain silence until discovery.
#### Scenario: Hear the visible clock
- **WHEN** the live listening showcase runs in the game or Jackdaw Play
- **THEN** each audible square burst and the clock lamp follow the same engine clock ID and sample timestamp
#### Scenario: Remove a showcased module
- **WHEN** either source module is removed
- **THEN** the showcase envelope and VCA are removed without changing unrelated voices
