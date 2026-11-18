## Context

The opening is a flat, open mudflat: nothing reflects sound except the ground. Steam
Audio's reflections and reverb (`add-rack-and-spatial-audio` 4.8) cannot be heard or
tuned there. A small rock shelter in the opening scene gives the audio work stream a
real, authored space, and gives the world its first place to hide from the wind.

## Goals / Non-Goals

**Goals:** a believable rock shelter in the opening scene, editable in Jackdaw, whose
walls and roof are acoustic rock; a burbler inside it to listen to.

**Non-Goals:** the reflection/reverb processing and buses (the audio stream's 4.8), the
listening review's verdict, material tuning beyond Steam Audio's rock preset, caves the
player walks deep into, procedural boulder scatter.

## Decisions

### D1. Boulders from Blender's Rock Generator, baked
The user asked for Blender's rock plugin: the Rock Generator add-on in the Extra Mesh
Objects extension (`mesh.add_mesh_rock`). `tools/blender/create_boulders.py` enables it,
generates each boulder from a fixed `user_seed` and scale (reproducible), applies its
subdivision and displacement modifiers (their procedural textures are Blender-only and
would not survive export) and decimates to a game budget. The add-on is GPL, but its
output geometry is ours: the shipped meshes are CC0, recorded in the family's
`asset.json`. The material is plain PBR (base colour, roughness) in the bleak palette
of `docs/art/style-guide.md`: desaturated warm grey, rough.

### D2. A Mojave boulder cluster with a small grotto at its base
After the user's reference (a cluster of rounded, weathered granite boulders in the
Mojave, 2026-11-17): big smooth boulders leaning on each other, warm buff granite, and a
small grotto at the base. Five boulders (`LAYOUT` in the generator):
- `boulder_0` back: 9 × 7 × 6.5 m;
- `boulder_1` left: 7 × 6 × 5.5 m; `boulder_2` right: 6.5 × 6 × 5 m;
- `boulder_3` top: 8 × 6 × 3.6 m, leaning across the left and right ones;
- `boulder_4` a small 3 × 2.5 × 2 m rock in front, breaking the outline.

The alcove under the top boulder is about 6 m wide, 4 m deep and 3 m high, in front of
the back boulder. The burbler (5 × 6.6 m) does not fit inside: it sits at the mouth,
to one side so the player can walk in, its stream running out. Previews:
`previews/props/rocks/grotto.png`, `grotto_inside.png`, `grotto_top.png`. Placement is
authored in the scene (D4), so its exact fit can be tuned in Jackdaw.

### D3. Acoustic rock by glTF extras
Boulder mesh nodes carry extras `"acoustic": "rock"`. A system in `stubby_world` adds
`AcousticOccluder::ROCK` to the mesh entities under any node with those extras, when
they spawn, like `restore_start_scale` reads `start_scale`. No asset kind is named, so
any future model can opt in the same way. The acoustic mesh is the visible LOD0 mesh
(`docs/audio/acoustic-level-authoring.md`: do not duplicate meshes just for acoustics);
far LODs are not marked.

### D4. Authored in the opening scene, not spawned
The boulders and the second burbler are placements in `assets/scene.bsn`, the opening
level, so they appear and can be moved in the Jackdaw editor (user, 2026-11-17). The
grotto's centre is at world (70.5, 0, −64.5), bearing 45 degrees and 95 m from spawn, so
it shows about 30 degrees left of the opening view, near the scrub oak; its mouth turns
toward spawn (yaw −45 degrees). Its burbler, at the mouth, is 61 m from the opening
burbler, so their hearing radii do not overlap. It is an ordinary placed burbler: its own
seeded BPM, an unpatched rack, silent until configured.

### D5. Colliders
Each boulder gets a cuboid collider from `tools/collider_shapes.py`, so the player walks
around the boulders and into the alcove under the top boulder. Cuboids are coarse for irregular rock; a convex hull
can replace them if the player catches on air.

## Risks / Trade-offs

- [Rock Generator output varies between Blender versions] → generate with the pinned
  local Blender, commit the `.blend` sources and glbs; regenerate only deliberately.
- [The mudflat ground is not an acoustic floor inside the grotto] → the generated ground
  is already tagged acoustic (4.6); check during the listening review.
- [A second burbler in the opening] → kept out of earshot of spawn and the first; it is
  silent until patched.

## Open Questions

- Whether the grotto should also count as shelter for "Build a camp" (the campfire wants
  a sheltered spot); for now it is an audio test space.
