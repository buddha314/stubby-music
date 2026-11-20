## ADDED Requirements

### Requirement: Rock openings and hollow trunks yield high-pass filters
Each opening in a rock formation and each hollow scrub oak trunk SHALL yield one
high-pass filter when collected with E, and SHALL NOT replenish.

#### Scenario: Collecting from an opening
- **WHEN** the player aims at a rock opening that has not been collected and presses E
- **THEN** one high-pass filter is added to the inventory and the opening's yield is gone

#### Scenario: Collected already
- **WHEN** the player presses E at an opening already collected
- **THEN** nothing is added and nothing sounds

#### Scenario: Two openings in one formation
- **WHEN** a formation has two openings
- **THEN** each opening yields its own filter under its own instance ID

### Requirement: Only some trunks are hollow
Whether a scrub oak's trunk is hollow SHALL be decided from the world seed and the oak's
instance ID, so the same seed always makes the same oaks hollow, and not every oak SHALL
be hollow.

#### Scenario: Seed 17
- **WHEN** the opening is generated twice from seed 17
- **THEN** the same oaks are hollow both times, and some oaks are not

### Requirement: An opening is heard high-passed
A source sounding within hearing of a rock opening or hollow trunk SHALL be heard through
it high-passed. An opening with nothing sounding near it SHALL be silent.

#### Scenario: Silent start
- **WHEN** nothing sounds near an opening
- **THEN** the opening makes no sound

#### Scenario: A burbler near an opening
- **WHEN** a voiced burbler sounds within hearing of a rock opening
- **THEN** the player near the opening hears its bright upper part, without its low fundamental
