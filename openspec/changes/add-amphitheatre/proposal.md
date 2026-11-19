## Why

The user (2026-11-18): the amphitheatre is going to **drive the story now**, so its assets
come soon. It is the map of the world's beings: a natural sandstone bowl in the chaparral
whose seats fill as the player wakes beings, a guide that reveals their patterns and acts
almost like a skill tree for building the new world. Players may find it early and not
understand why it is there until later. Seats give modules back.

## What Changes

- **The amphitheatre asset family** (`assets/structures/amphitheatre/`): a slickrock bowl worn
  into tiers, framed by a natural arch, around a round dancing floor with a spring at its
  centre; chaparral scrub on the rim. Echoes of Arches National Park and Greek hillside
  theatres, landforms only. Blender source, export script, LODs, colliders, acoustic rock, a
  Jackdaw prefab.
- **Seats from the being atlas:** the export script places one seat socket per seated being
  from `docs/research/beings/layout.json`, so beings that are alike sit side by side. Each
  seat has states: empty, hinted (a silhouette with its biome and kind), filled (the being's
  carving).
- **States of the whole:** an empty, weathered bowl that reads as a landform; the dancing
  floor's spring dry, then filling as seats fill.
- **Placement:** provisionally in a chaparral pocket of the opening scene, far enough out
  to be found early by wandering, until the biome map (`add-mythic-regions`) places it.
- **Behaviour (game):** seating a woken being fills its seat, gives back a module (a rule
  row, data), hints its neighbours, and raises the spring.

## Capabilities

### New Capabilities
- `amphitheatre`: the amphitheatre as a place, its seats, and what seating a being does.

### Modified Capabilities
None.

## Impact

- Art (Claude): `assets-src/blender/structures/amphitheatre/`, `tools/blender/`,
  `assets/structures/amphitheatre/`, `previews/structures/amphitheatre/`, `assets/scene.bsn`.
- Game (Codex): a binder and seat behaviour in the game's glue; a rule type for what each
  seat gives (`assets/recipes/`, Rule PR); reads `layout.json`.
- Depends on how a being is woken (`add-deity-shrines`, not yet opened); until then a seat
  can be filled by the review driver.
- Follows `add-being-atlas` (layout), `research-music-mythologies` (sharing rules),
  `docs/art/biome-characters.md` (chaparral: hybrids on the rocks).
