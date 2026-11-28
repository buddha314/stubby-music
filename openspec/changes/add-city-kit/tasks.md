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

- [x] 5.1 (claimed: Claude; decided against, 2026-11-28) ~~Bridge `city_` pieces to `PlacedAsset` with a no-op binder.~~ Measured: bridging the district's ~4,900 placements stalled the first frame past 20 s; without it the district runs. City pieces are scenery and stay unbridged; what needs the placement (collision) reads the `GltfSource` path instead.
- [x] 5.2 (claimed: Claude; 2026-11-28) Acoustics: single-level pieces (143 of 149) already reach Steam Audio through `prefab_presentation`'s node-extras reader, which names no kind, and their sockets stay as named nodes. Not covered: the flattened monument's acoustic rock (it would need bridging; left out, see 5.1).
- [x] 5.3 (claimed: Claude; 2026-11-28) `stubby_jackdaw/src/city_colliders.rs` gives every placed City model its shapes from `colliders.json` as static `Solid` children: walls block, and walkable shapes (paving, decks, bridges, tilted stair slabs, the canal bed) are stood on, since the player's body (bevy_ahoy) stands on colliders. Seen in the City save (`add-save-slots`).
- [ ] 5.4 Give canals a ground cut (or a raised city podium) so a district can sit at ground level.
- [ ] 5.5 Bring the full district within frame budget: measured 2026-11-28 on `breq` (RX 7700S, vsync, optimized dev build) at about 82,000 entities and 30-35 fps with dips to 7-10, against 60 fps for one building (the validation bay). Candidates: merge each building into one model with its own levels of detail (far buildings as `city_far_*` proxies), and split the district into regions that load separately.
