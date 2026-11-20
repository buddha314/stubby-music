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
