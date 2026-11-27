## ADDED Requirements

### Requirement: Players host and join
A player SHALL be able to host a session and others SHALL be able to join it, by address on a LAN and through a Steam invite, playing in the same world. A client of another build or protocol SHALL be refused with a message saying so.

#### Scenario: Join on a LAN
- **WHEN** one player hosts and another joins with the host's address
- **THEN** both stand in the same desert and see each other move

#### Scenario: Other build
- **WHEN** a client from a different build tries to join over Steam
- **THEN** it is refused, and its screen names both builds

### Requirement: Every peer has the same world
Peers SHALL generate the world from the host's world configuration, and SHALL agree on every seeded and authored entity without the world being sent.

#### Scenario: A client's world
- **WHEN** a client joins a host with seed 42
- **THEN** its terrain, placements and authored scenery match the host's

### Requirement: The host decides what changes
Collecting, building, crafting, machine configuration and inventory changes SHALL be applied by the host under the game's rules and replicated to every peer; a client's request the rules refuse SHALL change nothing.

#### Scenario: Two players collect the same thing
- **WHEN** two players collect the same source in the same moment
- **THEN** exactly one receives it, and both see the source taken

#### Scenario: Building without the cost
- **WHEN** a client asks to build a stream it cannot afford
- **THEN** nothing is built and its inventory is unchanged

### Requirement: Players move responsively
A player's own movement SHALL respond at once on their machine and stay consistent with the host, correcting smoothly when they disagree; other players SHALL move smoothly.

#### Scenario: A laggy link
- **WHEN** a client plays over a link with latency and loss
- **THEN** its own movement responds immediately and converges with the host's

### Requirement: Music plays together
Every peer SHALL hear the same machines with the same configurations, with their clocks following the host's so that bars line up.

#### Scenario: A configured machine
- **WHEN** the host configures a machine's rack
- **THEN** every client hears that machine play the same pattern, in time with the host
