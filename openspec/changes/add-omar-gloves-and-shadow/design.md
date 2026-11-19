## Context

`docs/art/omar.md` holds the full art direction; this change builds only what a
first-person player sees. The hand item (`stubby_player::hand`) is a model parented to the
camera at a fixed pose. The player is an avian capsule driven by Tnua; nothing of the body
is drawn.

## Decisions

### D1. Gloves are a first-person model that holds the hand item
One glb with a right glove (and the left, for two-handed items later), a short band of
flared mint sleeve with brass edging at the wrist, posed to grip the field recorder. It
mounts on the camera like the held item and shows only with an item in hand; each hand
item can name its grip pose. Midnight navy, worn-leather roughness, subtle segment seams;
no emissive.

### D2. The shadow is a separate, shadow-only body
A simplified full-body mesh of Omar's silhouette, at the player capsule's height, with
Bevy's shadow-only setting, so it casts on the ground and on boulders but is never seen
by the camera (it would fill the view from inside). Low poly is enough: shadows read only
the outline. Its silhouette carries the tall wrap with the hanging fringed end, the knee
coat and flared sleeves, the puffed trousers and boots.

### D3. The shadow follows the player's state
Standing, walking (a simple stride) and crouching poses, and the right arm raised while an
item is held, so the shadow and the gloves agree. Keyed poses or a short clip set, not a
full animation system.

## Risks

- The shadow-only setting must be checked on the current Bevy main; if it is missing, use
  a render layer the camera does not see.
- Self-shadowing the gloves from the shadow body: keep the gloves out of the shadow
  body's arm and mark the gloves `NotShadowCaster` if they cast an unwanted second hand.
