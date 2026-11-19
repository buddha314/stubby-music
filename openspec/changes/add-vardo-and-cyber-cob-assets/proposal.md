## Why

The world needs a recognizable, coherent vardo and pony asset pair. The user supplied a concrete art direction for a historically informed Irish Traveller barrel-top wagon and a working piebald cob with one prosthetic limb. Capturing it in the art guide and an OpenSpec change gives the Blender asset work a stable source of truth and catches continuity errors across views.

## What Changes

- Add the vardo and cyber-cob art direction and curated concept references to `docs/art/`.
- Create reproducible Blender source scenes and runtime glTF asset families for the wagon and horse, following the existing asset layout and review pipeline.
- Validate geometry and visual continuity across both assets' authored views; keep the concept PNGs out of runtime assets.

## Capabilities

### New Capabilities
- `vardo-cyber-cob-assets`: continuity, presentation, and asset-pipeline requirements for this paired art set.

### Modified Capabilities
None.

## Impact

Adds Blender generators/sources under `assets-src/blender/structures/` and `assets-src/blender/creatures/`, runtime asset families under `assets/structures/vardo/` and `assets/creatures/cyber_cob/`, previews under `previews/`, and design references under `docs/art/`. Any later level placement, interaction, animation, or gameplay behavior requires its own game-system scope; this change defines and produces the art assets.
