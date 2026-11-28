# The City kit

**Phase 4, the City** (Camp → Village → Town → City): a dense, vertical solarpunk city, lush
rather than chrome, lunarpunk at night. It is the reward for restoring the silent mudflat with
music, water and plants. This brief holds the continuity rules the kit keeps; the prompt that
started it is `docs/art/prompts/city-kit.md`. Regeneration, snapping and the manifest are in
`docs/art/city-kit/README.md`. The change is `openspec/changes/add-city-kit/`.

The written rules here win over the concept images.

## The look

One design language seen four ways (canal at dusk, lunarpunk night, civic plaza, solarpunk
day), built from these:

- **Stepped Southwestern massing.** Pale lime plaster over rammed earth, rounded corners,
  buttress piers at every building corner, stepped parapets and setbacks. Deep, shaded
  arcades at street level.
- **Carved relief, original motifs.** Strata bands (canyon sediment), stepped mesas (a canyon
  skyline), water ripples and sun rays, all shallow and in a shade deeper than the wall. No
  sacred or culturally specific designs: no stepped diamonds, no copied glyphs.
- **Grown timber and modern structure together.** Living grown-wood ribs (the scrub oak's
  bark) carry pergolas, balconies, the bandshell and the gallery's fanlight. Titanium and
  graphene do the precise work: frames, rails, rail track, solar leaves.
- **Every suitable surface supports life**, but people can still walk: planted roofs, sill
  planters, living walls, vine curtains, trees in plazas and promenades. Walking widths stay
  clear: promenades 3.4 m, walks 3 m, bridges 3.4 m between parapets.
- **Water follows the camp's first stream.** The approach keeps a stretch of the original
  mud stream as a relic in the paving. It becomes the canal at a stone head and spills over a
  weir. The floating five-line staff follows the canal's centreline, crisp over clear water.
- **Civic landmarks to navigate by.** The concert hall's bandshell (three nested shells), the
  gallery's sun-ray portal, the elevated light rail, and one distant pale monument: three
  leaning fins holding a ring, an abstract sun gate.
- **Dancers** are the four existing silhouette bodies, tinted solar amber, clay coral, leaf
  teal or dusk violet. Paths and plazas keep their silhouettes clear against both sky and glow.

Taken out of the concept images on purpose: the giant moon, waterfalls pouring off buildings,
neon trim on machines and buildings, hard light strips, and the visual noise of everything
glowing at once.

## Light

- **Living light is jellyfish and fairies**: a bright opaque core plus a separate translucent
  halo shell (alpha 0.2 to 0.4). Every core and halo is its own mesh and material, authored at
  peak; the game's audio clock drives breathing. Nothing animates on its own timer.
- **Accents belong to flora**: bells are cyan `#33ffd9`, vein leaves magenta `#ff4dd9`,
  tendrils green `#8cd933`, motes all three.
- **Machines carry amber `#ffb350` only**: the instrument pedestal, the pump house, the
  water gate and the lift's call point.
- **The staff is amber too**, matching the stream staff the game already draws
  (`create_stream.py`); the game brightens it as the water clears.
- **Street light is pale lantern bulbs** (`#ecd8e3`, the fairy-light language): bollards,
  door lanterns, bridge lamps, stage footlights, a string of lanterns under the bandshell.
- **Day, dusk and night share identical geometry.** Fog, sky and exposure are the runtime's
  lighting keys (`assets/lighting/lighting_keys.json`); nothing is baked into the models.
  Bloomed fog density 0.0008 to 0.002 is a runtime value, not a Blender volume.

## The modular contract

- Metres, Blender Z up, exported Y up. 1 m grid, 0.25 m detail, **4 m bay, 4 m floor to
  floor**. Doors 2.1 m. Walls 0.4 m. Slabs 0.3 m, with the top at the origin's level.
- **Linear modules** run along +X from an origin at ground level on the left front snap
  corner; their front faces Blender -Y (glTF/Bevy +Z) and their depth goes into +Y. Facades,
  slabs, walks, banks, parapets, rails and staff all follow this.
- **Standalone pieces** (plants, props, columns, the monument, the hall) sit on the centre of
  their base.
- **Turning a run**: rotate a module by 180° and place its origin at the run's far end. Side
  runs rotate by -90° (west) or +90° (east), as `create_city_district.py`'s `block()` does.
- **Corners and ends** wrap rather than abut: `city_facade_corner` is a rounded buttress centred
  on the footprint corner, and `city_facade_end` is a pilaster centred on a run's free end.
  Both enclose the cut wall ends, so the kit has no coplanar seams at corners.
- **The canal** is 8 m between wall faces: two 4 m water tiles, or one 8 m tile. The bank's
  wall face is the water line (y 0). Promenade z 0, coping z 0.12, water z -0.9, bed z -2.0.
  Corners: outer (convex land, the inside of a bend) and inner (concave land, a coping block
  where two walls meet). The canal end is the head where a stream arrives.
