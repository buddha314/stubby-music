## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 `tools/blender/create_flashlight.py`: forensic light source model (grip, body, bezel, violet lens; `Grip` and `Lens` nodes), previews, profile
- [x] 1.2 `tools/blender/create_disco_ball.py`: a mirror ball of square facets
- [x] 2.1 `stubby_core::FLASHLIGHT` (B); the flashlight in `HAND_ITEMS`; while held, the spinning disco ball with its ring of spotlights
- [x] 2.2 Reveal: pickups a beam passes over within range flare and fade; tests for the beams' directions, range and fading
- [x] 2.3 Hologram material (`assets/shaders/hologram.wesl`), overlay copies and beacons; style in `assets/ui/flashlight.json`
- [x] 3.1 Bounded graphical review: the ball's spots sweeping over a shell and reeds at dusk and at night
