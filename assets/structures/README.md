# Structures

## Campfire (`tools/blender/create_campfire.py`)

The camp and craft bench (`add-campfire-craft-bench`): a bundle of fourteen sticks over
ash. Built with Q for 25 reeds (`buildables.json`); its `placement` is `beside_stream`,
so aimed within its clock reach (5 m, `machines.json`) of a stream it snaps beside the
bank and joins it, borrowing that stream's clock.

Unlit, E opens its rack: add the snail-shell sequencer and set gates on 2 and 4; a whole
bar heard that way lights the fire, which burns while its clock runs and dies to embers
when it stops. Burning, E opens the **crafting screen** (Satisfactory's Craft Bench,
`stubby_game/src/crafting_screen.rs`), where recipes are made at the tempo the fire
hears; the RACK tab returns to the rack. Nodes: `Fire`, `Embers` (hidden by
`start_scale: 0`), `Light_fire`, `Sparks`, `Interact`; LOD0/LOD1.

## City kit (`tools/blender/create_city_kit.py`)

Phase 4's modular construction kit (`add-city-kit`): 86 placeable pieces under `city/`, each in
its own folder (`city/<id>/<id>.glb`, prefab, profile), on a 4 m grid: foundation and paving,
facades in two variants, floors and roofs, outdoor walks, stairs and lifts, the canal and its
bridges, terraces, light rail, gallery, concert hall, monument, planting, luminous flora, the
staff, street furniture and machines. `city/asset.json` indexes them and `city/colliders.json`
holds their collision proxies. Brief: `docs/art/city-kit.md`; regeneration, snapping and the
manifest: `docs/art/city-kit/README.md`.
