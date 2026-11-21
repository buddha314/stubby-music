All tasks are art and asset pipeline unless explicitly noted. The asset guide and the written continuity checklist in `docs/art/vardo-and-cyber-cob.md` are the source of truth.

## 1. References and art direction

- [x] 1.1 Transcribe the supplied decisions into `docs/art/vardo-and-cyber-cob.md` and add the curated reference studies.
- [x] 1.2 (claimed: Claude; 2026-11-20: previews/structures/vardo/ side, front, rear, three-quarters) Review the wagon silhouette with consistent front, rear, side, and three-quarter views; fix towing end, entrance end, wheel ratio, and shared geometry.
- [ ] 1.3 Review the cyber-cob from side and three-quarter views; verify one near-side front prosthesis and three natural legs, plus subtle optical strands in mane and tail.

## 2. Vardo asset

- [x] 2.1 (claimed: Claude; `tools/blender/create_vardo.py`) Create a reproducible headless Blender generator and source scene under `assets-src/blender/structures/`.
- [x] 2.2 (claimed: Claude; LOD metadata written from the policy, re-import in Jackdaw when it is open) Export `assets/structures/vardo/vardo.glb` with a family profile, appropriate collider, Jackdaw metadata/prefab, and model-owned LODs.
- [x] 2.3 (claimed: Claude; also night, top, and `--asset-review vardo` in game) Create matching side, front, rear, and three-quarter previews; check front/rear orientation, steering clearance, proportions, PBR materials, and restrained emissive accents.

## 3. Cyber-cob asset

- [ ] 3.1 Create a reproducible headless Blender generator and source scene under `assets-src/blender/creatures/`.
- [ ] 3.2 Export `assets/creatures/cyber_cob/cyber_cob.glb` with a family profile, appropriate collider, Jackdaw metadata/prefab, and model-owned LODs.
- [ ] 3.3 Create matching side and three-quarter previews; inspect limb count, prosthesis articulation/readability, coat pattern, feathering, mane and tail strands, tack, PBR materials, and emissive strength.

## 4. Pair review

- [ ] 4.1 Review the pony and wagon together at game scale; verify the shafts and practical harness meet without intersecting or obscuring the pony.
- [ ] 4.2 Run the asset layout and prefab checks, confirm both models load in the game/editor, and record the review in the art guide changelog.
- [ ] 4.3 Add the vardo and cyber-cob to the style sheet or an equivalent paired asset review sheet.
