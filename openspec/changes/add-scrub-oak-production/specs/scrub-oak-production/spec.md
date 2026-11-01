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

### Requirement: Dormant until watered
A scrub oak SHALL start dormant, in winter conditions, and SHALL escalate to full
bloom as clean water is applied to it, through budding and leaf-out. Lightning bugs
SHALL emerge from its open blooms, so a dormant tree produces none.
#### Scenario: A new tree
- **WHEN** a scrub oak is first placed and has received no clean water
- **THEN** it is bare and dormant and produces no lightning bugs
#### Scenario: Watering it
- **WHEN** clean water is delivered to a dormant tree over time
- **THEN** it buds, leafs out and finally blooms, and bugs emerge from its open blooms
#### Scenario: Muddy water
- **WHEN** only muddy water reaches the tree
- **THEN** it stays dormant

### Requirement: Trees vary procedurally
No two scrub oaks SHALL look identical. Each tree SHALL be a seeded variant with
per-instance variation drawn deterministically from the world seed and its instance ID.
#### Scenario: Same seed, same tree
- **WHEN** the same world is generated twice from one seed
- **THEN** each scrub oak has the same variant and the same per-instance variation
