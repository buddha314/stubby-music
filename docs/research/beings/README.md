# The being atlas

`beings.json` codes every mythic being found per biome, as an essence, by traits from a closed
vocabulary; `catalogue.md` holds each one's sources and sharing check (restricted beings are
listed there and never coded). `uv run tools/beings/cluster.py` validates the atlas, clusters
the beings by trait without using biome, and writes `report.md`, `dendrogram.png`, `map.png`
and `layout.json` (the amphitheatre's seating plan). Options: `--where music=core` keeps only
beings with that value, `--drop music,sound` leaves facets out; those runs go to `runs/<name>/`.
OpenSpec: `add-being-atlas`.

## Findings, 2026-11-18 (103 beings: 42 monsters, 41 spirits, 20 creatures; 30 with music at their core)

| Run | Best k | Agreement with biome (ARI at 8 clusters) | Nearest neighbour shares a biome |
| --- | --- | --- | --- |
| all traits | 2 | 0.02 | 34% (chance 20%) |
| without music and sound | 4 | 0.03 | 30% |
| without music, sound, place and element | 4 | 0.01 | 19% |
| music-core beings only (30) | 2 | 0.00 | 17% |

1. **Landscape shapes where a being lives, not what it is.** All the biome signal is in
   `place` and `element`: take them out and nearest neighbours share a biome no more often
   than chance. Form, role, disposition and theme recur in every landscape.
2. **With all traits, music decides the split**: 29 music beings against 74 others. It is
   the most decisive single trait, so analyses about kinds of being should drop it (and
   `sound`), as the user planned: restrict to music later.
3. **Without music, four kinds of being** (`runs/no-music_no-sound/`):

   | Kind | Size | Shared traits | Primary biomes |
   | --- | --- | --- | --- |
   | Givers | 21 | benevolent, seen | grassland 6, chaparral 4, mountain 4, tropical 3 |
   | Voices | 10 | unseen, heard only, night, ambivalent | desert 3, conifer 3, tropical 2 |
   | Devourers | 43 | dangerous, devouring, death | wetland 10, grassland 7, desert 6, tropical 6 |
   | Bargainers | 29 | ambivalent, sometimes seen | mountain 6, broadleaf 5, desert, chaparral, conifer, tropical 4 each |

   The mix differs by biome even though the kinds do not: **wetland is overwhelmingly
   devourers** (10 of 13: water horses, drowning imps, pond hags), **grassland is givers and
   devourers** with no voices or bargainers, the **forests and mountains lean to bargainers**,
   and **the desert has the most voices**, which suits the silent opening.
4. Some codes look wrong and need review: the devouring forest hag sits among the givers
   because Baba Yaga also helps the worthy.

Caveats: the coding is judgement from mostly secondary sources, and silhouettes are weak
(0.14–0.20): these are tendencies, not hard clusters.
