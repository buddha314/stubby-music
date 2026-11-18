## ADDED Requirements

### Requirement: Evidence-based FOSS replacement
The project SHALL evaluate an existing FOSS component before replacing substantial
custom infrastructure when a plausible candidate exists. The evaluation SHALL state
the current behavior, candidate license and compatibility, covered requirements,
uncovered requirements, integration cost, and a retain-or-adopt decision.

#### Scenario: Candidate does not meet behavior
- **WHEN** a candidate lacks a required behavior or cannot build against the project's
  engine and platform constraints
- **THEN** the project records the mismatch and keeps the smallest custom implementation
  needed for that behavior

#### Scenario: Candidate meets requirements
- **WHEN** a maintained FOSS candidate covers required behavior, satisfies dependency
  and licensing rules, and reduces ongoing project-owned code
- **THEN** the project adopts it behind a narrow adapter and preserves game-specific
  rules as data or adapter logic

### Requirement: Preserve domain guarantees
Replacing infrastructure SHALL preserve deterministic world generation, strict
data validation, safe canonical rules saves, and realtime audio-thread constraints
where those guarantees apply.

#### Scenario: Infrastructure replacement changes generated output
- **WHEN** a replacement changes seeded world output
- **THEN** the generator revision and compatibility behavior are explicitly updated

#### Scenario: Audio library work reaches the callback
- **WHEN** an adopted audio component processes sound on the realtime callback
- **THEN** it performs no unbounded geometry work, blocking synchronization, or
  allocation on that callback
