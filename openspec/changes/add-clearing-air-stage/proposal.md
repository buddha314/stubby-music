## Why

The world currently goes straight from bleak haze to full bloom. The designer has chosen a stage between them (2026-11-06): the haze lifts first and reveals a clear, burning sky, before the colour of life arrives. It gives players a visible reward before full bloom, and it is what the reference image for "dusk as the air clears" shows.

## What Changes

- Add a third lighting mood, `clearing`, between `bleak` and `bloomed`, keyed against the reference in `docs/art/style-guide.md`.
- Light quality blends through three stops instead of two.
- Update the style sheet to show three rows of keys.
- **BREAKING for the keys schema:** `light_quality` gains a stop. Nothing in the game reads the file yet, so no game code breaks.

## Capabilities

### New Capabilities

- `lighting-stages`: the three moods, what each means, and how light quality moves between them.

### Modified Capabilities

None. `add-art-direction` and `add-desert-opening` are in-flight, and their tasks point here.

## Impact

- **Art:** `assets/lighting/lighting_keys.json`, `docs/art/style-guide.md`, `tools/blender/render_style_sheet.py`, re-rendered keys.
- **Game:** `add-desert-opening` 3.4 to 3.7, not built yet, read and blend the keys. The stage cannot be seen in the game until they land.
