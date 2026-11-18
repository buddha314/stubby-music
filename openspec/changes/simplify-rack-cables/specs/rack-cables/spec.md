## ADDED Requirements

### Requirement: A rack offers only the places its machine needs
A machine's rack SHALL offer only the places its work needs. The burbler's rack SHALL be its clock, a VCO place and its volume, with no sequencer place; a machine whose work needs a sequencer (the campfire) SHALL offer one.

#### Scenario: The burbler
- **WHEN** the player opens a burbler's rack holding a snail shell
- **THEN** there is no place to mount it

### Requirement: Hovering a patched cable lights it
Hovering a patched cable SHALL light it from end to end, so the player can see where it starts and where it goes; it SHALL stop when the pointer leaves.

#### Scenario: Tracing a cable
- **WHEN** the pointer rests on a patched cable
- **THEN** that cable glows along its length and the others do not

### Requirement: A known patch guides its other end
While a cable's first end is in one end of an unmade suggested patch, the screen SHALL show only that patch's other jack, blinking, and the cable SHALL take the patch's colour.

#### Scenario: Patching GATE to TRIG
- **WHEN** the player plugs a cable's first end into the burbler clock's GATE
- **THEN** the volume's TRIG blinks and no other jack pulses
