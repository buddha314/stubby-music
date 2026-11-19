## ADDED Requirements

### Requirement: Every being is described by the same controlled traits

The being atlas SHALL hold one row per essence the game may use (spirit, creature or monster), with its biomes, status and a value set for each facet of a controlled vocabulary defined in the atlas. Validation SHALL fail on an unknown facet value, kind, status or biome, or a duplicate ID.

#### Scenario: A value outside the vocabulary

- **WHEN** a row lists `sound: ["kazoo"]` and the vocabulary has no `kazoo`
- **THEN** `uv run tools/beings/cluster.py --check` fails, naming the being and the value

### Requirement: Beings are clustered by traits, not by biome

The analysis SHALL group beings by trait distance alone and SHALL report how the groups compare with the beings' biomes, each biome's signature traits and each being's nearest neighbour.

#### Scenario: Reading the report

- **WHEN** the analysis runs on the atlas
- **THEN** `report.md` gives the clusters, their agreement with biome, and each biome's over-represented traits, and biome was not an input to the clustering

### Requirement: The layout is deterministic

The analysis SHALL write a 2D layout of the beings, keeping similar beings close, that is identical for the same atlas and pinned dependencies.

#### Scenario: Running twice

- **WHEN** the analysis runs twice on an unchanged atlas
- **THEN** `layout.json` is byte-identical
