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

- [ ] 3.3 The grotto burbler's E prompt reaches only from one side (user, 2026-11-18): its configuration trigger (the `Interact` sensor, 3 m horizontal reach) sits off the reachable side or is blocked by the boulders. Move the burbler or its `Interact` node so the player can configure it from the open ground at the grotto's mouth; recheck with a bounded run. Deferred by the user

## 4. Audio (audio stream)
- [ ] 4.1 (claimed: Codex) Hear and measure reflections/reverb in the grotto with its burbler patched (`add-rack-and-spatial-audio` 4.8); record the controlled review in `docs/audio/acoustic-level-authoring.md`. The 2026-11-18 bounded review shows strong authored-path occlusion (blocked outside about −97.4 dBFS; clear outside about −33.7 dBFS), 5/5 live reflection output/emitter matches, and a short post-mute decay at the inside and clear outside positions. Subjective listening and a reflection-disabled control remain; the result is not a calibrated cave reverb. Details and limits are in `docs/audio/grotto-acoustics-investigation.md`.
