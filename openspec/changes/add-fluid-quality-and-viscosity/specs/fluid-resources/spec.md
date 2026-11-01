## ADDED Requirements
### Requirement: Initial muddy-water resource
Muddy water SHALL be available as an initial raw fluid resource for stream transport and later cleaning.
#### Scenario: Raw water
- **WHEN** an initial muddy-water source supplies a stream
- **THEN** the transported resource retains its muddy-water identity and quality state until a defined process changes it
### Requirement: Fluid viscosity
Every fluid SHALL have a finite positive viscosity value or a validated data-defined curve that produces one. Viscosity SHALL affect transport independently of visual appearance.
#### Scenario: Less viscous fluid
- **WHEN** two supplied streams have identical geometry, driving conditions and available capacity, but one carries lower-viscosity fluid
- **THEN** that fluid moves faster and supports greater potential volumetric flow
#### Scenario: Invalid fluid definition
- **WHEN** a fluid definition produces zero, negative or non-finite viscosity
- **THEN** validation rejects it
### Requirement: Water cleaning progression
Cleaning muddy water SHALL improve water quality, provide more usable nutrients, reduce effective viscosity and enable faster stream transport.
#### Scenario: Cleaned water
- **WHEN** a declared cleaning process transforms muddy water
- **THEN** its output has better water quality, greater usable nutrient provision and lower viscosity than its input
#### Scenario: Clean stream
- **WHEN** cleaned output flows through an otherwise equivalent supplied stream
- **THEN** it travels faster than the uncleaned input, subject to declared capacity limits
### Requirement: Consistent stream transport
Stream transport SHALL use deterministic simulation state and consistent quantity, speed and flow units. Transfers SHALL conserve transported volume and respect available supply and downstream capacity.
#### Scenario: Junction transfer
- **WHEN** a stream transfers water to one or more downstream segments
- **THEN** source quantity decreases by exactly the total amount the destinations receive and no destination exceeds capacity
### Requirement: Productive water delivery
Improved delivered water quality and nutrient provision SHALL support greater resource-mine productivity, including scrub-oak lightning-bug production, with light and other production conditions held constant.
#### Scenario: Better supply to an oak
- **WHEN** a scrub-oak mine receives improved water quality and useful nutrient delivery with light quality held constant
- **THEN** it supports greater lightning-bug production over an equivalent interval
