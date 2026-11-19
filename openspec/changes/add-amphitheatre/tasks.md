Requested by the user on 2026-11-18: the amphitheatre drives the story now, assets first.
Art and scene: Claude. Seat behaviour, rules and binder: Codex.

## 1. Look (Claude)
- [ ] 1.1 Reference board in `docs/art/amphitheatre/` (Arches slickrock bowls, arches and fins; Greek hillside theatres, limestone, maquis), each image's licence recorded; a concept in the bleak palette and its cleared state

## 2. Model (Claude)
- [ ] 2.1 `tools/blender/create_amphitheatre.py`: the bowl and tiers, the arch, the dancing floor and `Spring` (`Dry`/`Rising`/`Full`), rim scrub; plain PBR; `"acoustic": "rock"` extras; LOD0/LOD1; source in `assets-src/blender/structures/amphitheatre/`, glb in `assets/structures/amphitheatre/`, previews in `previews/structures/amphitheatre/`
- [ ] 2.2 Seats from `docs/research/beings/layout.json` (design D4): `Seat_<being_id>` sockets with `Empty`/`Hint`/`Filled` stones; reproducible from the layout
- [ ] 2.3 Profile (states), colliders per tier (`tools/collider_shapes.py`), LOD policy and `.glb.meta`, Jackdaw prefab; `cargo test -p stubby_jackdaw --test prefabs`
- [ ] 2.4 The game's three lines (prefab bridge, binder, part bindings for `Spring` and seats; AGENTS.md *A new placeable kind*)

## 3. Scene (Claude)
- [ ] 3.1 A chaparral pocket in `assets/scene.bsn` with the amphitheatre (design D7); `tools/check_asset_layout.py`
- [ ] 3.2 Bounded watchdog review: the first view through the arch, the empty bowl, a half-filled review state

## 4. Behaviour (Codex)
- [ ] 4.1 Rule type for what seating a being gives (Rule PR: schema, validator, example rows)
- [ ] 4.2 Seat states: filling a seat gives its module once, hints its layout neighbours, raises the spring; saved
- [ ] 4.3 A review driver that seats beings without the shrine flow, until `add-deity-shrines` provides waking
