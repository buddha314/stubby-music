## ADDED Requirements
### Requirement: Sunflower reticle
The game SHALL display a small sunflower outline reticle with an open center and contrasting edging at the center of the gameplay viewport.
#### Scenario: Gameplay view
- **WHEN** the game renders the player HUD
- **THEN** twelve outlined petals and an open center ring provide a centered aiming reference
#### Scenario: Viewport resize
- **WHEN** the viewport changes size or runs in Jackdaw Play
- **THEN** the reticle remains centered at a constant logical size
