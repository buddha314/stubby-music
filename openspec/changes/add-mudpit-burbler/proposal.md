## Why

The clock needs a source in the world that fits the theme (the music of nature brings
light to the world) and the builder-game model (resources come from nodes and mines).
A boiling mud pit already keeps time: it burbles. Making it a configurable clock mine
gives the player their first machine to set up, Satisfactory-style, and their first
complete voice: a clock, a VCO and a volume control.

## What Changes

- **Mudpit burbler**, a world object that is an infinite mine producing a **clock**:
  every burble is a beat, at a tempo set per burbler.
- **A resource node like Satisfactory's iron ore**: a gurgling hot-spring mud pot that
  bubbles at its BPM.
- **Configuration**, Satisfactory-style: E at the burbler opens a HUD; Escape closes it.
- **The HUD is a basic three-place Eurorack**: the burbler's built-in clock (fixed), a
  VCO slot, and a volume knob standing in for a full VCA. Together they make the
  minimal voice: clock triggers VCO, volume sets the level. Nothing is built in the world.
- **Mud streams** flow from the burbler's outlet and pulse at its BPM: the stream is
  the played sequence, laid out in space. Raw mud stays muddy; other constructors
  clean it. **Reeds** will be the conveyor-belt segments (a later change).
- **Connection halo**: the outlet lights a soft halo while something is connected.
- **Assets** (art): the burbler (`mudpit_burbler`); the HUD rack's clock panel and
  volume module. The earlier rack-beside-the-pit version is kept as a concept,
  `mudpit_burbler_rig`, not used by the game.

## Capabilities

### New Capabilities
- `mudpit-burbler`: the clock node, its configuration HUD, its voice, its mud-stream
  outlet and connection halo, and its asset.

### Modified Capabilities
<!-- none -->

## Impact

- Art: `tools/blender/create_burbler.py`, `assets/models/mudpit_burbler.glb`,
  `assets/machines/mudpit_burbler/`.
- Game: a new mine definition (`stubby_recipes`), an interaction and UI screen, and
  an engine patch per configured burbler (`stubby_game`). Uses the existing Clock,
  VCO, Envelope and VCA modules.
- Relates to `add-resource-source-model` (mines) and `add-module-asset-binding`
  (placing modules in slots, module IDs).
