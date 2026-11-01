## Why
The pieces of the opening exist as assets and specs: the lightning bug, the mudpit
burbler, streams, reeds, the dormant scrub oak and the desert props. Nothing yet says
how they sit together in the region where the player starts, or what the simulation
does there from minute one. This change composes the **opening region**: the starting
conditions, what is placed where, and the loop the world runs.

It is a simulation, not a level. The region has no goal, completion state or ending.
Its systems keep running, and the procedural world carries on past its edge.

## What Changes
- Starting conditions: silent windstorm, winter, bleak light quality, every scrub oak
  dormant.
- Region contents and placement: spawn, lightning bugs near a dormant oak grove,
  a mudpit burbler, reed beds around wet ground, and scattered desert props, laid out
  by placing Jackdaw prefabs in the opening scene.
- Placement constraints that keep the opening playable for every seed. Spawn is out of
  earshot of any bug. The first stream is affordable from nearby reeds. A dormant oak
  is within a few stream segments of the burbler.
- The loop the region runs: bugs → square wave → burbler voice → stream carries the
  music → clean water wakes oaks → oaks bloom and produce bugs, and growth raises light
  quality so the region blooms outward.

## Capabilities
### New Capabilities
- `opening-region`: the starting region's conditions, contents, placement constraints
  and running loop.

## Impact
World generation and placement (`crates/stubby_game`), the Jackdaw opening scene, and
the existing `add-desert-opening` discovery spec, whose "Clock" is now the burbler.
Depends on `add-mudpit-burbler`, `add-reeds`, `add-scrub-oak-production`,
`add-resource-replenishment` and `add-fluid-quality-and-viscosity`.
