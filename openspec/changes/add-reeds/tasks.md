## 1. Art (Claude)
- [x] 1.2 Reeds outline the stream segment (`Reeds_L`, `Reeds_R`), dry to green with clarity (`create_stream.py`) (claimed: claude)
- [x] 1.1 Reed clump asset: four seeded variants, standing/harvested/regrowing, dry-to-green, sway groups, far LOD (`tools/blender/create_reeds.py`, `assets/flora/reeds/`) (claimed: claude)

## 2. Game (Codex)
- [ ] 2.1 Catalog: `reed` item and `reed_clump` resource, yield and a timed replenishment policy (see design for a starting point)
- [ ] 2.1a Catalog recipe: 6 `reed` → 1 stream segment; laying a planned stream spends it per segment, and the hologram's `invalid` state covers an unaffordable plan
- [ ] 2.2 E harvest of reed clumps through the existing pickup path; silent on stubble
- [ ] 2.3 Show the clump's state from the harvest ledger: `Standing` vs `Stubble`; optional regrowth scale
- [ ] 2.4 Place clumps on wet ground from the world seed; variant and per-instance variation as in `assets/flora/reeds/asset.json`
- [ ] 2.5 Greenness from local water or light quality; sway `Sway_<n>` with the wind

## 3. Prefabs carry their resource (found 2026-11-01)
The reed prefabs (`jackdaw/assets/prefabs/flora/reed_clump_0..3.bsn`) hold only the
model; Play adds `ResourceNode` by file stem. Bugs and oaks carry theirs on the prefab,
so a placed reed shows no resource and no editable `instance` in the editor.
- [ ] 3.1 (Claude, in Jackdaw once the editor builds again) add `ResourceNode { resource: "reed_clump", instance: "reed_clump-<k>" }` to each reed prefab and **Save as Prefab**
- [ ] 3.2 (Codex) Play binding keeps an authored `ResourceNode` on a reed placement instead of inserting its own (`prefabs.rs`), as for bugs and oaks
