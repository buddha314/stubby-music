## ADDED Requirements

### Requirement: Design parity is distinct from shipped parity
The parity ledger SHALL mark proposed-only, partially implemented and implemented source capabilities separately. No unbuilt acoustic, weather, UI or module-library design SHALL be reported as a shipped feature.

#### Scenario: Design parity is distinct from shipped parity
- **WHEN** an upstream task remains unchecked and has no implementation evidence
- **THEN** the ledger records a design backlog rather than completed parity

### Requirement: Preserve native architecture when adopting designs
Adoption SHALL retain Firewheel/fundsp, existing f32 signal conventions and visible-rack-only patches. Godot C++ hosting and VCV voltage scaling SHALL NOT be silently substituted for the existing Bevy audio contracts. The 1 BPM master clock on the engine's sample counter is adopted by user decision (2026-11-16): the day cycle runs on it (`sync-calendar-and-night-sky`); making every tempo clock a phase-locked whole multiple of it needs its own change.

#### Scenario: Preserve native architecture when adopting designs
- **WHEN** a module or clock design is selected for implementation
- **THEN** a dedicated proposal records compatibility, native reuse, licensing and offline validation before code

### Requirement: Future design work has explicit gates
Acoustic profiles SHALL distinguish physics, simplified simulation and gameplay abstraction, with sound provenance and silent-opening rules. Weather SHALL remain silent until voiced and use sample-timed musical events. Root-inlet, local air-quality, icon HUD and module-library work SHALL each require feasibility evidence and a scoped implementation proposal.

#### Scenario: Future design work has explicit gates
- **WHEN** a planned feature moves to implementation
- **THEN** its owning change includes accepted behavior, reuse decisions, measurements and test scenarios
