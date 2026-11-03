## ADDED Requirements

### Requirement: Stream build mode
Q SHALL enter stream build mode when no screen is open. In build mode, aiming within 3 m
of a free port SHALL light that port's halo, and left click SHALL anchor the plan there.
Right click SHALL drop the anchor; Escape or Q SHALL leave build mode.

#### Scenario: Anchoring at the burbler
- **WHEN** the player presses Q and aims at the ground by a burbler's mouth
- **THEN** the mouth's halo appears, and a left click anchors a planned stream there

### Requirement: Planned streams curve from their port
An anchored plan SHALL leave its port along the port's flow direction and curve smoothly
to the aim point, as a whole number of 4 m segments shown as a hologram whose flow
chevrons scroll downstream. A plan that bends tighter than 2 m radius SHALL be invalid.

#### Scenario: Planning a curve
- **WHEN** the player anchors at a port and aims 10 m away to its side
- **THEN** a hologram of 3 segments curves from the port to the aim point, chevrons moving downstream

### Requirement: Streams cost reeds
Each segment SHALL cost the `stream_segment` buildable's cost (6 reeds). While planning,
the reed icon and `need / have` SHALL show under the reticle, red when the player has
too few, and the hologram SHALL show invalid. Laying SHALL spend the reeds all or
nothing; an invalid plan SHALL not be laid.

#### Scenario: Not enough reeds
- **WHEN** a plan of 3 segments needs 18 reeds and the player holds 12
- **THEN** the cost reads 18 / 12 in red, the hologram is invalid, and clicking lays nothing

#### Scenario: Laying spends reeds and chains
- **WHEN** the player holds 24 reeds and lays a valid 2-segment plan
- **THEN** 12 reeds are spent, the stream is laid, and its far end becomes the new anchor
