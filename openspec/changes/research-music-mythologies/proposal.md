## Why

The game's arc (`NARRATIVE.md`) scatters music deities and creatures across a procedural
map, each in a region like its mythic origin, starting in the US Southwest and the Mojave,
perhaps blended with Australian Aboriginal myth. Regions (`add-mythic-regions`) and the
carvings the player wakes with song (`add-deity-shrines`) both need to know who these
figures are, where their stories come from, and whether the game may use them. Many are
living, sacred traditions, and some knowledge is restricted. The user decided
(2026-11-18) that the game draws **only on stories their communities share openly**, with
each source recorded.

## What Changes

- Add `docs/research/music-mythologies.md`: for each region, the music deities, spirits
  and creatures; what each is tied to (landscape, instrument, sound, season); the source;
  and who shares the story and how freely.
- A sharing check on every entry: openly shared by its community, restricted, or unclear.
  Only the first may be used in the game.
- Regions are biome groups (user, 2026-11-18): desert (US Southwest and Mojave,
  Aboriginal Australia), Mediterranean, temperate forest (Ireland and England, the
  Pacific Northwest) and tropical (Africa, perhaps Southeast Asia). About a dozen
  characters in all. The desert comes first.
- **No code, assets or rule changes.** Figures stay research until a region or shrine
  change adopts them.

## Capabilities

### New Capabilities
- `mythology-sources`: how mythic figures are researched, checked for sharing, recorded
  and adopted.

### Modified Capabilities
None.

## Impact

- Docs: `docs/research/music-mythologies.md`; `docs/design/design-guide.md` records the
  decision.
- Gates `add-mythic-regions` and `add-deity-shrines`: neither uses a figure that this note
  has not cleared.
