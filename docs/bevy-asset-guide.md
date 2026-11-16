# Assets in Bevy and Jackdaw

Place one master prefab; its model owns the levels of detail. The engine chooses
which level to render. Asset families are grouped under `assets/<category>/<id>/`:

```text
flora/scrub_oak/
  asset.json
  colliders.json
  scrub_oak_0/
    asset.json
    scrub_oak_0.bsn
    scrub_oak_0.glb
    scrub_oak_0.glb.meta
machines/mudpit_burbler/
  asset.json
  colliders.json
  mudpit_burbler.bsn
  mudpit_burbler.glb
  mudpit_burbler.glb.meta
```

Existing flora and desert variants each retain their own prefab. Family-prefab
variant selection is a separate feature; moving files must not change a placed
variant or its identity. The burbler rig is a separate presentation asset with
its own profile under `machines/mudpit_burbler_rig/`.

## Authoring and import

1. Edit the Blender generator/source under `tools/blender/` and
   `assets-src/blender/<category>/`. Export into the asset's runtime folder. Do not
   edit imported meshes or re-export merely to change their folder.
2. For hand-authored levels, export `<id>_LOD0`, `<id>_LOD1`, etc. inside one `.glb`.
   Select the model in Jackdaw, choose **Import LOD levels**, then **Apply**.
   For appropriate rigid models, **Generate** simplifies the model's meshes.
   Review stateful or animated meshes before accepting generated levels.
3. Commit `<model>.glb.meta` beside the model. It chooses Jackdaw's `ModelLoader`
   and records level sources, model size, transition style and screen thresholds.
   Preserve existing metadata when moving a model. Generated caches are disposable.
4. Drag the model into Jackdaw, add its gameplay components and use **Save as
   Prefab**. Save `<id>.bsn` beside the model. Edit existing prefabs in Jackdaw;
   the migration only changes their model paths. Do not add a `LodGroup` per
   placement. `LodOverride` is available for deliberate placement exceptions.
5. Set unique gameplay instance IDs where required. Never put a physics body on
   a flashing/scaling visual; use the existing interaction markers and binders.

Profile and `GltfSource` paths are relative to `assets/`, for example
`flora/scrub_oak/scrub_oak_0/scrub_oak_0.glb#Scene0` in a profile. Preserve scene
selectors, glTF node names, animation clips, collider coordinates, and state extras
such as `start_scale`. Keep previews under `previews/`, mirroring asset folders,
and share textures/materials where already shared.

## LOD policy

With this project open in Jackdaw, `python3 tools/jackdaw_lods.py --dry-run` lists
the models to import/generate. Without `--dry-run` it calls Jackdaw's import,
generate and apply operators, then applies the configured thresholds and crossfade.
`--switch` applies the authoring policy to existing metadata without contacting the editor.
`--check` validates metadata against that policy without writing it.
Normal loading and this folder migration do not rewrite thresholds.

`tools/assets/lod_policy.json` is an authoring aid: reference distances at 45 degrees
vertical FOV, unit placement scale, using the model's largest-side `size`.
For distance d and size s, screen height is `s / (2*d*tan(FOV/2))`.
Jackdaw owns runtime conversion, including placement size and camera FOV. The
last screen threshold controls final disappearance, with an explicit reference
cull distance per family. Flora handovers come from each family's `far_lod.distance_m`.
The first tuning pass uses a 12% crossfade band. The shell's reference handover is
1 m and final cutoff 12 m; its placed 3x scale makes these about 3 m and 36 m.
Keep the policy and committed metadata together when tuning. Bevy's visibility
ranges support dithering; this does not imply streaming or automatic glTF LOD
simplification in a plain Bevy loader.

## Verification and shipping

Run `python3 tools/check_asset_layout.py` and
`cargo test -p stubby_jackdaw --test prefabs --locked` after moving assets.
The checker resolves profile/collider/prefab paths; the Rust checks parse actual
Jackdaw metadata and verify referenced LOD nodes against the glTF.
`cargo test -p stubby_game --test module_assets --locked` checks module bindings.

Ship the entire runtime `assets/` tree including `.glb.meta` files. Run the
`stubby_jackdaw` binary for the authored world: its standalone and editor modes
both use Jackdaw's runtime plugin. The bare `stubby_game` executable is a legacy
procedural preview and does not load Jackdaw prefab scenes. No editor cache is
needed; Jackdaw generates configured simplified levels on a fresh load.

Build first. Graphical diagnostics must use `tools/jackdaw_timed_test.py` and its
independent 20-second watchdog. For the authored standalone review use
`python3 tools/jackdaw_timed_test.py --standalone --prefabs --name=lod-layout`. Review transitions, final culling, changed FOV
and scale, and dormant/grown/harvested parts. Do not claim a rendering speedup
from a folder move. Research and versioned engine sources are in
[the LOD research](research/bevy-jackdaw-lod-assets.md); the complete path mapping is
[lod-asset-paths.json](research/lod-asset-paths.json).

## Bounded LOD review

Build `stubby_jackdaw`, then use `tools/jackdaw_timed_test.py --lods flora --name=lod-flora`
or `--lods machines --name=lod-machines`. Each launches a standalone tripod review
under the independent 20-second watchdog. It holds noon to make geometry comparisons
readable, without saving changes to the production opening's time or placements.
Screenshots and `events.jsonl` go to `/tmp/stubby-lod-<name>/`. Review the normal dawn
opening separately when changing atmosphere/materials.

The evidence includes actual runtime requested/ready masks, ranges, FOV and scale.
Requested/ready levels include Jackdaw's prefetch/hysteresis: readiness is not the
same as a drawn mesh. `submitted_triangles` counts CPU-visible mesh index triangles,
including scene-instance descendants; it is not a GPU timing measurement or the
number of rasterized pixels. Degenerate scale-zero state meshes may still count.
