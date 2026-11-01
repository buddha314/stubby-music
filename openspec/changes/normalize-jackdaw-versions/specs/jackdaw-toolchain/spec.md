## ADDED Requirements

### Requirement: Jackdaw-led compatibility set
The game and editor adapter SHALL use one Cargo workspace and lockfile, one Bevy
version/source matching the selected Jackdaw revision, and Jackdaw's Rust toolchain.
Audio and recipe domain crates MUST remain independent of Bevy.

#### Scenario: Building either entry point
- **WHEN** a developer builds the standalone game or Jackdaw adapter
- **THEN** both resolve the same Bevy version/source from the committed workspace lockfile

### Requirement: Explicit shared source identity
The project SHALL record the exact shared Jackdaw source revision, upstream base,
Bevy version/source and compiler, and SHALL validate them before managed build/launch.
Validation MUST be read-only and MUST explain mismatches without rewriting files.

#### Scenario: Checkout drift
- **WHEN** the shared Jackdaw source differs from the recorded revision
- **THEN** validation fails with the expected and actual revision and an upgrade instruction

#### Scenario: Dependency drift
- **WHEN** the workspace lockfile or Bevy source differs from the compatibility record
- **THEN** validation fails before managed launch

### Requirement: Preserve fork fixes during upstream integration
Incoming upstream changes SHALL be merged on an isolated sync branch from the
fork's tested supported branch. Shared branch history MUST NOT be rewritten to synchronize.
The integration SHALL retain required local fixes and their regression coverage.

#### Scenario: Fork divergence
- **WHEN** upstream and the fork each contain unique commits
- **THEN** synchronization preserves both histories and tests the resulting merge

### Requirement: Validated editor promotion
A candidate editor SHALL pass the smallest relevant build or regression check and
project version validation before promotion. Promotion MUST preserve a known-good
revision and MUST NOT modify an active or dirty shared checkout.

#### Scenario: Candidate fails
- **WHEN** a required candidate check fails
- **THEN** the shared active editor remains on its prior revision and the failure is recorded

#### Scenario: Graphical verification
- **WHEN** a graphical diagnostic is performed
- **THEN** the existing independent watchdog stops it within 20 seconds
