## 0. Decide (user)
- [x] 0.1 One high-pass filter per rock opening and per hollow trunk; not every trunk is hollow (user, 2026-11-19)
- [x] 0.2 Recur semi-daily (`after` 1800 s); hollows collectable while dormant; collecting silences the source until it returns (user, 2026-11-19)

## 1. Art (Claude)
- [x] 1.1 (claimed: claude; done 2026-11-19) Rock formations from the boulder kit, several variants, some with an opening and `Opening` sockets; acoustic rock extras; LODs, colliders, prefabs; previews. `tools/blender/create_rock_formations.py`, `assets/props/rocks/rock_formation_{0..5}/`: a pile and a scatter (no opening), a window rock (bored slab), a slot (two fins about 1 m apart), a bridge (capstone on two supports, 1.6 m passage) and a tor (bored boulder plus a slot: two openings). Sockets `Opening_<k>` with extras `resource: rock_opening`, `width_m`, `height_m`; every passage runs along the model's Z (Blender Y). Placed formations are scenery for now: bridged, solid from their LOD0 triangles (openings stay open) and acoustic, through the same game lines as boulders
- [x] 1.2 (claimed: claude; done 2026-11-20) Hollow scrub oak trunk variant with a `Hollow` socket; seeded hollow rate in `assets/flora/scrub_oak/asset.json`. Variants 6 and 7 (`tools/blender/create_scrub_oak.py`, `HOLLOW`): the stems rise from a squat gnarled bole with a weathered hollow bored into one side, its mouth the `Hollow` socket (extras `resource: hollow_trunk`). The rate is data: the seeded `oak_groves` rule picks among all eight variants, two of them hollow (5 of 19 oaks on seed 42), so no per-instance switch was needed
- [x] 1.3 (claimed: claude; done 2026-11-19) Item icon `assets/ui/icons/high_pass_filter.png`

## 2. Rules (data)
- [x] 2.1 (claimed: claude; done 2026-11-20) `high_pass_filter` item; `rock_opening` and `hollow_trunk` resources yielding one, `replenishment: after 1800`; `rules-fmt`

## 3. Game (Codex)
- [x] 3.1 (claimed: claude; done 2026-11-20) Binders: formation and hollow oak sockets become pickups with stable instance IDs (`model_part_bindings`). Done generically (user: "collection should be an abstract effort, not written for each object"): any leaf socket of a placed model whose glTF extras name a `resource` the rules define is a pickup of it (`stubby_game::collectibles`), instance `<placement>/<socket>`; restored sockets now carry their extras, and are restored once per model (they were doubled per flattened mesh). A hollow oak needs only its `Hollow` socket tagged `resource: hollow_trunk`
- [x] 3.2 (claimed: claude; done 2026-11-20) Collect with E through the existing pickup path; the recorder records it
- [ ] 3.3 (audio stream) Sound near an opening or hollow is heard high-passed (D3)
- [x] 3.4 Seeded placement: formations and hollow oaks (`add-hybrid-world-placement` 2b.4, 2b.5). Formations (#255); hollow oaks 2026-11-20, each hollow bound as a `hollow_trunk` pickup by the generic socket rule
- [ ] 3.5 Bounded watchdog review: find an opening, collect its filter
