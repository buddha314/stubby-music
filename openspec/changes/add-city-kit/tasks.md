Art and asset pipeline unless noted. Source of truth: `docs/art/city-kit.md`.

## 1. Direction and discovery

- [x] 1.1 (claimed: Claude) Save the brief as `docs/art/prompts/city-kit.md`; write the brief, contract and overrides in `docs/art/city-kit.md`; add City colours to the style guide.
- [x] 1.2 (claimed: Claude; Blender 5.4 alpha on `breq`) Inspect repository conventions (exporter, LOD nodes, sockets, acoustic presets, prefab and collider formats) and test a headless export.

## 2. P0 kit

- [x] 2.1 (claimed: Claude) Foundation and paving, both facade variants with corners and ends, floor and roof, outdoor walks, stairs, ramp, guardrail and lift.
- [x] 2.2 (claimed: Claude) Canal banks, corners, ends, water, access steps and the mud-stream relic; footbridges and abutments; planting (planter, agave, grass, shrub, tree, vine strip).
- [x] 2.3 (claimed: Claude; `check_city_kit.py`, 86/86) Validate every glb: material acoustic extras, identity transforms, normals and UVs, sockets, prefabs; previews re-import each glb.

## 3. P1 and P2

- [x] 3.1 (claimed: Claude) Terraces, light rail, gallery, concert hall, monument, luminous flora, the water-music staff.
- [x] 3.2 (claimed: Claude) Street furniture, infrastructure machines (amber signal, `Interact`, `Audio_source`), growth overlays and a sapling.

## 4. Reference district

- [x] 4.1 (claimed: Claude) Build the district and test strip from linked instances, with a hidden blockout, Day/Dusk/Night scenes and five review cameras; render the review views.
- [x] 4.2 (claimed: Claude) Write the layout as `assets/world/asset_reviews/city.bsn`, the manifest, the colliders and the LOD metadata; run the asset layout and LOD policy checks.
- [x] 4.3 (claimed: Claude; 2026-11-28 on `breq`, 20 s watchdog, no errors in the log) Review the district in the game (`tools/jackdaw_timed_test.py --asset-review city`): the prefabs load and render (arcades, vines, lanterns, planting). The player spawns at the opening point beside the lifted district, in the bleak haze; the bloomed look waits on light quality from foliage. A review spawn inside the district is still to add.

## 5. Game support (Codex: crates)

- [ ] 5.1 Add the `city_` prefix to `stubby_jackdaw/src/prefab_bridge.rs` and a no-op scenery `AssetBinder` in `stubby_game/src/glue.rs`.
- [ ] 5.2 Add `city_` to `model_part_bindings.rs` so the pieces' `acoustic` node extras reach Steam Audio, and their sockets (`Interact`, `Audio_*`, `Display_0`, `Recording_source`) are recreated.
- [ ] 5.3 Author the colliders in `assets/structures/city/colliders.json` on the prefabs once `add-prefab-colliders` lands; support standing on `walkable` shapes and tilted stair slabs.
- [ ] 5.4 Give canals a ground cut (or a raised city podium) so a district can sit at ground level.
