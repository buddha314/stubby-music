## Context

`research-music-mythologies` turned sixteen figures into **essences** (design D6): kinds of
being, not gods, chosen as we go. The user wants the beings, monsters included, clustered,
and the clusters shown in the world by a natural amphitheatre.

## Decisions

### D1. One row per essence, traits from a closed vocabulary
Each row: `id` (snake_case), `essence`, `kind` (spirit, creature, monster), `biomes` (the
first is primary), `status` (candidate, chosen, set_aside, consult), `found_in`, and ten
multi-valued facets. The vocabulary lives in the file; a value outside it fails validation,
so growing the vocabulary is a deliberate edit. Coding is judgement: the first pass is
Claude's and is reviewed as essences are chosen.

### D2. Biome is never a clustering input
Traits alone group the beings, and the report measures how well those groups match biomes
(adjusted Rand index). That answers the question the amphitheatre poses: do the landscapes
make the beings, or do other kinships cut across them? `place` is partly a proxy for biome;
the report shows both, and a later run may leave `place` out to test it.

### D3. Distance and method: established, small, explainable
Mean over facets of the Jaccard distance between trait sets (a facet empty on both sides
is skipped), so a facet with many values does not outweigh one with few. Average-linkage
hierarchical clustering (scipy), k by silhouette (scikit-learn), metric MDS with a fixed
seed for the 2D layout. No hand-written clustering. The script declares pinned
dependencies inline and runs with `uv run`; nothing is added to the game's build.

### D4. The amphitheatre reads the layout, later
`layout.json` holds each being's position in [-1, 1]², its cluster and primary biome. The
amphitheatre change decides how that becomes seats, carvings or terraces. The layout is
deterministic: same atlas and pins, same layout.

## Risks / Trade-offs

- Seventeen beings is too few for stable clusters: the first run finds k = 2 with a weak
  silhouette (0.23). Results are a conversation piece until the atlas, monsters included,
  is larger.
- Trait coding carries the coder's reading of each tradition. Keep codes to what the
  research note's entries say.

## Open Questions

1. What the monsters are, per biome, and whether "monster" means dangerous to the player
   or only ambivalent.
2. Where the amphitheatre stands, and whether it shows clusters, biomes, or both.
