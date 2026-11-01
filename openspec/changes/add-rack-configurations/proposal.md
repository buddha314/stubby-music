## Why
All machines are instruments: every machine is configured on a mini Eurorack the
player builds and patches, as the mudpit burbler is. Racks will grow (VCOs, filters,
VCAs and more), and each machine needs its own configurator. Without a shared rack,
every machine would reinvent the screen, slots, mounting and patching.

## What Changes
- An abstract rack: one common `Rack` type holding what every rack has, plus a
  `RackMachine` trait for what each machine does differently.
- Configuration recipes: each machine's rack layout is data, a list of slots, each
  for one component kind (clock, VCO, VCF, VCA, ...) at a width in HP, fixed or not.
- Labelled outlines: an empty slot shows the outline and label of the component
  that goes there, so the player sees what to make and where it goes.
- Shared art: a rack frame that stretches to any width, slot outlines for common
  widths, and a label per component kind (`tools/blender/create_hud_rack.py`).
- The mudpit burbler becomes the first machine on it.

## Capabilities
### New Capabilities
- `rack-configurations`: the abstract rack, configuration recipes, labelled slots.

## Impact
`crates/stubby_game` (rack screen, mounting, patching), machine profiles' `configure`
blocks, `add-mudpit-burbler` (first machine), later the water filter rack.
