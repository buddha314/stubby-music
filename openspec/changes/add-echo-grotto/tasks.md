Requested by the user on 2026-11-17. Art and scene: Claude. Reflections, reverb and
the listening review: the audio work stream (Codex, `add-rack-and-spatial-audio` 4.6/4.8).

## 1. Boulders (Claude)
- [x] 1.1 (claimed: claude) `tools/blender/create_boulders.py`: Rock Generator add-on (`mesh.add_mesh_rock`, fixed seeds), modifiers applied, decimated, PBR rock material, `"acoustic": "rock"` extras on mesh nodes, LOD0/LOD1 nodes; sources in `assets-src/blender/props/rocks/`, glbs in `assets/props/rocks/boulder_<k>/`, previews in `previews/props/rocks/`; CC0 recorded in the family `asset.json`
- [x] 1.2 Family profile, colliders (`tools/collider_shapes.py`), LOD policy entry and `.glb.meta` (`tools/jackdaw_lods.py --switch`), a Jackdaw prefab per variant; `cargo test -p stubby_jackdaw --test prefabs`

## 2. Acoustic marking (Claude)
- [x] 2.1 `stubby_world`: meshes under a node with glTF extras `"acoustic": "rock"` get `AcousticOccluder::ROCK` on spawn; unit test

## 3. Scene (Claude)
- [x] 3.1 Place the grotto's boulders and a second `mudpit_burbler` in `assets/scene.bsn` (world 70.5, −64.5; bearing 45 degrees, 95 m from spawn; mouth toward spawn; the burbler at the mouth, 61 m from the opening burbler); `tools/check_asset_layout.py`
- [x] 3.2 Bounded watchdog review: the grotto from spawn's direction and from inside, the burbler at its mouth (standalone, 2026-11-17: from 25 m in front the cluster stands backlit at dawn with the burbler glowing at its mouth; from the alcove the ground is in the top boulder's shade, rock frames the view and the burbler sits at the mouth)

## 4. Audio (audio stream)
- [ ] 4.1 (claimed: Codex) Reflections/reverb heard in the grotto with its burbler patched (`add-rack-and-spatial-audio` 4.8); pass the in-game occlusion check for 4.6; record the review in `docs/audio/acoustic-level-authoring.md`. First bounded review (2026-11-17) confirmed the scene geometry is published, but the square demo source stayed unoccluded at matched-distance outside points; live reverb is not routed into Firewheel yet.
