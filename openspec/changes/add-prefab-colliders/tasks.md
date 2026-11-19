## 1. Load the integration

- [ ] 1.1 Jackdaw fork: add `exclude = ["integrations"]` to the root `Cargo.toml` on a `fix/` branch; confirm the fork still builds; add it to the fork-carried list, with nothing filed upstream
- [ ] 1.2 Add `jackdaw_avian_integration` (`default-features = false`) to the workspace's pinned Jackdaw git dependencies and `crates/stubby_jackdaw`; confirm one Bevy and one avian (`tools/check_jackdaw_versions.py`, `Cargo.lock`)
- [ ] 1.3 Add `AvianColliderBridgePlugin` beside `JackdawPlugin`, and decide how the standalone `stubby_game` gets the same colliders
- [ ] 1.4 Decide what `jackdaw.toml` needs: `[modules] physics = true` or nothing, given the editor does not ship the Physics module

## 2. Scrub oak colliders

- [ ] 2.1 Author `AvianCollider` cylinders on child entities of `assets/flora/scrub_oak/scrub_oak_<k>/scrub_oak_<k>.bsn`, from `assets/flora/scrub_oak/colliders.json`
- [ ] 2.2 `cargo test -p stubby_jackdaw --test prefabs` loads them with known types
- [ ] 2.3 Confirm one placed oak has a static collider in Play and survives opening and saving in the editor; if prefabs cannot load `AvianCollider`, stop and use the private-component fallback in the design

## 3. Player

- [x] 3.1 Stop-and-slide in `move_player` (`controls.rs`) against static colliders; existing `controls.rs` tests still pass (2026-11-18: a capsule cast against `stubby_core::Solid` colliders, sliding along the hit, up to three times a step)
- [ ] 3.2 Confirm E targeting near an oak and walking through reeds, by hand, in a bounded 20-second Play

## 4. Other solid families

- [ ] 4.1 The burbler's hot-mud pool (`assets/machines/mudpit_burbler/colliders.json`); its stream banks stay walkable
- [x] 4.3 Boulders (user, 2026-11-18: "I can walk through the boulders"): each placed boulder gets static trimesh colliders from its own `<id>_LOD0` meshes, built in `stubby_jackdaw::model_part_bindings::solid_boulders` and marked `Solid`, not the coarse `colliders.json` boxes, so the grotto's opening stays open. Bounded check: walking straight at a boulder stops at its surface and slides along it
- [ ] 4.2 Stumps (`assets/props/desert/colliders.json`) and any other families `colliders.json` lists as solid

## 5. Docs

- [ ] 5.1 Update `docs/research/player-controller-and-physics.md` and `tools/collider_shapes.py`'s docstring: physics is no longer gated
- [ ] 5.2 Mark `add-desert-opening` 4.2d done or point it here; remove the interim movement when 4.2b lands
