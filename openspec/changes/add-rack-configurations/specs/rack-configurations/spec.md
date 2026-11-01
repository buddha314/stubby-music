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

### Requirement: Every rack has a cable comb with the machine's spare cables
Every rack SHALL show a small cable comb on its frame (`CableComb`, `Comb_<n>` slots).
A machine's configuration SHALL list its spare cables (`configure.cables`), and they
SHALL hang on the comb, folded over a slot with both plugs down, until the player takes
one to patch. Unplugging both ends of a cable SHALL hang it back on the comb.

#### Scenario: Opening the burbler's rack
- **WHEN** the player opens an unpatched burbler's configuration screen
- **THEN** two cables, verdigris and rust, hang on the comb ready to be placed

#### Scenario: Taking a cable
- **WHEN** the player drags a plug off the comb onto a jack
- **THEN** that cable leaves the comb and its other end follows the pointer

### Requirement: Patch pods pulse while waiting for a cable
Each jack's halo (`Jack_<port>_Halo`) SHALL pulse, in the cable's colour, while a spare
cable that the machine's configuration assigns to that jack is still unpatched; other
jacks' halos SHALL rest, and a halo SHALL rest once its jack is patched.

#### Scenario: Inviting the patch
- **WHEN** a VCO and the volume are mounted and no cable is patched
- **THEN** the jacks the burbler's cables are for pulse, each in its cable's colour, and the rest stay dark

#### Scenario: Patched
- **WHEN** a cable joins two jacks
- **THEN** those jacks' halos rest
