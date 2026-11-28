## ADDED Requirements

### Requirement: A save reopens a place and time
A save SHALL record the scene, the world seed, whether the seeded layout runs, where the player's feet stand and which way they face, and the calendar time. Loading it SHALL open that scene with that seed and stand the player there at that time.

#### Scenario: Opening the City save
- **WHEN** the game starts with `--load=city_neighbourhood`
- **THEN** the City neighbourhood loads without the opening's seeded layout, and the player stands on its south promenade facing the canal at golden hour, without the main menu

### Requirement: Saves are versioned
A save SHALL carry a format version, and loading SHALL refuse a save whose version is newer than the game's.

#### Scenario: A newer save is refused
- **WHEN** a save's version is greater than the game's
- **THEN** loading reports it and the game does not start in that save

### Requirement: Continue and Save Game
When a save exists, the main menu SHALL offer Continue, which opens the newest save. The pause menu SHALL offer Save Game, which writes the current place and time to the player's quicksave slot.

#### Scenario: Saving then continuing
- **WHEN** the player saves from the pause menu, quits and chooses Continue
- **THEN** the game opens in the same scene, standing and facing where they saved
