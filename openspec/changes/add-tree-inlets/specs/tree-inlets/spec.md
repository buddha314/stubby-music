## ADDED Requirements

### Requirement: Streams snap alongside a tree
While laying a stream, aiming at a scrub oak's trunk or within the snap distance of its inlet ring SHALL snap the planned stream's end to the ring point nearest the aim, arriving along the ring's tangent, so that the stream runs alongside the tree and never through its trunk.

#### Scenario: Aiming at the trunk
- **WHEN** the player lays a stream and aims at an oak's trunk
- **THEN** the hologram ends on the oak's ring, beside the trunk, running along it

#### Scenario: Aiming away from the tree
- **WHEN** the aim is farther than the snap distance from every inlet ring
- **THEN** the stream ends at the aim, as before

### Requirement: A snapped stream waters its tree
An oak's inlet ring SHALL lie inside its watering reach, so that a flowing stream laid to it waters the tree. Inlets SHALL be a contract that names no asset kind, and the ring's size SHALL be data on the owner's profile.

#### Scenario: Watering from the inlet
- **WHEN** a flowing stream is laid snapped to an oak's inlet
- **THEN** the oak is watered
