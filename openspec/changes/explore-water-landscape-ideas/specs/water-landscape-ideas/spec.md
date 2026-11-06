## ADDED Requirements

### Requirement: The reference stays findable

The project SHALL keep a research note recording the source video's link, what it describes with approximate times, and that it was not checked against primary sources.

#### Scenario: Designing water mechanics

- **WHEN** someone starts work on digging, fluids, or sharing water
- **THEN** the related open changes point them to `docs/research/persian-water-systems.md`

### Requirement: Sources and ideas stay separate

The note SHALL distinguish what the source says from game ideas the project derived from it.

#### Scenario: Reading an idea

- **WHEN** a reader looks at a game idea in the note
- **THEN** it is clear the idea is the project's own and undecided, not a claim from the video

### Requirement: Ideas need a decision before they are built

An idea from the note SHALL NOT be implemented until the designer chooses it and it has its own OpenSpec change. That change SHALL resolve any conflict with existing rules, including that mines are infinite.

#### Scenario: Adopting an idea

- **WHEN** the designer chooses an idea
- **THEN** it gets its own change proposal, and this change records the decision

#### Scenario: An idea that conflicts with a rule

- **WHEN** an adopted idea contradicts an AGENTS.md rule
- **THEN** its proposal states the conflict, and the rule changes only by the designer's decision
