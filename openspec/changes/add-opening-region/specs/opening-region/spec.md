## ADDED Requirements

### Requirement: Open-ended region
The opening region SHALL have no goal, completion state or ending. Its systems SHALL
keep running for as long as the game runs, and the procedural world SHALL continue
past the region's edge.

#### Scenario: After the first bloom
- **WHEN** the first scrub oak in the region reaches full bloom
- **THEN** nothing ends: the oak keeps producing bugs and the player keeps building

### Requirement: Starting conditions
A new game SHALL start in a silent windstorm, in winter, with light quality at 0 and
every scrub oak in the region dormant.

#### Scenario: New game
- **WHEN** a new game starts
- **THEN** no sound is audible at spawn, every oak is dormant and props show their bleak look

### Requirement: Region contents
The opening region SHALL contain one mudpit burbler, a grove of dormant scrub oaks,
loose lightning bugs near the grove, reed beds on wet ground, and scattered desert
props, laid out by placing Jackdaw prefabs in the opening scene.

#### Scenario: Same seed, same region
- **WHEN** the same world is generated twice from one seed
- **THEN** the region's burbler, oaks, bugs, reeds and props are identical in both

### Requirement: Playable placement for every seed
Placement SHALL keep the opening playable for any seed. Spawn SHALL be outside every
bug's hearing range. The nearest oak SHALL be 20–40 m from the burbler. Reed clumps
within 40 m of the burbler SHALL hold at least enough reeds for a stream that connects
them.

#### Scenario: Silent start
- **WHEN** a world is generated with any seed
- **THEN** no lightning bug lies within 35 m of spawn

#### Scenario: First stream is affordable
- **WHEN** a world is generated with any seed
- **THEN** reeds within 40 m of the burbler cover 6 reeds per segment of a stream to the nearest oak

### Requirement: The region's loop
The region SHALL let the player close this loop: bugs give square waves, a square wave
voices the burbler, the burbler's stream carries the music, clean water wakes the
oaks, and blooming oaks produce bugs. Growth SHALL raise light quality, so the region
blooms outward from where the music is.

#### Scenario: Closing the loop
- **WHEN** clean water reaches a dormant oak from a voiced burbler's stream
- **THEN** the oak escalates to bloom and its open blooms begin producing lightning bugs

### Requirement: Placed prefabs are live
Every world asset placed from a Jackdaw prefab SHALL behave in Play as its kind does
when the game spawns it: a placed lightning bug flashes and clicks on its clock and can
be collected, and a placed scrub oak is a mine.

#### Scenario: A placed bug
- **WHEN** a lightning bug prefab is placed in the opening scene and Play starts
- **THEN** it flashes and clicks on its own clock and can be collected with E

#### Scenario: Duplicate instance IDs
- **WHEN** two placed prefabs share an `instance` ID
- **THEN** the game reports the duplicate instead of silently merging their harvest state
