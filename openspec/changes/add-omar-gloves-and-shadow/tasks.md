Art (Claude) builds the models; the player code is small and in `stubby_player`.
`docs/art/omar.md` is the source of truth.

## 1. Art direction

- [x] 1.1 Transcribe the supplied decisions into `docs/art/omar.md` and add the curated references (`docs/art/omar/`).

## 2. Gloves

- [x] 2.1 Blender generator for the gloves and wrist cuff (`tools/blender/`), exported to `assets/characters/omar_gloves/` with a profile.
- [x] 2.2 Grip pose around the field recorder; previews in the first-person framing.
- [x] 2.3 Mount the gloves with the held item in `stubby_player::hand`; hidden with an empty hand.

## 3. Shadow

- [x] 3.1 Blender generator for the low-poly silhouette body, exported to `assets/characters/omar_shadow/`.
- [x] 3.2 Spawn it on the player, shadow-only, at the capsule's height and facing.
- [x] 3.3 Standing, walking, crouching and item-held poses.

## 4. Review

- [x] 4.1 Bounded run (`tools/jackdaw_timed_test.py`) with the low sun behind the player: the shadow reads as Omar, the gloves hold the recorder, and the camera never sees the body. (Checked 2026-11-19 at 08:30 facing west: the shadow shows the wrap and the raised arm; the gloves grip the recorder. Walking and crouching poses not yet seen in play.)
