## ADDED Requirements
### Requirement: Grounded movement
The player SHALL spawn standing on the terrain, move horizontally with WASD, sprint while holding Left Shift, jump on Space, and crouch while holding C.
#### Scenario: Spawn and walking
- **WHEN** the level starts or the player walks while looking down
- **THEN** the feet remain supported by the mudflat and pitch does not cause descent
#### Scenario: Jump and land
- **WHEN** Space is pressed while grounded
- **THEN** the player jumps under gravity and lands on the terrain without sinking or automatic repeated jumps
#### Scenario: Crouch
- **WHEN** C is held and released
- **THEN** eye height and movement speed lower and return without moving the feet
### Requirement: First-person look
Mouse movement SHALL turn yaw and pitch without a held mouse button while gameplay owns the cursor, with pitch limited to avoid flipping.
#### Scenario: Look and cursor release
- **WHEN** the player moves the mouse during gameplay
- **THEN** the view turns without RMB
#### Scenario: Focus and capture
- **WHEN** Escape is pressed or the window loses focus
- **THEN** the cursor is released and movement stops until the focused game is clicked again
