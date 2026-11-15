## ADDED Requirements

### Requirement: One definition per rack module
Every module that can sit in a rack SHALL have exactly one module definition,
`assets/modules/<id>/module.json`, naming the engine parts it is built from, its
internal wiring, the engine port behind each panel jack, the engine parameter behind
each knob, and each knob's default. Game code SHALL read a module's engine behaviour
only from its definition, never from code keyed by the module's ID.

#### Scenario: The volume, from data
- **WHEN** the burbler's rack builds its volume module
- **THEN** it adds one `Vca` engine part whose `cv` input is the panel's TRIG jack, as `volume/module.json` says

#### Scenario: A new module needs no code
- **WHEN** a designer adds `module.json` for a module whose engine parts already exist and mounts it in a rack configuration
- **THEN** the module patches and sounds without any change to Rust code

### Requirement: Definitions are validated strictly
Loading module definitions SHALL be strict: an unknown field, an unknown engine part
kind, an engine port or parameter the part does not have, or a duplicate module ID
SHALL fail. Loading SHALL report every error at once, each with the file, module ID
and field.

#### Scenario: Two mistakes, one report
- **WHEN** a definition binds a jack to `vco.outt` and gives a knob the unknown default source `bug_tempo`
- **THEN** loading fails and the report names both errors with the file, module and field

### Requirement: Definitions agree with the panel
Each definition SHALL be cross-checked against the module's `asset.json`: every panel
port SHALL have a jack binding and every panel knob a parameter binding, and none
SHALL name a port or knob the panel lacks. A jack's direction SHALL come from the
panel profile and SHALL match the direction of the engine port it binds.

#### Scenario: Panel and definition drift
- **WHEN** the panel has a V/OCT jack that `module.json` does not bind
- **THEN** validation fails naming the module and the `voct` port

#### Scenario: Output bound to an input
- **WHEN** a definition binds the panel's OUT jack (an output) to the VCA's `in` port
- **THEN** validation fails naming the jack and the mismatched direction

### Requirement: Knob defaults are numbers or named sources
A knob's default SHALL be either a number within the knob's range or a named default
source from a closed list (initially `machine_clock_bpm`, the machine's seeded tempo,
and `lightning_bug_pitch`, the bug click's frequency). A new source SHALL be a rule
change, not a data change.

#### Scenario: The burbler clock starts at its seeded tempo
- **WHEN** a burbler with a seeded tempo of 96 BPM opens its rack for the first time
- **THEN** the clock's BPM knob reads 96, from the `machine_clock_bpm` source

#### Scenario: A mounted square VCO starts at the bug's pitch
- **WHEN** a Square VCO is mounted and its FREQ knob has never been turned
- **THEN** it plays at the lightning bug click's frequency

### Requirement: One source of defaults for every use
Every place a module is instantiated, the rack and the module showcase alike, SHALL
take its engine parts and starting parameters from the same definition.

#### Scenario: Rack and showcase agree
- **WHEN** the volume is built once on the burbler's rack and once in the module showcase
- **THEN** both start with the same gain

### Requirement: The patch plan is exactly the rack
A rack's planned engine patch SHALL contain only the parts of mounted modules, their declared internal wiring,
the patched cables, and the normalled output. It SHALL add nothing the panel does
not show.

#### Scenario: Partly patched
- **WHEN** only the clock GATE to volume TRIG cable is patched on the burbler
- **THEN** the plan holds the clock, VCO and VCA parts and that one connection, and nothing else

#### Scenario: Planning needs no Bevy
- **WHEN** the planner's tests run with `cargo test -p stubby_rack`
- **THEN** they build no Bevy app and open no audio device
