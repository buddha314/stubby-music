## Why

The player character now has a name and a look: Omar, from the user's art direction
(2026-11-18, `docs/art/omar.md`). The game is first person, so for now the player needs to
see only two things of him (user: "right now I think we just need Omar's gloves and
shadow"): his gloved hands holding the hand item, and his shadow on the ground. Today the
recorder floats in the lower right with no hands, and the player casts no shadow.

## What Changes

- Record Omar's art direction and curated references in `docs/art/` (done with this
  proposal).
- Model Omar's gloves (long, midnight navy, with the coat's flared mint cuff at the
  wrist) as a first-person hand rig that holds the hand item in its existing pose.
- Give the player a shadow-only body: Omar's full silhouette (tall wrap, knee coat, flared
  sleeves, puffed trousers, boots) that casts a shadow but is never drawn to the camera,
  following walking, crouching and the held item.

## Capabilities

### New Capabilities
- `omar-character-assets`: Omar's first-person gloves and shadow body, their look and how
  the player sees them.

### Modified Capabilities
None.

## Impact

Art: Blender generators under `tools/blender/` and `assets-src/blender/characters/`,
runtime families `assets/characters/omar_gloves/` and `assets/characters/omar_shadow/`,
previews under `previews/`. Code (small, `stubby_player`): mount the gloves under the
held item's pose, spawn the shadow body on the player with Bevy's shadow-only rendering,
pose it for crouching. The rest of Omar (a full rendered body) is out of scope until the
user asks.
