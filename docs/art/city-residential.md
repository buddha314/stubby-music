# The City, lived in: mixed-use residential expansion

The second City brief (`docs/art/prompts/city-residential.md`) extends the City kit
(`docs/art/city-kit.md`) so the city reads as a place where people live, work and shop,
not a resort. It adds shops on the street, offices above, homes above those, shared roof
gardens, and the residents' own way in and up. The kit's grid, materials, light rules and
pipeline are unchanged. The references are in `docs/art/city-kit/residential/references/`
(CC0, the user's). Change: `openspec/changes/add-city-residential/`.

## The building grammar

Every building is the same stack on the 4 m bay and 4 m storey:

| Storey | Use | How it reads without labels |
| --- | --- | --- |
| Ground | **Shops**, cafes, the residents' door | Shopfronts (stallriser, display glass, sign band, awning), stocked shelves, cafe tables outside. The residents' door is recessed and solid, with letterboxes, a plaque and a bench: plainly not a shop. |
| 1-2 | **Offices** | A continuous ribbon of glass from desk height with slim mullions, a brise-soleil, workstations and screens inside. No balconies. |
| 3 and up | **Homes** | Smaller, separate windows in deep reveals; sliding balcony doors; loggias; solid privacy bays. Curtains in three colours, balconies with chairs, laundry, planters, shade cloths and screens between neighbours. Lamps glow warm at night. |
| Roof | **Shared garden** | Planted and decked tiles, a greenhouse, raised beds, a long table, pergolas, the stair head. |

- **Homes are not showrooms.** The home bays are mostly wall: the window pair is 3.2 m² of
  glass in a 16 m² bay, and every building mixes privacy bays and screens into its homes.
  Offices get the long glass.
- **Every home is individual.** A seeded rule picks each bay's type, its curtains and what
  stands on its balcony, so no two storeys match while every piece repeats.
- **Residents' circulation is separate.** The street door leads to a lobby with
  letterboxes, then a glass spiral stair tower with a lift beside it. Every home storey
  opens onto the tower through an apartment door, and a stair head lets residents onto the
  roof. Public bridges and promenades stay at street level; no public route passes a
  living room.
- **Domestic light is warm and sparse.** It is a pale hearth colour (`#ffd2a1`), separate
  from the machine amber (`#ffb350`, still only on machines: the lift's call point). Each
  room has one or two lamps; windows are not lit panels.

## The four buildings, from one kit

All four come from `tools/blender/city_mixed.py`'s `mixed_block`, with different arguments:

| Building | Where | Shape |
| --- | --- | --- |
| Street wall | the canal's north bank, x 40-56 | 4 x 3 bays, 8 storeys, pier corners, stair tower behind; three furnished homes |
| Rounded corner block | facing the plaza and the canal's bend | 5 x 4 bays, 8 storeys, curved shop, office and home corners with curved balconies; a shared courtyard behind |
| Stepped garden tower | the new inland street | 3 x 3 bays, 10 storeys, stepping back at storeys 4 and 7 onto garden terraces |
| Taller tower | the inland street, across a shared courtyard | 3 x 3 bays, 12 storeys |

A distant skyline of ordinary towers uses `city_far_tower_storey`.

## Delta manifest: the prompt's IDs mapped to pieces

Reused from the City kit (not rebuilt): slabs, roofs, parapets and corner piers, the arcade,
stairs and lift, guardrails, garden deck, pergola, benches, planting, glow flora, paving,
and the canal pieces. New pieces carry priority `RES` in the manifest
(`docs/art/city-kit/manifest.csv`):

| Prompt ID | Family | Pieces |
| --- | --- | --- |
| MIX01 | Retail frontage | `city_shop_glazed`, `city_shop_cafe`, `city_shop_corner_round`, `city_awning_{coral,teal,ochre}`, `city_shop_display`, `city_bistro_set`; open arcade: existing `city_facade_arcade` |
| MIX02 | Office frontage | `city_office_glazed`, `city_office_corner_round`, `city_office_service`, `city_sunshade_4`, `city_workstation`, `city_meeting_table` |
| RES01 | Residential facade | `city_res_window_pair` (+ `_b`), `city_res_slider` (+ `_b`), `city_res_loggia`, `city_res_privacy`, `city_res_corner_round`, `city_res_door`. Variant A plaster; B timber battens or clay wash |
| RES02 | Balcony kit | `city_balcony_res_4`, `city_balcony_round`, `city_balcony_end`, `city_balcony_planter`, `city_privacy_screen`, `city_shade_fabric`, `city_laundry`, `city_balcony_chairs`; recessed: `city_res_loggia`; rail: built into the balcony (and the existing `city_guardrail_4`) |
| RES03 | Dwelling shell | `city_room_floor_4`, `city_partition_4`, `city_partition_door`, `city_slab_round`, `city_roof_round`; studio, one-bedroom and family homes are assemblies (`furnished_home`) |
| RES04 | Domestic interior | `city_sofa`, `city_armchair`, `city_coffee_table`, `city_dining_set`, `city_bed`, `city_kitchen`, `city_bookshelf`, `city_curtain_{linen,coral,teal}`, `city_floor_lamp`, `city_pendant_lamp`, `city_rug`, `city_laundry` |
| RES05 | Access core | `city_res_entry`, `city_mailboxes`, `city_core_spiral`, existing `city_lift`, `city_res_door` (the door off the stair tower's landing), `city_roof_access`; outdoor access galleries: existing `city_walk_bay` + `city_guardrail_4` |
| RES06 | Communal terrace | `city_greenhouse`, `city_raised_beds`, `city_communal_table`; existing `city_garden_deck`, `city_pergola`; courtyards are assemblies |
| RES07 | Building assemblies | `mixed_block` in `tools/blender/city_mixed.py`: street wall, rounded corner, stepped terrace, taller tower |
| RES08 | Distant facade | `city_proxy_home_{living,bedroom,kitchen}`, `city_proxy_office`, `city_proxy_shop` (room sets behind windows), `city_far_res`, `city_far_office`, `city_far_tower_storey` |

## Contract additions

- Rounded corners occupy their 4 x 4 m corner cell, with the arc centred on the cell's inner
  corner (4, 4) and a radius of 4 m. Runs continue at x 4 (front) and y 4 (side). Each storey
  takes a rounded slab, and the roof takes `city_roof_round`.
- Interior sets and furniture snap to the facade bay's origin (sets) or their own base centre
  (furniture). They never sit inside a structural mesh. A home storey is a slab, then a timber
  floor, then partitions, then furniture.
- The stair tower is one 4 x 4 m cell per storey, outside a building's side or back. The bay
  facing it is an apartment door (`city_res_door`).
- New materials keep the acoustic rule. Fabrics (curtains, awnings, upholstery, bedding,
  rugs, laundry) are carpet, interior limewash brick, rails metal, furniture wood. Small
  goods, plants and lamps are none.

## Where the repository overrides the prompt

1. **Layout**: the pieces join the City family (`assets/structures/city/`, sources in
   `assets-src/blender/structures/city/`), not a new folder tree.
2. **Acoustics**: as in the City kit, on materials (verified per glb) and on large meshes.
3. **Not walkable in the game yet**: homes, balconies, stairs and roofs carry collision
   proxies (`colliders.json`), but the game cannot stand on colliders yet
   (`add-prefab-colliders`). The furnished homes are review sets, not playable rooms.
4. **The game does not bind the pieces** (`city_` in the prefab bridge and model-part
   bindings), so acoustics and sockets (`Interact`, `Door`, `Seat*`) are data only.
   These are tasks 5.x in `add-city-kit`.
5. **Ground level**: the validation bay's review scene stands on the game's ground. Its paving
   sits exactly at ground level, so the mudflat shows through it in the game.

## Measured

From the manifest and `docs/art/city-kit/district_layout.json` (near level, before LOD):

| | Placements | Triangles |
| --- | --- | --- |
| The four mixed-use buildings and their courtyards | 2,034 | 514,206 |
| The rest of the district | 1,263 | 252,202 |
| Planting dressed at sockets (both) | 1,221 | 438,168 |
| Skyline (far tower storeys) | 162 | 38,400 |
| Test strip | 247 | 60,372 |
| **Total** | **4,927** | **1,303,348** |

The 63 new pieces total 20,446 triangles. Their range: interior sets 200 to 450, furniture 100
to 900, facade bays 150 to 600, the stair tower 1,600. Planting is the largest share
of the district; its far levels (`city_tree`, `city_shrub`) and Jackdaw's LOD switching are
what will hold it down. FPS and draw calls have not been measured in the game: the pieces are
not bound yet (`add-city-kit` 5.1), and a 20-second review run does not measure it. Eevee
renders a 1280 x 720 review view in about 30 s on `breq`; that is a preview cost, not a
runtime one.
