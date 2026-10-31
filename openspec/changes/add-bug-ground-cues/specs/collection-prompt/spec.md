## ADDED Requirements
### Requirement: Collection popup
The HUD SHALL show an E collection popup below the reticle when an uncollected bug is within the collection radius.
#### Scenario: Enter reach
- **WHEN** the player enters the 3 metre horizontal collection radius of an uncollected bug
- **THEN** the E collection popup becomes visible even between flashes
#### Scenario: Leave or collect
- **WHEN** no uncollected bug remains within reach
- **THEN** the popup disappears

### Requirement: Ground-distance collection
Collection and its prompts SHALL measure distance in the horizontal ground plane, ignoring bug height.
#### Scenario: Hovering bug within ground reach
- **WHEN** the bug is no more than 3 metres away horizontally, even at 3 metres above the terrain
- **THEN** E can collect it and the prompt is visible
