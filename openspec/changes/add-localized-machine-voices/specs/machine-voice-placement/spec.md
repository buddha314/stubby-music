## ADDED Requirements

### Requirement: A voice has a place in the world

A rack's voice SHALL be panned by where its sound source is relative to the player's camera, changing smoothly as either moves.

#### Scenario: Turning away from a burbler

- **WHEN** a burbler is sounding to the player's left and the player turns to put it on their right
- **THEN** the voice moves from the left ear to the right ear without a click

#### Scenario: Removing a voice

- **WHEN** a cable is unplugged and the rack rebuilds its voice
- **THEN** the engine accepts any position update for the old output and raises no error

### Requirement: The stream carries the voice

A voice's source SHALL be whichever is nearest the player: the machine, or any point of the streams laid from it, including streams chained from those streams.

#### Scenario: Standing at the end of a stream

- **WHEN** the player stands by the far end of a burbler's stream
- **THEN** the voice is heard from the end of the stream, not from the burbler

#### Scenario: A burbler with no stream

- **WHEN** the machine has no stream
- **THEN** the voice is heard from the machine

### Requirement: A voice has a hearing range

A voice SHALL fade with distance from its source and SHALL be silent at the `hearing_radius_metres` in the machine's `configure` block.

#### Scenario: Walking away

- **WHEN** the player walks away from a sounding burbler and its streams
- **THEN** the voice gets quieter and is silent beyond the radius

### Requirement: Placement adds nothing to the patch

Placing a voice SHALL NOT add a module the player does not see in the rack or a connection the player did not make.

#### Scenario: An unpatched burbler

- **WHEN** a burbler's rack is not fully patched
- **THEN** it stays silent wherever the player stands
