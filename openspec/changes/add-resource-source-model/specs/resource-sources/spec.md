## ADDED Requirements
### Requirement: Distinct source objects
The domain SHALL distinguish exhaustible resource instances from infinite mine instances with stable definition and instance IDs.
#### Scenario: Finite pickup
- **WHEN** an exhaustible lightning-bug object is successfully harvested
- **THEN** its configured item stacks are granted once and it becomes collected
#### Scenario: Infinite mine
- **WHEN** a scrub-oak mine object is created or inspected
- **THEN** it is classified as an infinite mine and has no finite depletion counter
#### Scenario: Collecting a mine
- **WHEN** the finite pickup API receives a mine object
- **THEN** it refuses the operation without granting items or modifying the mine
### Requirement: Mine catalog definitions
Mine definitions SHALL reference a known produced resource and declare applicable productivity factors. A scrub-oak mine SHALL produce lightning-bug resources.
#### Scenario: Scrub oak
- **WHEN** the shipped scrub-oak mine definition is loaded
- **THEN** its output is lightning_bug and its factors include water quality and light quality
#### Scenario: Invalid definition
- **WHEN** a mine references an unknown resource or repeats a productivity factor
- **THEN** catalog validation rejects it
### Requirement: Authorable mine instances
The game SHALL expose a reflected mine component separate from finite collection nodes.
#### Scenario: Mine authoring
- **WHEN** a scrub-oak MineNode is instantiated
- **THEN** it has a stable mine ID and instance ID and is not a finite pickup
