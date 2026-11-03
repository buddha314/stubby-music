## Why
The burbler's outlet has a stream asset, a hologram and a reed cost, but the player
cannot lay a stream. Streams are how music reaches plants ("Water the tree" needs one),
so laying them is the next step of the opening. The burbler change (`add-mudpit-burbler`
2.7) says "like dragging a conveyor from a generator" without fixing the controls, and
the rules have no type for what something costs to build (`add-reeds` 2.1a).

## What Changes
- **Stream build mode, Satisfactory's conveyor.** Q enters it. Aiming near a free port
  (the burbler's mouth or a laid stream's end) lights that port's halo; left click
  anchors there. The hologram then curves from the port toward the aim point; left click
  lays it if valid, and its end becomes the new anchor. Right click drops the anchor;
  Escape or Q leaves build mode.
- **Smooth curves.** A plan leaves its port along the port's flow direction and curves to
  the aim point. It is a whole number of 4 m segments, each bent along the curve.
- **Reed cost.** Each segment costs 6 reeds. While planning, the reed icon and
  `need / have` show under the reticle, red when short; the hologram turns invalid.
  Laying spends the reeds, all or nothing.
- **Hologram animation.** The plan's `Flow` chevrons scroll downstream; tints follow
  valid/invalid; the anchored port's halo shows while extending.
- **New rule type `buildables`:** `assets/recipes/buildables.json` rows give a build cost
  (`stream_segment`: 6 `reed`). Rule PR: schema, validator and `Catalog::build`.
- Out of scope (later): staff wave flow, notes and bar lines on laid streams, burbler
  bubbles, removing streams (F), saving laid streams, terrain/collision checks.

## Capabilities
### New Capabilities
- `stream-building`: laying streams from ports in build mode, their cost and hologram.
### Modified Capabilities
- `rules-data`: a `buildables` rules file with per-build item costs.

## Impact
`stubby_recipes` (new rule type, `rules-fmt` schema), `assets/recipes/buildables.json`,
`stubby_game` (new `streams.rs`: build mode, curve, mesh bending, hologram, cost HUD),
the burbler's `AssetState` (`port_highlight`, `stream_connected`).
