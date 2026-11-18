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
- **Sources.** Many of these are living, sacred traditions, and some knowledge is
  restricted (to initiates, to men or to women); petroglyphs are sacred sites to the
  descendants of their makers. The choice to make: draw only on stories their
  communities share openly, with each source recorded, or design original deities from
  the landscape and the kinds of myth there, not from specific sacred beings. The research
  (`research-music-mythologies`) records how freely each story is shared either way.
- How waking a deity plays (a rack with targets, as the campfire's, is the obvious fit) and
  what it gives back (light quality around it, a region's power).
- The echo grotto's boulders as the first carving site.

## The vardo: the player's base

**Decided (user, 2026-11-18):** the player's base, our equivalent of Satisfactory's HUB,
is a **barrel-top vardo drawn by a donkey**, from Irish Traveller culture. The user will
provide reference images (to be kept beside this guide).

Draft:
- It holds what Satisfactory's HUB holds: the goals terminal and storage. The campfire
  stays the craft bench, beside it.
- Open: whether it moves. Satisfactory's HUB is fixed; a donkey-drawn one can travel,
  which suits a journey from one mythic region to the next. Where it stands at the start
  (spawn) and what moving it costs.
- Treat the Traveller tradition with the same care as the myths: build from the
  references, not caricature.

## Styling

**Decided (user, 2026-11-18):** players meet mythic and high-tech elements side by side,
so designs may use modern styling. The pillar "Satisfactory meets Inside" still opens the
game; the futuristic look is what blooming earns, arriving with the later stages.

To do: add this as a pillar in `docs/art/style-guide.md` (`add-art-direction`).

## Planned changes

Proposed 2026-11-18, not yet opened: `research-music-mythologies` (first; regions and
shrines need it), `add-narrative-arc`, `add-vardo-hub` (waits for the reference images),
`add-mythic-regions`, `add-deity-shrines`. Existing changes to amend: `add-art-direction`
(the styling pillar), `add-opening-region` (region one of the Mojave; the vardo at spawn),
`sync-major-minor-milestones` (major goals as arc stages), `add-campfire-craft-bench` (the
bench beside the vardo).
