## Context

The hand (`stubby_player::hand`) holds one item from `HAND_ITEMS`, each with a key and a
model posed on the player camera. Collectibles are marked `Interactable::Pickup` while
available (`stubby_core`); the aimed one is outlined (`sync-targeting-and-compass`).

## Decisions

**D1. A disco light on B, not a hand item (revised by the user, 2026-11-17).** B toggles
`stubby_player::flashlight::DiscoLight`; the disco ball (D2) shows while it is on,
whatever the hand holds, so the field recorder stays up and no flashlight model is shown
(the flashlight model, its profile, previews, `.blend` and `tools/blender/create_flashlight.py` were removed on 2026-11-17; they are in git history before that date if a held torch ever returns). The original decision follows: The flashlight
joins `HAND_ITEMS` with `FLASHLIGHT` (B): the torch in the hand, its light the disco ball (D2). Holding it is being on: no separate switch. Putting it away (B again, or H for the recorder) turns it
off. Not while a screen has the controls (as the hand already is).

**D2. The light is a disco ball.** While the disco light is on, a mirror ball (a
generated model of square mirror facets) appears, when switched on, in the upper-left
corner of a level view: 0.95 m away, 33 degrees left and 17 degrees up, cut by the left and
top edges at 16:9, so the player knows it is there (matched to the user's reference
screenshot, 2026-11-17; it was first above the head, 2026-11-16, and easy to miss). Near a clock it pulses to the nearest clock's beats as
they are heard (`pulse` in the style: reach, depth, settle time). It is placed once, from the eye's place and heading at
that moment, and stays at that place in the world while on; it does not follow the view.
It spins about the vertical. It carries a ring of narrow `SpotLight`s, aimed outward
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
- Changed (user, 2026-11-18): a dozen 5-degree beams crossed each place too seldom, so
  pickups blinked, and bugs between flashes vanished with their overlays. Offered "always
  on" or "more, smaller", the user chose more, smaller: 48 beams of 2.5 degrees whose spots
  land from about 3 m to 27 m (far ones brighter by distance squared), a beam's heading
  revealing at any height, so each place is crossed several times a second; revealed
  pickups are `Spotlit` and bugs stay lit. A second ring of 16 beams goes up (25 and 50
  degrees above the horizon, between the downward ones; user, 2026-11-18: "for later
  use"), lighting what is overhead. The GPU light clusters start large enough for all of
  them, so Bevy never resizes them mid-play (it warned it might draw a few wrong frames).

## Risks

- [Overlay copies of reed clumps] → only pickups within reach (30 m) are revealed, and
  their copies are removed when the player leaves or switches off.
- [Through-occluder drawing reads as a cheat] → it is the point of the tool; range keeps
  it local.

## Measured (2026-11-16, RX 7700S, embedded Play, `--sky` review)

The main opaque pass goes from about 2.2–2.8 ms to 4.6–5.0 ms while the ball's twelve
shadowless spotlights are on (clustered lighting touches every lit fragment). Acceptable
for a tool held briefly; `beams.count` in `assets/ui/flashlight.json` is the lever if it
matters on the integrated GPU. The holograms' transparent pass stays under 0.03 ms.
