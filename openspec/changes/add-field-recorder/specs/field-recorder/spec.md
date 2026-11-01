## ADDED Requirements

### Requirement: The player holds the field recorder
During play the field recorder SHALL be shown held in the player's right hand at its
`Grip` node, at real size, its `Capture` node pointing forward.

#### Scenario: Playing
- **WHEN** the game is in play with the cursor captured
- **THEN** the recorder is visible in the lower right of the view, windguard forward

### Requirement: The recorder shows collection
While E collects, the recorder's `Light_rec` SHALL pulse and its `Level_<n>` bars SHALL
show activity; otherwise the light rests dark.

#### Scenario: Collecting a bug
- **WHEN** the player holds E within reach of a lightning bug
- **THEN** the record light pulses and the level bars move until the collection completes
