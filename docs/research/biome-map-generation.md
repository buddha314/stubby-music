# Biome map generation: Factorio, Satisfactory, Farthest Frontier, and us

Researched **2026-11-18**, at the user's request, to start the procedural-generation thread:
the game has one finite opening scene, and the mythic regions (`docs/design/design-guide.md`,
*Mythic regions*) need a full map with the biome groups **interspersed**. This note
records how three builder games make their landscapes and proposes how we adapt them. It
approves nothing; the design belongs in an OpenSpec change (`add-mythic-regions`, or a new
`add-biome-map`). It extends `seeded-world-generation.md` (2026-11-01, seeds and
reproducibility), which this note does not repeat.

Facts are cited. Anything under *For us* is a proposal.

## What we need the map to do

From the design guide and AGENTS.md, as of today:

- **Four biome groups**: desert (Mojave and US Southwest, Aboriginal Australia),
  Mediterranean, temperate forest (Ireland and England, Pacific Northwest), tropical (Africa,
  perhaps Southeast Asia). Each region is the home of music deities and creatures from its
  group; **about a dozen figures** in all.
- **Interspersed**, not one biome per map and not concentric rings.
- **A desert start**: the authored opening (spawn, burbler, grove, echo grotto) stays as it
  is, and the game opens silent, so spawn is out of hearing range of every sound.
- **Seeded and deterministic** (AGENTS.md, *Procedural generation*).
- **First-person and readable**: Satisfactory meets Inside, so places must be memorable.
- **Bleak until it blooms**: every biome starts hazy and desaturated, and light quality
  rises locally around foliage. Biomes must differ even while they are still dead.
- **A journey**: the vardo travels from one region to the next, so regions must connect by
  routes a cart can take.

Today's ground is a flat mudflat of Voronoi plates, ±108 m around spawn
(`stubby_world/src/terrain.rs`: `height()` varies by 4.5 cm). There is no elevation and no
`noise` crate in the workspace yet, although AGENTS.md's stack table lists `noise` 0.9.

## Factorio: a stack of noise fields, read everywhere

