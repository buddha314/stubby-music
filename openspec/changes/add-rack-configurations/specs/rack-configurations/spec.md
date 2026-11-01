## ADDED Requirements

### Requirement: Every machine has a rack
Every configurable machine SHALL be configured on a rack built from the same common
rack: the same screen, slots, mounting and patching, opened with E within range.
Each machine SHALL define its own rack configuration as data.

#### Scenario: Two machines, one rack implementation
- **WHEN** a burbler and a later machine are each opened with E
- **THEN** both show a rack screen with their own title and slots, built by the same rack code

### Requirement: Configuration recipes lay out the rack
A machine's configuration recipe SHALL list its slots in order, each with a component
kind, a width in HP and whether it is fixed. The rack SHALL be as wide as its slots.

#### Scenario: The burbler's recipe
- **WHEN** a burbler's rack opens
- **THEN** it shows a fixed 8 HP clock, an 8 HP VCO slot and a 4 HP VCA slot, in that order

### Requirement: Labelled outlines guide construction
An empty slot SHALL show an outline of its width labelled with its component kind. A
component SHALL mount only in a slot of its kind, and a fixed slot SHALL never be empty.

#### Scenario: Empty VCO slot
- **WHEN** the burbler's VCO slot is empty
- **THEN** it shows an 8 HP outline labelled VCO with a square-wave glyph

#### Scenario: Wrong kind
- **WHEN** the player drags a VCA onto the VCO slot
- **THEN** it does not mount and the VCA stays in the inventory
