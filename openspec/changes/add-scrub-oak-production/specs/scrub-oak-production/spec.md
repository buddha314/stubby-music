## ADDED Requirements
### Requirement: Infinite scrub-oak mine
Scrub-oak trees SHALL be infinite mines producing exhaustible lightning-bug instances.
#### Scenario: Repeated production
- **WHEN** a tree completes successive production cycles
- **THEN** it produces fresh finite bug instances without consuming a finite mine reserve
#### Scenario: Collected offspring
- **WHEN** a generated bug is collected
- **THEN** only that bug is exhausted and its parent tree remains an infinite source
### Requirement: Environmental productivity
Scrub-oak bug production SHALL increase as water quality or light quality improves, with the other factor held constant.
#### Scenario: Better water
- **WHEN** water quality increases with light quality held constant
- **THEN** the tree produces more lightning bugs over an equivalent production interval
#### Scenario: Better light
- **WHEN** light quality increases with water quality held constant
- **THEN** the tree produces more lightning bugs over an equivalent production interval
