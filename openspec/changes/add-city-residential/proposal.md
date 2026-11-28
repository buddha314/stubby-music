## Why

The City kit (`add-city-kit`) built civic landmarks and generic blocks, but the city did not
yet read as a place where people live. The user's second brief
(`docs/art/prompts/city-residential.md`) asks for ordinary mixed-use buildings: shops on the
street, offices above, homes above those, shared roof gardens, and separate circulation for
residents.

## What Changes

- Add a residential and mixed-use extension to the City kit: retail and office frontages,
  residential facades in two variants, a balcony kit, a dwelling shell, domestic furniture
  and fabrics, an access core (street door, letterboxes, spiral stair tower, roof access),
  communal roof pieces, and interior and distant proxies (`tools/blender/city_build_res.py`).
- Add `tools/blender/city_mixed.py`: one generator for mixed-use buildings (street wall,
  rounded corner, stepped garden tower, taller tower) and furnished homes.
- Validate one complete mixed-use bay first (`create_city_mixed_bay.py`, review scene
  `city_mixed_bay.bsn`), then extend the reference district with the neighbourhood, a
  skyline, an assembly test row and residential review cameras.
- Add domestic colours to the palette: textiles, interior limewash, rails, and warm hearth
  light (distinct from the machine amber).

## Capabilities

### New Capabilities
- `city-residential-assets`: the mixed-use stack and how its uses read, circulation, the
  individualised homes, and the pieces' pipeline.

### Modified Capabilities
None.

## Impact

Art and asset pipeline only, with no crate changes. It inherits `add-city-kit`'s open crate
tasks (binding, walkable colliders). Shared City materials changed: the glass is clearer
(alpha 0.18), so existing pieces were regenerated.
