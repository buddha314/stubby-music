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

Draft stages, to be settled in `add-narrative-arc`:

| Stage | What the player sees | Ties to |
| --- | --- | --- |
| Mudflat | silent windstorm, bleak haze, no one around | the opening (`add-desert-opening`) |
| Camp | the vardo, a campfire, the first burblers and streams | `add-campfire-craft-bench` |
| Village | the first deities woken; the air clears | lighting stage `clearing` |
| Town | regions bloom; people return | lighting stage `bloomed` |
| City | the solarpunk city | not designed yet |

Open: what triggers each stage (light quality, deities woken, goals met), and how the
stages map onto the long-term goals (`sync-major-minor-milestones`).

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
- **Regions: decided (user, 2026-11-18):** similar biomes are grouped, because the
  creatures are a product of their environment, with small regional differences:
  **desert** (US Southwest and Mojave, Aboriginal Australia), **Mediterranean**,
  **temperate forest** (Ireland and England, the Pacific Northwest) and **tropical**
  (Africa, perhaps Southeast Asia). Everyone in this world lives together well and
  intermingles, so a region's figures may come from any tradition in its group. The game
  needs **about a dozen characters** in all.
- **Figures: decided (user, 2026-11-18):** unnamed, as spirits of a place, with imagery
  similar to, not identical with, their cultural heritage. Imagery gets the same check as
  stories: only openly shared motifs and styles, never a copy of a particular work.
- How waking a deity plays (a rack with targets, as the campfire's, is the obvious fit) and
  what it gives back (light quality around it, a region's power).
- The echo grotto's boulders as the first carving site.

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
- **Ground:** the desert is **sand, salt and mud flats**; the Mediterranean is **chaparral**,
  shrubs and rocky sand. Temperate forest and tropical are still to define.

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