- **Sockets** are childless empties on the root (named `Plant_*`, `Vine_*`, `Reeds_*`,
  `Display_0`, `Exhibit`, `Stage_centre`, `Recording_source`, `Instrument_*`, `Seat*`,
  `Interact`, `Audio_*`, `Door`, `Note_*`, `Mote_*`, ...), each with extras
  `accepts: <kind>`. Plants, displays and player content are never baked into structure.
- **Connectors** are recorded in the manifest, not in the model: each has a position and the
  direction the next module continues in.
- **Levels of detail**: plants and the monument carry `<id>_LOD1`. Everything else is one
  light level (12 to 2,600 triangles).
- **Collision**: simple proxies per piece in `assets/structures/city/colliders.json` (boxes
  and cylinders; stairs and ramps as tilted slabs). Render meshes never double as collision.

## Materials and acoustics

Flat, portable PBR with no textures (the style guide's rule). Materials are shared by name
across the kit (`City_<name>`). Each declares its Steam Audio preset as glTF material extras
`acoustic`, and large near-level meshes also carry it as node extras (what the game reads).

| Material | sRGB | Acoustic |
| --- | --- | --- |
| Plaster | `#d6c2a8` | brick (lime over rammed earth, as the cottage) |
| Relief, Clay | `#bba187`, `#a85b43` | brick |
| Stone, Monument | `#c9b9a3`, `#eadbd2` | rock |
| Paving, StoneDark | `#c9b9a3`, `#9c8a74` | concrete |
| GrownWood, Timber | `#4b4037`, `#8a7a62` | wood |
| Titanium, Graphene | `#b8bcc2`, `#1a1c1f` | metal |
| Glass, Water | `#7fa6a3`, `#2f8a84` | glass (as the cottage's rain tank) |
| Soil, Moss, Bed, Mud, Crust | `#302116` ... | gravel |
| Canvas | `#cfc6b2` | carpet |
| Leaves, grass, agave, blooms, every glow | | none: living and small parts stay out of the acoustic scene |

## Where the repository overrides the prompt

The prompt proposed defaults "pending repository inspection". These were changed to fit the
repository, or are blocked on runtime work that belongs to the crates:

1. **Layout.** The prompt suggested `art/city/{source,scripts,exports,review}`. Instead the kit
   uses the repository's family layout: `tools/blender/` (scripts),
   `assets-src/blender/structures/city/` (sources), `assets/structures/city/<id>/` (glb,
   profile, prefab), and `previews/structures/city/`.
2. **Acoustic extras.** The prompt asked for material extras. The game reads node extras on
   large near-level meshes (`model_part_bindings.rs`), so the kit writes both. Materials
   without a class say `"none"`; their profiles say `null`, as the other families do.
3. **City pieces are scenery, not bridged placeables.** AGENTS.md's three lines for a new
   placeable kind were tried and measured: bridging the district's ~4,900 placements to
   `PlacedAsset` stalled its first frame past 20 s. Without it, the single-level pieces'
   acoustic node extras still reach Steam Audio (`prefab_presentation` reads them on any
   model), and `stubby_jackdaw/src/city_colliders.rs` makes their walls solid from
   `colliders.json` by the model's `GltfSource` path. The flattened monument is the one piece
   whose acoustics are not read.
4. **Colliders: walls block, floors are stood on.** `colliders.json` follows the other
   families' format, plus `tilt_about_x_radians` (stairs and ramps as tilted slabs) and
   `walkable`. Every shape becomes a static `Solid` collider in the game
   (`city_colliders.rs`); the player's body (bevy_ahoy) stands on colliders, so the raised
   City's paving, decks, bridges and stairs are walked on. The canal bed is a floor too.
5. **The ground has no cut for canals.** The canal sits below the promenade. The game's ground
   would cover it, so the Jackdaw review scene (`assets/world/asset_reviews/city.bsn`) stands
   the district 2.3 m up. A terrain cut or a raised city podium is runtime and level work.
6. **Instancing.** In Blender the district links each piece's asset collection, so it holds
   no copies. In the game, every placement is a Jackdaw prefab instance (`IsA`).
7. **Glow.** Blender's glare and the game's bloom are separate. The preview renders add a
   compositor glare at night for review only.

## Continuity checklist

- [ ] Facade variants A (plaster) and B (garden glass) share the floor band at 3.7 to 4.0 m
      and stack on the same grid.
- [ ] Corners and ends wrap; no coplanar faces at seams.
- [ ] The canal is 8 m between faces everywhere; bridges bear on both copings.
- [ ] No machine light that is not amber; no flora light that is amber.
- [ ] Every core has a separate halo; no glow strips.
- [ ] Doors are 2.1 m; walks 3 m deep; parapets 1.1 m; guardrails 1.1 m.
- [ ] The district reads from the approach: rail, bandshell and monument are visible.
- [ ] Day, dusk and night are the same geometry.
