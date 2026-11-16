## Why

Collectibles are hard to find: snail shells are 9 cm and silent, and a lightning bug is
invisible between flashes. The user asked (2026-11-16) for a flashlight like a crime-scene
forensic light, on B, that shows all collectibles in a holographic style. B is already
reserved for the flashlight (AGENTS.md).

## What Changes

- **A forensic flashlight in the hand.** B brings it up and switches it on (and puts it
  away), as H does the field recorder; the hand holds one item at a time. Its model is an
  alternate light source of the kind used at crime scenes: a black anodised body, knurled
  grip, wide bezel and a violet lens, made by a Blender generator.
- **Its light is a disco ball** (user, 2026-11-16). The torch's light manifests as a
  mirror ball floating in front of the player. It spins and throws a ring of narrow
  violet and cyan beams that sweep across the ground and anything near, as a disco ball's
  spots do.
- **Collectibles the beams touch turn holographic.** Every available pickup (whatever E
  can collect: bugs, reeds, shells, later ones) that a beam passes over, within range,
  shows:
  - a holographic shell over its body: cyan-to-violet rim glow, scrolling scanlines and a
    slight flicker, drawn through grass and reeds;
  - a holographic beacon above it (a thin column of light and a turning diamond), so a
    9 cm shell or a bug between flashes is still seen.
  It flares as a beam crosses it and fades over a second or so, so collectibles glitter as
  the spots sweep.
- **Style and reach are data:** `assets/ui/flashlight.json` (the ball's size, place and
  spin, the beams' count, colours, width and range, the reveal's width and fade, the
  hologram colours).

## Capabilities

### New Capabilities
- `forensic-flashlight`: the held flashlight, its beam, and the holographic reveal of
  collectibles.

### Modified Capabilities
None in accepted specs.

## Impact

- `stubby_core`: the `FLASHLIGHT` key (B).
- `stubby_player`: the flashlight as a hand item (`hand.rs`), the disco ball and its
  beams, and the reveal over `Interactable::Pickup` (no asset kinds named), with a
  hologram material.
- `assets/shaders/hologram.wesl` (WESL, as Bevy's own shaders now are; user preference),
  `assets/ui/flashlight.json`.
- Art: `tools/blender/create_flashlight.py` → `assets/equipment/flashlight/`;
  `tools/blender/create_disco_ball.py` → `assets/equipment/disco_ball/`.
