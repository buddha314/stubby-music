# Design guide

How the game's world, its arc and its settings are designed. The story itself is
`NARRATIVE.md` (the user's); this guide holds the design notes that serve it: settings,
the player's base, styling, and the decisions still open. The look in detail is
`docs/art/style-guide.md`; rules and numbers are `assets/recipes/`.

Mark each note **decided** (with who and when) or **open**. Requirements still go through
OpenSpec; a note here is direction, not a requirement.

## The arc

**Decided (user, 2026-11-18):** from a desolate mudflat to a glorious solarpunk city of
the future, in the spirit of *Tomorrowland* and *Big Hero 6*'s San Fransokyo. Music and
water rebuild civilization: small villages, then towns, then whole cities.

**Decided (user, 2026-11-20):** organize the first phase reviews as Camp →
Village → Town → City. The silent mudflat is the opening before Camp.

| Phase | What the player sees | Ties to |
| --- | --- | --- |
| Camp | The vardo, a campfire, the first burblers and streams | `add-campfire-craft-bench` |
| Village | Clearer water reaches the first grove and the trees begin to change the local air | `add-reed-water-purifier`, `clean-air-by-foliage` |
| Town | Buildings and a commons grow around the water route as people return | `add-settlement-phase-reviews` |
| City | A solarpunk city grows along the same route | `add-settlement-phase-reviews` |

**Draft progression.** Camp becomes Village when there is enough clean water to
plant a crop. Village becomes Town when three villages trade goods. Town becomes
City when the player can build an art gallery, a concert hall and at least one
light rail system. The growing resident population is called **Dancers**.
Thresholds, population mechanics and how the phases map onto long-term goals
are open. [Settlement phase design](../../openspec/changes/add-settlement-phase-reviews/design.md#draft-progression-rules)

The [Dancer consumption schedule and character brief](../../openspec/changes/add-settlement-phase-reviews/design.md#draft-dancer-consumption-schedule)
cover daily rations, shelter, silhouette variants and local clocks. The
[Burb Sweeper economy audit](../research/burb-sweeper-economy.md) records
precedents for housing, shared labour, crop inputs and long-run balance.

The Jackdaw examples are review scenes. The
[long-session design previews](long-session-previews.md) describe the experience
to compare at each stop.

The [roadmap](roadmap.md) outlines what each phase adds, the pillars proposals are
checked against, the foundations to build first and an audit of the rules (2026-11-20).

## Mythic regions

**Decided (user, 2026-11-18):** the map is procedural and has regions, each the home of
music deities and creatures from one mythology, in a biome like their origin. Deities are
found drawn or carved into rocks and trees, and are rejuvenated with song. The first
setting is the **US Southwest and the Mojave**, perhaps blended with **Australian
Aboriginal** myth (both desert lands; Aboriginal songlines map country through song, much
as our streams are staff music).

Open:
- **Sources: decided (user, 2026-11-18):** the game draws only on stories their
  communities share openly, each with its source recorded. Nothing restricted (to
  initiates, to men or to women), no sacred objects or ceremonies, nothing whose sharing
  is unclear; carvings are our own designs, not copies of real rock art.
  `research-music-mythologies` holds the research and the check on each figure.
- **Regions are biomes, and figures are grouped by landscape, not culture (user,
  2026-11-18, replacing the earlier four culture-based groups).** Eight game biomes, merged
  from the WWF's 14 terrestrial biomes (Olson et al. 2001) and its freshwater types; a
  region's figures come from any culture whose homeland is that kind of landscape. Mountain
  and wetland are landforms: the map takes them from elevation and lake basins, not climate.
  The game needs **about a dozen characters** in all.

  | Game biome | WWF biomes | Landscape |
  | --- | --- | --- |
  | **Desert** (the start) | 13 deserts and xeric shrublands; endorheic basins | sand, salt and mud flats, dry lakebeds that refill |
  | **Chaparral** | 12 Mediterranean forests, woodlands and scrub | shrubs, rocky sand, scrub oak |
  | **Grassland and savanna** | 7, 8 grasslands, savannas and shrublands | open grass, scattered trees |
  | **Broadleaf forest** | 4 temperate broadleaf and mixed; 2 tropical dry broadleaf | oak, ash, deciduous woods |
  | **Conifer rainforest** | 5 temperate coniferous; 6 boreal | tall, wet, mossy conifers |
  | **Tropical forest** | 1 tropical moist broadleaf | jungle |
  | **Wetland** | 9 flooded grasslands; floodplain freshwaters | marsh, bog, reed beds around lakes |
  | **Mountain** | 10 montane grasslands; 11 tundra (alpine); montane freshwaters | bare rock, alpine meadow, mountain lakes |

- **Figures: decided (user, 2026-11-18):** unnamed, as spirits of a place, with imagery
  similar to, not identical with, their cultural heritage. Imagery gets the same check as
  stories: only openly shared motifs and styles, never a copy of a particular work.
- **Essences, not gods (user, 2026-11-18):** a figure is the essence of a kind of being,
  not a specific god: "Kokopelli before some humans called him Kokopelli". The wandering
  flute player, the echo, the teacher under the waterfall, the instrument made from a
  beloved creature: beings many peoples met and named in their own ways. The research's
  traditions are where an essence was found, not who it is. Which essences the game uses is
  **decided as we go**; the pool is the table in `docs/research/music-mythologies.md`.
- How waking a deity plays (a rack with targets, as the campfire's, is the obvious fit) and
  what it gives back (light quality around it, a region's power).
- The echo grotto's boulders as the first carving site.
- **Monsters and the amphitheatre (user, 2026-11-18):** the beings include **monsters**,
  still to research. Once the beings are sorted, a **natural amphitheatre** somewhere on the
  map shows how they cluster. Each being's traits are data in the being atlas
  (`docs/research/beings/`, `add-being-atlas`), and `uv run tools/beings/cluster.py`
  clusters them and writes the seating layout.
- **The amphitheatre is a guide and a skill tree (user, 2026-11-18):** the player finds it a
  little way into the game, and its map of the beings reveals the patterns (the kinds of
  being, what each biome's beings share) as a guide to the rest of the world. It acts
  almost like a **skill tree** for building the new world. Draft, to settle in its change:
  each woken being takes its seat, empty seats show where kin are still to be found (a
  silhouette, a biome, a kind), and neighbouring seats suggest what to look for next, as
  Satisfactory's MAM research tree shows what a discovery opens.
- **Amphitheatre decisions (user, 2026-11-18):** it stands **in the chaparral**. **Seats give
  modules back**: waking a being and seating it yields a rack module. Players **may find it
  early but not understand why it is there until later**: an empty, weathered bowl of stone
  that only reads as a map once some seats are filled. It must **rhyme with sacred gathering
  places**, the grounds where rain dances and other ceremonies are held, in form and spirit
  only: a circle of dancing ground, a hearth or spring at the centre, seating rising around
  it. Under the sharing rules it copies no real site, ceremony or sacred structure; the rhyme
  comes from forms many peoples share. The chaparral fits: the Greek theatre grew from the
  *orchestra*, the round dancing floor, with the chorus dancing and singing at its centre.
- **Amphitheatre look (user, 2026-11-18): echoes of Arches National Park and Greek
  landscapes.** From Arches: red and salmon sandstone, natural arches and fins, and the
  smooth slickrock bowl below Delicate Arch, a natural amphitheatre already. From Greece:
  a theatre whose tiers follow the hillside (Epidaurus), pale limestone, maquis scrub,
  cypress and olive against a hard blue sky. Together: a sandstone bowl worn into tiers,
  framed by an arch, with chaparral scrub on its rim. Landforms only: no petroglyphs or rock
  art (the park's Ute panels included), and no copy of a particular theatre.
- **Biome characters (art, 2026-11-18):** each biome leans into what its beings share:
  desert *the unseen*, chaparral *hybrids on the rocks*, grassland *under the open sky*,
  wetland *the drowning water*, broadleaf *faces in the grove*, conifer *the deep wood*,
  tropical *the mimic canopy*, mountain *giants and their gifts*. `docs/art/biome-characters.md`.

## The world map

**Decided (user, 2026-11-18):** the regions above lie on one procedural map. Research and
the generation proposal: `docs/research/biome-map-generation.md`.

- **Size:** about as big as Satisfactory's (roughly 5.4 × 5.4 km), **finite but
  complicated**: hills, canyons, lakes, not open plains.
- **Biome groups are interspersed**, and **the game starts in the desert**.
- **Danger rises toward the perimeter.** The dangers are not defined yet.
- **Hills and water:** the land has real elevation, including **mountain lakes**.
  Technology later in the game pumps water.
- **Lakes and ponds refill as the story moves along**: they start dry or low, and the arc
  fills them.
- **The edge of the world is a foggy wall** with club lights and vague figures dancing in it.
- **Ground:** the desert is **sand, salt and mud flats**; the chaparral is shrubs and rocky
  sand. The other biomes' ground is still to define.

Open: how many regions per group; what the dangers are; whether streams must run downhill
before pumps exist.

## The vardo: the player's base

**Decided (user, 2026-11-18):** the player's base, our equivalent of Satisfactory's HUB,
is a **barrel-top vardo drawn by the cyber-cob**, from Irish Traveller culture: a sturdy
piebald Irish cob with exactly one prosthetic leg (near-side front) and faint fibre-optic
strands in its mane and tail. **Not a donkey** (the art direction rules it out). The look,
continuity rules and reference images are in `docs/art/vardo-and-cyber-cob.md` and
`docs/art/vardo-cyber-cob/`; the models are `add-vardo-and-cyber-cob-assets`.

Draft:
- It holds what Satisfactory's HUB holds: the goals terminal and storage. The campfire
  stays the craft bench, beside it.
- Open: whether it moves. Satisfactory's HUB is fixed; a horse-drawn one can travel,
  which suits a journey from one mythic region to the next. Where it stands at the start
  (spawn) and what moving it costs.
- Treat the Traveller tradition with the same care as the myths: build from the
  references, not caricature.

## Omar: the player character

**Decided (user, 2026-11-18):** the protagonist is **Omar**, a well-travelled explorer:
a bright orange tagelmust over the face, wraparound neon-trimmed goggles, a knee-length
dark-mint double-breasted coat with frog fasteners, long midnight-navy gloves, sand
trousers and heavy boots. The look, continuity rules and references are in
`docs/art/omar.md` and `docs/art/omar/`. **For now the game needs only his gloves (the
first-person hands) and his shadow**: `add-omar-gloves-and-shadow`.

## Styling

**Decided (user, 2026-11-18):** players meet mythic and high-tech elements side by side,
so designs may use modern styling. The pillar "Satisfactory meets Inside" still opens the
game; the futuristic look is what blooming earns, arriving with the later stages.

To do: add this as a pillar in `docs/art/style-guide.md` (`add-art-direction`).

## Planned changes

Opened: `research-music-mythologies` (2026-11-18; regions and shrines need it). Proposed,
not yet opened: `add-narrative-arc`, `add-vardo-hub` (the base's gameplay; the art is
`add-vardo-and-cyber-cob-assets`),
`add-mythic-regions`, `add-deity-shrines`. Existing changes to amend: `add-art-direction`
(the styling pillar), `add-opening-region` (region one of the Mojave; the vardo at spawn),
`sync-major-minor-milestones` (major goals as arc stages), `add-campfire-craft-bench` (the
bench beside the vardo).
