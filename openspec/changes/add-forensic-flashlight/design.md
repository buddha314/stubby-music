## Context

The hand (`stubby_player::hand`) holds one item from `HAND_ITEMS`, each with a key and a
model posed on the player camera. Collectibles are marked `Interactable::Pickup` while
available (`stubby_core`); the aimed one is outlined (`sync-targeting-and-compass`).

## Decisions

**D1. A hand item, B.** The flashlight joins `HAND_ITEMS` with `FLASHLIGHT` (B): the torch
in the hand, its light the disco ball (D2). Holding it is being on: no separate switch. Putting it away (B again, or H for the recorder) turns it
off. Not while a screen has the controls (as the hand already is).

**D2. The light is a disco ball.** While the flashlight is held, a mirror ball (a
generated model of square mirror facets) floats in front of the player at chest height,
a child of the view, and spins. It carries a ring of narrow `SpotLight`s, aimed outward
and down from the ball and spread round it, so their spots land on the ground and sweep
as it turns. Colours alternate violet and cyan (forensic violet, disco sparkle). Shadows
off: a dozen shadowed spotlights would be a dozen shadow passes. Count, colours, width,
range, spin and the ball's size and place are data.
- Rejected: one forward cone (the first design): the user wants the disco-ball sweep.

**D3. Reveal where the beams pass, over the pickup contract.** Each frame, each beam's
world direction is known from the ball's spin; an entity with `Interactable::Pickup`
within range and inside any beam's reveal width is lit to full, and otherwise fades over
about a second, so collectibles flare as spots cross them and glitter as the ball turns.
No asset kind is named: anything collectable is shown.

**D4. The hologram is our own small material**, a Bevy `Material` with a WGSL shader
(`assets/shaders/hologram.wesl`, WESL): Fresnel rim (cyan to violet), scanlines scrolling in
world space, a little flicker, additive, unlit, and depth test off so it shows through
grass. It is drawn on overlay copies of the pickup's meshes (children of each mesh, so
they follow its animation and LODs), plus a beacon (a thin column and a turning diamond)
above the pickup. No crate does this; it is a few dozen lines of standard Bevy.
- Rejected: outlines for the reveal (the outline already means "E acts on this").
- Rejected: revealing everything in range regardless of the beams: the sweep is the feel.

## Risks

- [Overlay copies of thousands of reed blades] → only pickups in the beam are revealed,
  and the copies are removed when they leave it.
- [Through-occluder drawing reads as a cheat] → it is the point of the tool; range keeps
  it local.

## Measured (2026-11-16, RX 7700S, embedded Play, `--sky` review)

The main opaque pass goes from about 2.2–2.8 ms to 4.6–5.0 ms while the ball's twelve
shadowless spotlights are on (clustered lighting touches every lit fragment). Acceptable
for a tool held briefly; `beams.count` in `assets/ui/flashlight.json` is the lever if it
matters on the integrated GPU. The holograms' transparent pass stays under 0.03 ms.
