## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 `tools/blender/create_flashlight.py`: forensic light source model (grip, body, bezel, violet lens; `Grip` and `Lens` nodes), previews, profile
- [x] 1.2 `tools/blender/create_disco_ball.py`: a mirror ball of square facets
- [x] 2.1 `stubby_core::FLASHLIGHT` (B); the flashlight in `HAND_ITEMS`; while held, the spinning disco ball with its ring of spotlights
- [x] 2.2 Reveal: pickups a beam passes over within range flare and fade; tests for the beams' directions, range and fading. Changed 2026-11-18: 48 small beams out to 30 m, revealing by heading at any height, so reveals read as steady (`Spotlit`; bugs stay lit between flashes)
- [x] 2.3 Hologram material (`assets/shaders/hologram.wesl`), overlay copies and beacons; style in `assets/ui/flashlight.json`
- [x] 2.4 (user, 2026-11-17) The disco light is a toggle on B, not a hand item: `DiscoLight`, no flashlight model in hand, the recorder stays held; test for the toggle, the hand and an open screen
- [x] 2.5 (user, 2026-11-17) The ball appears in view: the upper-left corner of a level view (33 degrees left, 17 up, 0.95 m), matched to the user's reference; near a clock it pulses on the nearest clock's heard beats (`pulse` in `assets/ui/flashlight.json`); tests for the placement and the swell
- [x] 3.1 Bounded graphical review: the ball's spots sweeping over a shell and reeds at dusk and at night
