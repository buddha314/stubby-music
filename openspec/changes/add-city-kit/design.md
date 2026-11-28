## Context

The brief (`docs/art/prompts/city-kit.md`) proposes defaults "pending repository inspection".
The repository's own conventions take precedence. The resulting contract, and each place the
brief was overridden, are recorded in `docs/art/city-kit.md`. This design notes only the
decisions behind them.

## Decisions

- **One family, one library.** All pieces live in `structures/city/` with the prefix `city_`
  (plants and props included), so one asset library and one catalog file cover the kit, and
  the game can later bind the family with one prefix test.
- **One piece per placeable asset.** Each module is its own glb and prefab. Levels assemble
  them as prefab instances, and the Blender district as linked collection instances. The
  layout is a single list that both share (`district_layout.json`).
- **A grid, not free-form.** 4 m bays and storeys, linear modules from a left front corner,
  standalone pieces from the base centre. Corners and ends wrap the cut ends of walls, so
  seams are never coplanar.
- **Acoustics both ways.** Materials carry `acoustic` extras, as the brief asks, and large
  near-level meshes carry node extras, which is what the game reads. Both use only the
  presets in `stubby_core::AcousticOccluder::preset`.
- **Data, not baked content.** Plants, displays, exhibits, performers, recordings, notes and
  motes are sockets with an `accepts` kind. The district dresses plant sockets by a seeded rule.
- **Light is the style guide's.** Cores and halos are separate meshes, authored at peak.
  Flora uses the bloomed accents, machines and the staff use amber, and streets use pale
  lantern bulbs.

## Risks / Trade-offs

- Flat PBR with no textures keeps the kit light, but up close the facades are plain; a trim
  sheet can come later without changing the grid.
- Halo shells add overdraw in dense glow (tendrils, motes). Sparse placement is the
  mitigation until the runtime is measured.
- The pieces are not walkable or acoustic in the game until section 5 lands.
