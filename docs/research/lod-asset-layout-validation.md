# Asset layout migration validation

2026-11-16, branch `docs/lod-asset-layout`, based on `f7638fa`.

## Changes checked

- 43 GLBs and 17 adjacent Jackdaw metadata files moved without changing a byte.
- 27 master prefabs moved beside their corresponding variant/model; their contents
  change only in model paths. Existing placed instances and gameplay IDs remain.
- Profiles, collider references, world scenes, rack/hand/stream paths and exporters
  now name asset-family paths. Module collider entries were split into their
  corresponding module folders; family-level flora/desert colliders remain shared.
- `tools/jackdaw_lods.py --dry-run` discovers the same 10 authored and 7 generated
  LOD jobs at their new paths. Existing size, fade, thresholds and final culling
  values are preserved, including Snap transitions. No FOV/scale policy was retuned.

## Results

- `python3 tools/check_asset_layout.py`: passed; generated/mudflat.glb is the one
  explicit exception because Jackdaw prepare generates this ignored floor.
- Missing-reference negative check: reported the source profile and missing model.
- `cargo test -p stubby_jackdaw --test prefabs --locked`: 5 passed. Covers known
  prefab types/real models, seven-instance review scene resolution, all 17 metadata
  files, a missing LOD node diagnostic, and an isolated clean-cache load of both
  authored oak levels and all three generated burbler levels.
- `cargo test -p stubby_jackdaw --test prefab_runtime --locked`: 2 passed;
  every moved prefab binds to its expected live gameplay asset, and repeated bugs
  retain independent identities/clocks.
- `cargo test -p stubby_game --test module_assets --locked`: 2 passed.
- Version consistency, Python compilation, strict OpenSpec validation and diff
  whitespace checks passed.

## Bounded graphical evidence

Command: `python3 tools/jackdaw_timed_test.py --standalone --prefabs --name=lod-layout`.
The timed tool now has an explicit standalone mode using the existing watchdog,
display handling and recovery; it does not open or contact an existing editor.
The built authored-world binary uses JackdawPlugin, as editor Play does.

The watchdog killed the process at 20.022 seconds (exit -9 is expected). It ran on
GNOME/Wayland, RADV RX 7700S, Vulkan, at 2560x1600/60 Hz and restored that mode.
The review reported `near_check.success=true` and `complete.success=true`, with
seven assets, no prefab errors, collection, regrowth, sample-driven animation,
flashes, a loaded/configured rack, and far foliage seen before returning near.
The completed review retained ten player cables and the expected item yields.

Evidence remains at `/tmp/stubby-lod-lod-layout/events.jsonl` and screenshots in
that folder; watchdog report/log are `/tmp/jackdaw-flicker-lod-layout.{json,log}`.
Screenshots confirm the loaded rack and models. The log contains transient
commands targeting despawned entities during rack replacement; it contains no
asset load errors or panics. Those warnings are outside this path-only migration.

This verifies one bounded run, not indefinite display stability or a performance
gain. Changed camera FOV/placement scale and the final disappearance threshold were
not separately exercised visually: their model metadata and runtime code are
unchanged. Fresh visual calibration is appropriate when those settings change.
