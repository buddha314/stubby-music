## Why

Laying a stream to a scrub oak ran it straight through the trunk (user, 2026-11-18,
screenshot): the stream's end goes wherever the player aims, and a tree offers nothing to
snap to. In Satisfactory a conveyor snaps to a machine's input; a stream should snap to
the tree it waters and run alongside it.

## What Changes

- A new contract, `stubby_core::StreamInlet { radius_m, snap_m }`: a ring round its
  owner where a laid stream can end.
- Stream laying (`stubby_streams`): aimed inside the ring or within `snap_m` of it, the
  planned stream's end snaps to the ring point nearest the aim, arriving along the ring's
  tangent, so it passes alongside the owner and never through it.
- Every scrub oak carries an inlet (the game's glue), its ring from the oak's profile
  (`assets/flora/scrub_oak/asset.json` `stream_inlet`: 2.2 m, snapping within 2 m), inside
  the oak's watering reach (`water_reach_m`, 3 m), so a stream laid there waters it.
- Watering stays by proximity. The Godot game's explicit root inlets (connection
  ownership, the tree's beat pulse; `add-tree-inlets-and-pulse`) remain later work.

## Capabilities

### New Capabilities
- `tree-inlets`: where a laid stream snaps beside a tree.

### Modified Capabilities
None (stream building and watering are in-flight changes; this adds to them).

## Impact

`stubby_core` (contract), `stubby_streams` (planning), `stubby_game` glue, the oak profile
and its generator `tools/blender/create_scrub_oak.py`.
