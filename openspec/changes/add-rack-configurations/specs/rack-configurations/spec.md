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

### Requirement: Cables come from the inventory
Every cable on a rack SHALL be one of the player's patch cables, taken from the inventory
when its first end goes into a jack's patch and given back when it is unplugged; there
SHALL be no cable comb (user, 2026-11-18; `simplify-rack-cables`). A machine's
configuration SHALL list the patches its voice needs (`configure.cables`); they are only
what the screen suggests, and any cable SHALL join any output to any free input, as in
VCV Rack. A cable making a suggested patch SHALL take that patch's colour.

A cable SHALL start on a jack. Its far end SHALL snap to a jack it can go into when the
pointer comes within 40 screen pixels of it: its plug shows in that jack, and releasing
there SHALL plug it in, even off the jack itself.

#### Scenario: Opening the burbler's rack
- **WHEN** the player opens an unpatched burbler's configuration screen holding 10 patch cables
- **THEN** the patches to make show as ghosts, and no cables hang on the frame

#### Scenario: Patching takes a cable and unplugging gives it back
- **WHEN** the player patches the VCO's OUT to the volume's IN, then unplugs it
- **THEN** the inventory holds 9 patch cables while it is patched, and 10 again after

#### Scenario: Any cable, any patch
- **WHEN** the player patches GATE to TRIG and OUT to IN in either order
- **THEN** the voice is complete and sounds

#### Scenario: Snapping
- **WHEN** the player releases the cable in hand just beside a jack it can go into
- **THEN** it plugs into that jack

### Requirement: Patch pods pulse while waiting for a cable
Each jack's halo (`Jack_<port>_Halo`) SHALL pulse, in the patch's colour, while a patch
that the machine's configuration names for that jack is still unmade; other
jacks' halos SHALL rest, and a halo SHALL rest once its jack is patched.

#### Scenario: Inviting the patch
- **WHEN** a VCO and the volume are mounted and no cable is patched
- **THEN** the jacks the burbler's patches are for pulse, each in its patch's colour, and the rest stay dark

#### Scenario: Patched
- **WHEN** a cable joins two jacks
- **THEN** those jacks' halos rest

### Requirement: The rack screen shows the relevant inventory
As in Satisfactory, the rack screen SHALL show the player's inventory: a "Relevant" strip
of the items the rack's open places accept, each with the amount held and a number key
(1 to 0, in order), followed by the rest of the inventory. Clicking a relevant item, or
pressing its number, SHALL insert it; an item the player holds none of SHALL show dimmed
with where it comes from.

#### Scenario: Inserting the VCO
- **WHEN** the burbler's rack is open with its VCO slot empty and the player holds 2 square waves
- **THEN** the Relevant strip shows "[1] Square wave x 2", and pressing 1 mounts a Square VCO and leaves 1 square wave

