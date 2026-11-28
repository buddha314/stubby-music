## Context

The City kit's contract (4 m bay and storey, floor band, facade line, sockets, acoustic
rule) holds. This change adds uses, not a new grid. The brief and mapping are in
`docs/art/city-residential.md`.

## Decisions

- **One generator, many buildings.** `mixed_block` takes a footprint, a storey stack of
  uses, corners (pier or round), setbacks and a stair tower position. Four building types
  differ only in arguments, so they share every piece.
- **Uses read from openings, not labels.** Shops have display glass and awnings; offices a
  continuous glass ribbon and sunshades; homes small windows, sliding doors, loggias,
  privacy bays and balconies. Homes are mostly wall.
- **Inhabited, cheaply.** Most windows show a shallow room set (a proxy) behind them; a few
  homes are fully furnished. Furniture and fabric are separate pieces, never baked into
  facades.
- **A seeded rule individualises.** Bay types, curtain colours and balcony contents vary by
  seed, so no two storeys repeat while every piece is shared.
- **Validate before multiplying.** One two-bay slice (shop, residents' door, office, three
  home storeys, stair tower, roof garden) was exported, imported in the game and reviewed
  before the neighbourhood was built.

## Risks / Trade-offs

- Placement count grows: the district now has about 4,900 placements. Proxies keep each
  window cheap (tens to a few hundred triangles), but draw calls should be measured in game.
- Glass is alpha-blended, and every interior behind it adds overdraw; the far-tower proxies
  carry no glass.
