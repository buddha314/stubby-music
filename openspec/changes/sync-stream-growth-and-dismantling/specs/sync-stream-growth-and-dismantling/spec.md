## ADDED Requirements

### Requirement: Dismantle refunds and breaks flow
F SHALL dismantle the aimed stream segment, refund its original build cost atomically, split connectivity and stop downstream flow across the removed segment. A full inventory SHALL leave segment and inventory unchanged.

#### Scenario: Dismantle refunds and breaks flow
- **WHEN** the middle of a four-segment connected stream is dismantled
- **THEN** upstream remains flowing, disconnected downstream lies quiet, and the removed segment cost is returned once

### Requirement: Water clarity caps growth
Growing assets SHALL define growth levels as data. Scrub oak SHALL reach budding 0.3 in mud, leafed 0.7 in silty water and full bloom 1.0 in clear water. Growth SHALL rise toward its water-supported cap and recede more slowly without water, using simulation time.

#### Scenario: Water clarity caps growth
- **WHEN** muddy flow waters a dormant oak long enough
- **THEN** it reaches budding and cannot reach full bloom

### Requirement: Watering reports real flow
The initial parity adapter SHALL water an oak within 3 m horizontally of a flowing stream using the clearest eligible stream; stopping flow SHALL report unwatering and preserve latched goal completion. Explicit root-inlet connections SHALL be a subsequent migration, not assumed implemented.

#### Scenario: Watering reports real flow
- **WHEN** the supplying burbler is unpatched
- **THEN** watering stops and growth recedes while a completed water_the_tree goal stays complete
