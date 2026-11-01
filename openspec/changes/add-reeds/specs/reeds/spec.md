## ADDED Requirements

### Requirement: Reeds are a frequent renewable resource
Reed clumps SHALL be harvestable resource instances that yield `reed` items when the
player uses E on them, and SHALL return after a short elapsed-time replenishment
interval, so reeds are renewed often.

#### Scenario: Harvesting a clump
- **WHEN** the player presses E at a standing reed clump
- **THEN** the player receives the clump's reed yield and the clump is cut to stubble

#### Scenario: Harvesting stubble
- **WHEN** the player presses E at a clump that is stubble
- **THEN** nothing is collected and no confirmation sound plays

#### Scenario: Growing back
- **WHEN** the clump's replenishment interval has elapsed since it was harvested
- **THEN** the clump is standing again and can be harvested

### Requirement: Reeds build stream segments
A stream segment SHALL cost 6 reeds. Laying a planned stream SHALL spend 6 reeds per
segment, and a laid segment SHALL be outlined by reeds along both banks.

#### Scenario: Laying a stream
- **WHEN** the player lays a planned stream of 3 segments holding 20 reeds
- **THEN** 18 reeds are spent and 2 remain

#### Scenario: Not enough reeds
- **WHEN** the planned stream needs more reeds than the player holds
- **THEN** the hologram shows invalid and nothing is laid

#### Scenario: Reed outline
- **WHEN** a stream segment is laid
- **THEN** reeds line both of its banks, dry while the water is muddy and green as it clears

### Requirement: Reeds are very easy to get
Reeds SHALL be gathered as easily as leaves in Satisfactory: clumps SHALL be abundant
on wet ground, and holding E SHALL collect successive nearby clumps.

#### Scenario: Gathering for a stream
- **WHEN** the player holds E while walking along a bank of standing clumps
- **THEN** each clump in reach is harvested once and its reeds are added

### Requirement: Reeds are common on wet ground
Reed clumps SHALL be placed far more often than lightning bugs, on wet ground such as
burbler pools and stream banks, deterministically from the world seed.

#### Scenario: Same seed, same reeds
- **WHEN** the same world is generated twice from one seed
- **THEN** the same reed clumps appear at the same places with the same variants

### Requirement: Reed clumps vary and show their state
Each clump SHALL be a seeded variant with per-instance variation, and SHALL show
whether it is standing, harvested or regrowing.

#### Scenario: Regrowing
- **WHEN** a harvested clump is partway through its replenishment interval
- **THEN** it shows stubble and, optionally, reeds partly grown back