Factorio generates an unbounded map chunk by chunk from **noise expressions**: formulas over
`x`, `y` and constants such as `map_seed` that can be chained, so one field can read
another. At surface creation each surface's expressions are simplified, constant-folded and
compiled into three programs (tiles, cliffs, entities and decoratives), then evaluated per
chunk. [FFF #390][fff390]

- **Fields.** Elevation (zero is the coastline; water and cliffs read it) plus three climate
  axes: **moisture, temperature and aux**, where aux is "terrain type", e.g. yellow versus
  red desert. A fourth discrete *noise layer* gives small-scale patchiness. Each axis has
  frequency and bias controls in the map settings. [FFF #401][fff401],
  [noise-expression docs][factorio-noise]
- **Placement as volumes.** Each tile, tree and decorative is placed by a probability
  expression over those axes: a volume in climate space. Earlier entries block later ones,
  so overlapping volumes starve a decorative; Wube visualised the volumes in Blender to find
  overlaps, and replaced the old "peaks" format so dunes "fade in more gradually over
  multiple sand types". Moisture drives tree density; zero moisture means no trees.
  [FFF #401][fff401]
- **Primitives.** Multi-octave basis (Perlin-style) noise, **ridge noise** (absolute value,
  inverted) for narrow connected land, **spot noise** (spaced cones, richest at the centre)
  for ore and volcanoes, and Voronoi functions including `voronoi_cell_id` (a random value
  per cell) and `voronoi_facet_noise` (zero on cell boundaries). [FFF #390][fff390],
  [noise-expression docs][factorio-noise]
- **Starting area.** A distance cone from the origin keeps spawn on land. A separate slider
  sets starting-area moisture "without affecting the rest of the world", because a forest
  start is easier than a desert one. [FFF #390][fff390], [FFF #401][fff401]
- **Natural paths.** Land-bridge paths along ridge lines, canyon paths along inverted
  ridges, and forest trails where trees would block movement are "combined and subtracted
  from both cliff and tree placement". [FFF #401][fff401]
- **Performance.** Chunk generation cost is a design constraint: Vulcanus went from
  18.35 ms to 2.83 ms per chunk by special-casing grid-aligned basis noise. [FFF #390][fff390]

**Lesson:** biomes are not regions anyone draws. They are where the climate fields happen to
fall, so they intersperse naturally and blend softly, and everything (tiles, trees, cliffs,
ore, paths) reads the same few fields. The cost is control: noise does not promise that a
given biome appears, how big it is, or what lies next to it.

## Satisfactory: hand-made biomes, seeded resources

Satisfactory's world is **hand-crafted with no procedural terrain**: 21 named biomes on a
playable island of about 5.4 × 5.4 km, four of them starting areas. Resource nodes were
placed by hand per biome; caves were laid out to support trains; most of the landscape
cannot be deformed. [Satisfactory wiki: Biomes][sf-biomes]

- **Start areas are rated**, not equal: Grass Fields and Rocky Desert are for first-time
  players, Northern Forest for experienced ones (rich but steep), and Dune Desert for
  advanced players (water and foliage are rare, so early power is hard). A start does not
  limit where you go. [Satisfactory wiki: Biomes][sf-biomes]
- **Biomes are memorable set pieces**: Spire Coast, Titan Forest, Jungle Spires, Blue
  Crater. Neighbours are composed: Desert Canyons sits between Spire Coast and Northern
  Forest, and a tunnel cave links Western Dune Forest to Rocky Desert. [Satisfactory wiki:
  Biomes][sf-biomes]
- The level designers' problem was translating level design tools "into something you can
  build with nature", and the team talks about optimising the player's journey across an
  enormous map. [Gamewatcher interview][sf-interview]
- **Update 1.2 (2 June 2026) keeps the terrain and randomises the resource layer**:
  Resource Node Randomization (Default, Random, Basic/Advanced/Fossil-Fuel Rich), Node
  Purity (All Pure … All Impure, Random) and a shareable World Seed. The seed alone does not
  reproduce a world; other settings must match. [Satisfactory wiki: Update 1.2][sf-12]

**Lesson:** what makes Satisfactory's places memorable is authorship: composed neighbours,
signature landmarks, start areas tuned for difficulty. Its own seed randomises only the
layer on top of the authored terrain.

## Farthest Frontier: one biome per map, chosen by preset

Crate Entertainment publishes nothing about the generator's internals; what is known comes
from players and the community wiki, so treat the details as observed, not documented.

- **One biome per map**, chosen at creation: Idyllic Valley (default, easy), Lowland Lakes
  (easy), Plains (medium, fertile, little stone and water), Alpine Valleys (medium,
  mountainous, ore-rich, scarce water and clay), Arid Highlands (hardest: infertile patches,
  scarce water, plentiful stone, sand and gold). [Farthest Frontier wiki][ff-wiki]
- **The biome is a difficulty and resource profile**, not just a look: it sets fertility,
  water, trees, forage, ores and wildlife, and changes what the player must solve first.
  [Farthest Frontier wiki][ff-wiki]
- **Sizes are presets** of roughly 1, 2.8 and 5.3 km² (the wiki's own measurements). The
  seed is 11 hex characters, and players observe that different digits drive terrain shape,
  the biome and height. Changing size or biome re-rolls the seed. [Farthest Frontier
  wiki][ff-wiki], [Steam discussion][ff-steam]
- **Seeds drift between patches**: players report a known seed no longer rolling the same
  after an update. [Steam discussion][ff-steam]

**Lesson:** a biome should carry a gameplay profile (what is scarce, what is plentiful, how
hard it is) as data, and the generator's version is part of the seed's meaning (the 2026-11-01
dossier already requires this).

## Two general techniques worth borrowing

- **Climate lookup.** Red Blob Games picks a biome from two noise maps, elevation (as a
  stand-in for temperature) and moisture, through a Whittaker-style table, and notes that
  one field is not diverse enough but two is "pretty good". The method is local per point and
  "doesn't enforce global relationships between regions". [Red Blob Games][redblob]
- **Nearest point in climate space.** Minecraft Java 1.18+ defines each biome as intervals
  over six parameters (temperature, humidity, continentalness, erosion, weirdness, depth)
  and, outside every interval, takes the closest one. The same noises shape the terrain, a
  "soft link between biome and terrain": mountains tend to be mountain biomes.
  [Minecraft wiki][minecraft]

## For us: a proposal

None of the three alone fits. Factorio intersperses well but promises nothing per region;
Satisfactory's places are memorable but there is one map; Farthest Frontier has one biome
per map. Our regions are **named homes for a dozen authored figures**, so we need guarantees
(every group present, a desert start, a figure's site in every region) that pure noise
cannot give, and the variety that hand-authoring cannot. The proposal is a **region graph
for guarantees, climate fields for texture, authored stamps for memory**:

### 1. Region graph (the guarantees)

- A **finite map** split into jittered-grid **Voronoi regions** at landscape scale, the same
  construction as today's mud plates (`terrain::site`, `terrain::plate`) at roughly a
  thousand times the size. For example, a 4 × 4 km map with cells about 700 m across gives
  around 30 regions; size is an open question below.
- **Assign a biome group per region** by sampling the climate fields (below) at its centre
  and taking the nearest group in climate space, Minecraft-style. Climate continuity then
  makes plausible neighbours (desert beside Mediterranean, temperate beside tropical
  only rarely) without an adjacency table.
- **Then enforce the guarantees** with a small seeded repair pass: the spawn region is
  desert; every group has at least *k* regions; no group forms one giant block. Re-assign
  the cheapest cells (closest in climate space to the needed group) until the guarantees hold.
  This is the part Factorio does not do.
- **Warp the boundaries** with low-frequency noise so regions do not read as polygons, and
  blend across an **ecotone band** (tens of metres) using distance to the cell edge,
  Factorio's `voronoi_facet_noise` idea.
- The graph is computed once for the whole map, at load, from the seed: tens of cells
  are trivial to compute and give every system a stable region ID.

### 2. Continuous fields (the texture)

Pure functions of `(seed, x, z)`, so they are chunk-order independent:

- **Elevation**: low-frequency multi-octave noise, plus **ridge noise** for spines and
  canyons, flattened by a **starting-area cone** around spawn so the authored opening stays a
  flat mudflat (Factorio's cone).
- **Temperature and moisture**: choose the region's group and vary flora within it.
- **Aux**: the variant within a group (Mojave versus red-centre desert; Irish versus Pacific
  Northwest forest), Factorio's terrain-type axis. This is how "small regional
  differences" within a group (design guide) become data.

**Placement as climate volumes**: every reed bed, shrub, tree variant and rock declares
ranges over moisture, temperature and aux, as Factorio's decoratives do, in a data file
(a rule type, so the rules designer could tune it). Reuse `jackdaw_terrain`'s scatter
for candidates, as `add-hybrid-world-placement` already plans, with the climate volume
as its eligibility mask. Check overlaps the way Wube did: if two entries share a volume,
one never places.

**Resources by spot noise**: burblers, scrub-oak groves and reed beds as spaced cones (spot
noise) weighted by moisture and region, so wet ground carries reeds (AGENTS.md: reeds are
abundant on wet ground) and every region gets its clock sources.

### 3. Authored stamps (the memory)

Satisfactory's lesson. Each region gets **one hero site**, a hand-made Jackdaw prefab cluster
(a shrine with its figure carved in rock or a tree, like the echo grotto), placed by the
generator at the best-scoring spot in the region (elevation, slope, distance from the edge)
with a cleared, flattened footprint. The **opening is the first stamp**: the existing scene,
protected by its silence radius. Region identity comes from the stamp; the fields fill the
space between. This follows `add-hybrid-world-placement`'s rule that authored placements
win over generated ones.

### 4. Routes (the journey)

Factorio's natural paths, for the vardo: connect neighbouring regions' hero sites along the
region graph's edges, route each path through low ground (canyons) or along ridges, and
subtract the path from boulder, cliff and tree placement so a cart can always pass. A
route is a stable path the narrative can use ("follow the canyon east to the grove").

### 5. Biomes as gameplay profiles

Farthest Frontier's lesson. Each group carries data: what is scarce, what is plentiful, the
acoustic character (desert rock is reflective, forest absorbs and scatters; see
`docs/audio/acoustic-level-authoring.md`), and the waveform sources and figures it holds.
Rates and recipes stay in `assets/recipes/`; nothing here invents a waveform source (AGENTS.md).

### Determinism

Everything above follows `seeded-world-generation.md`: seed plus generator version plus
settings, separate seeded streams per purpose (graph, fields, scatter, stamps) so adding a
decorative does not move a shrine, and fields sampled in global coordinates. Farthest
Frontier's seed drift is the warning: bump the generator version whenever output changes.

## Open questions for the user

1. **Map size and shape.** Finite and bounded (Satisfactory 5.4 km square, Farthest Frontier
   up to about 5 km²), or unbounded chunks (Factorio)? A finite map makes the guarantees
   above simple; this note assumes finite, with chunked meshes for culling and streaming.
2. **How many regions per group**, and does the start stay a single desert region, or can
   the player start in another group as Satisfactory offers four starts?
3. **Elevation and streams.** Real hills change stream laying: does mud flow downhill, and
   may a stream climb? Today everything is flat.
4. **Edges.** What bounds the world: mountains, sea, the windstorm thickening into a wall?
5. **Bleak before bloom.** What tells a dead Mediterranean region from a dead desert: rock
   colour, landforms, the shapes of dormant plants? `assets/lighting/lighting_keys.json`
   would need keys per group.
6. **Do regions change with progress?** Factorio's map is fixed once generated; ours
   blooms. Does blooming only recolour and grow, or can it reshape terrain (water returning
   to dry lakes)?

## Sources

- [Factorio: Friday Facts #390, Noise expressions 2.0](https://factorio.com/blog/post/fff-390)
- [Factorio: Friday Facts #401, New terrain, new planet](https://factorio.com/blog/post/fff-401)
- [Factorio: noise-expression reference (2.0.76)](https://lua-api.factorio.com/latest/auxiliary/noise-expressions.html)
- [Satisfactory wiki: Biomes](https://satisfactory.wiki.gg/wiki/Biomes)
- [Satisfactory wiki: Update 1.2 notes](https://satisfactory.wiki.gg/wiki/Update_1.2)
- [Satisfactory: level designers' interview, Gamewatcher](https://www.gamewatcher.com/satisfactory-developers-hannah-beuger-rosario-baarh-interview)
- [Farthest Frontier wiki: World Generation (player-measured)](https://farthestfrontier.wiki/wiki/World_Generation)
- [Farthest Frontier: players' seed findings, Steam](https://steamcommunity.com/app/1044720/discussions/0/3315233099231143818/)
- [Red Blob Games: Making maps with noise functions](https://www.redblobgames.com/maps/terrain-from-noise/)
- [Minecraft wiki: World generation (Java 1.18+)](https://minecraft.wiki/w/World_generation)

[fff390]: https://factorio.com/blog/post/fff-390
[fff401]: https://factorio.com/blog/post/fff-401
[factorio-noise]: https://lua-api.factorio.com/latest/auxiliary/noise-expressions.html
[sf-biomes]: https://satisfactory.wiki.gg/wiki/Biomes
[sf-12]: https://satisfactory.wiki.gg/wiki/Update_1.2
[sf-interview]: https://www.gamewatcher.com/satisfactory-developers-hannah-beuger-rosario-baarh-interview
[ff-wiki]: https://farthestfrontier.wiki/wiki/World_Generation
[ff-steam]: https://steamcommunity.com/app/1044720/discussions/0/3315233099231143818/
[redblob]: https://www.redblobgames.com/maps/terrain-from-noise/
[minecraft]: https://minecraft.wiki/w/World_generation
