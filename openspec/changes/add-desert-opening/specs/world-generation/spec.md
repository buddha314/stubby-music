## ADDED Requirements

### Requirement: Deterministic seeded generation
World generation SHALL be a pure function of a `u64` seed and chunk coordinate, using
seeded RNG only. The same seed SHALL produce identical terrain and placements on
every run and machine.

#### Scenario: Reproducible chunk
- **WHEN** chunk (3, -2) is generated twice with seed 42
- **THEN** the vertex positions, crack layout, and placements are bit-identical

#### Scenario: Different seeds differ
- **WHEN** chunk (0, 0) is generated with seeds 42 and 43
- **THEN** the crack layouts differ

### Requirement: Cracked mudflat terrain
The terrain SHALL be a near-flat dry lakebed with a polygonal mud-crack pattern
(Voronoi-style cells with raised, curled edges), plus low-frequency undulation that
stays within ±0.5 m of the base plane across the starting area.

#### Scenario: Flat start
- **WHEN** the spawn chunk and its 8 neighbours are generated
- **THEN** no point deviates more than 0.5 m from the base plane

### Requirement: Chunk streaming
Terrain SHALL be generated in fixed-size chunks around the player, off the main
thread, and despawned beyond a radius, with no seams between neighbouring chunks.

#### Scenario: Seamless border
- **WHEN** two adjacent chunks are generated
- **THEN** their shared edge vertices have identical positions and normals

### Requirement: Placement of discoverables
Generation SHALL place the Square VCO within a bounded distance of spawn and the Clock
further out, at seed-determined positions guaranteed to be reachable on foot.

#### Scenario: Square placement
- **WHEN** a world is generated with any seed
- **THEN** the Square VCO lies between 40 m and 120 m from spawn, outside its 35 m hearing radius

#### Scenario: Clock placement
- **WHEN** a world is generated with any seed
- **THEN** the Clock lies farther from spawn than the Square VCO, and within 300 m
