## Why

The user (2026-11-18): once the creatures are sorted, monsters included, the world will have a
**natural amphitheatre** somewhere that shows how the beings cluster. That needs each being
described by the same metadata, and a cluster analysis that groups beings by their traits
and compares the groups with the biomes. The essences in `research-music-mythologies` are
prose; this change makes them data.

## What Changes

- **The being atlas**, `docs/research/beings/beings.json`: one row per essence (spirit,
  creature or monster) with its biomes, status and traits from a controlled vocabulary
  (form, sound, role, transformation, place, time, presence, disposition, gift, theme). Rows
  mirror the essence table in `docs/research/music-mythologies.md`, which keeps the evidence
  and sharing checks.
- **The analysis**, `tools/beings/cluster.py` (run with `uv run`; pinned dependencies):
  validates the atlas, clusters beings by trait distance without using biome, and reports
  how the clusters line up with biomes, each biome's signature traits and each being's
  nearest neighbour. It writes `report.md`, `dendrogram.png`, `map.png` and `layout.json`
  (a seeded 2D layout keeping similar beings close: the amphitheatre's seating plan).
- **Monsters** join the atlas: research them by biome under the same sharing rules.
- **No game code.** The amphitheatre itself is a later change that reads `layout.json`.

## Capabilities

### New Capabilities
- `being-atlas`: the beings' metadata, its vocabulary, and the cluster analysis over it.

### Modified Capabilities
None.

## Impact

- New: `docs/research/beings/`, `tools/beings/cluster.py`.
- Follows `research-music-mythologies` (essences, sharing checks) and feeds a future
  amphitheatre change and `add-mythic-regions`.
