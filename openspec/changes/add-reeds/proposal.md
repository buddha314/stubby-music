## Why
Streams need a building material, and it should never be the bottleneck. Reeds are
that material: very easy to get, like leaves in Satisfactory. They grow in clumps on
wet ground, come back soon after harvest, and are the ingredient for stream segments:
6 reeds build one segment, and the reeds then outline the segment's banks.

## What Changes
- Add the reed clump: a harvestable resource node that yields `reed` items with E, is
  cut to stubble, and returns after a short elapsed-time replenishment interval.
- Add the reed clump asset: seeded variants, per-instance variation, standing,
  harvested and regrowing states, dry-to-green colour, sway and a far LOD
  (`tools/blender/create_reeds.py`, `assets/flora/reeds/`).
- Recipe: **6 reeds → 1 stream segment.** Laying a segment from the plan spends them.
- The stream segment asset is outlined by reeds along both banks, dry while the
  water is muddy and green as it clears.

## Capabilities
### New Capabilities
- `reeds`: a frequent renewable resource on wet ground.

## Impact
Resource catalog (a `reed` item, a `reed_clump` resource with a timed replenishment
policy from `add-resource-replenishment`, and the stream segment recipe), stream
building in `add-mudpit-burbler`, world placement and the reed assets.
