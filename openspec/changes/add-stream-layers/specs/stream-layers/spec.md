## ADDED Requirements

### Requirement: A stream accumulates layers downstream of their joins
A stream SHALL carry an ordered set of layers, each joined at a distance along the chain from its burbler. The burbler's heartbeat SHALL join at its mouth. A layer SHALL show on the staff only downstream of its join.

#### Scenario: The campfire's layer
- **WHEN** a campfire joined to a stream plays its sequencer on beats 2 and 4
- **THEN** notes on 2 and 4 appear on the staff at the join and ride downstream, and none appear upstream

### Requirement: A campfire snaps to a stream within 5 m
While a campfire is being placed, a stream within 5 m of the aim SHALL snap its hologram beside the stream, and the placed campfire SHALL be joined to the stream at the point nearest it.

#### Scenario: Placing by the stream
- **WHEN** the player aims a campfire 3 m from a laid stream
- **THEN** the hologram snaps beside the stream and the campfire joins it there

### Requirement: The campfire's rack inherits the stream's voice
A joined campfire's rack SHALL show the stream's voice as it arrives, its clock, VCO and volume, patched and fixed, so that the player adds only the sequencer and its patches.

#### Scenario: Opening a joined campfire
- **WHEN** the player opens a campfire joined to a burbler's stream
- **THEN** the burbler's clock, VCO and volume are in place and patched, and only the sequencer place is open

### Requirement: The join is visible
While a joined campfire's layer plays, a ghostly staff SHALL run from the join point to the fire and rise from the fire into the air.

#### Scenario: A playing fire
- **WHEN** the campfire's sequencer is running
- **THEN** a translucent staff runs from the stream to the fire and up from it
