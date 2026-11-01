## Framing
An open-ended simulator in the Satisfactory mould. The opening region is the world's
starting state plus the first loop the player can close. Nothing ends: the loop keeps
running, the region keeps changing, and the world goes on past the region's edge.

## Starting conditions
- **Silence.** The silent windstorm (`add-desert-opening`). The first sound is a
  lightning bug's click, heard only within its hearing range.
- **Winter.** Every scrub oak starts dormant (`add-scrub-oak-production`). Reeds stand
  dry.
- **Bleak.** Light quality is 0 everywhere; props show their bleak look.
- **Time.** The shared calendar starts at daylight (`add-resource-replenishment`).
  Bugs flash in the dark, so the first night matters.

## What the region contains
| Element | Role in the sim | Count (first pass) |
| ------- | --------------- | ------------------ |
| Spawn | where the player stands | 1 |
| Lightning bugs | exhaustible square-wave source; returns next night | 3–6, loose near the grove |
| Scrub-oak grove | infinite bug mines, dormant until watered | 3–5 oaks, one grove |
| Mudpit burbler | the clock mine; the stream's source | 1 |
| Reed beds | frequent renewable; 6 reeds per stream segment | many clumps on wet ground near the burbler and in damp hollows |
| Desert props | silhouettes in the haze; bloom with light quality | sparse scatter |

## Placement constraints (every seed)
- Spawn lies outside every bug's hearing range (35 m), so the game starts silent.
- The grove lies 40–120 m from spawn, the bugs within its reach (the existing square
  placement band).
- The burbler sits 20–40 m from the nearest oak, so a first stream of 5–10 segments
  connects them.
- Reed clumps within 40 m of the burbler hold at least enough reeds for that stream
  (6 per segment), with room to spare.
- All of it is reachable on foot over flat mudflat.

## Layout: prefabs in Jackdaw (decided)
Levels are designed by placing **prefabs** in the Jackdaw opening scene. Every
placeable world asset has one under `jackdaw/assets/prefabs/` (oaks, reed clumps,
props, the burbler, the lightning bug, world modules, stream segments), made and
edited in Jackdaw and checked by `jackdaw/tests/prefabs.rs`. The scene file
is the region's layout, so it is deterministic by construction. Seeded scatter (props,
extra reeds) may fill in around the authored layout later; the placement constraints
below hold for the authored layout and for any scatter.

The game must turn placed prefabs into live things: a placed bug flashes and clicks on
its own clock, a placed oak is a mine in the ledger, a placed module joins the engine.
Today bugs and modules are spawned in code at fixed positions, so binding behaviour to
placed entities is the first game task (2.5).

## The loop
1. Hear a bug, follow it in, collect it: square-wave items.
2. Configure the burbler: a square-wave VCO in its rack gives the burbler a voice at
   its BPM.
3. Gather reeds and lay a stream from the burbler. The stream carries the music as
   sheet music on muddy water.
4. Water reaches the oaks. **Clean** water wakes them (muddy does not). They bud, leaf
   out and bloom.
5. Open blooms produce bugs, so square waves become renewable. Growth near the music
   raises light quality, and the region blooms outward: props bloom and the storm eases.
6. The player keeps extending: more streams, more oaks, new sources further out.

## Open questions
- **The first clean water comes from a filter mini-rack** (decided in direction, designed
  later; `add-fluid-quality-and-viscosity`): high-pass filters separate pure liquid,
  low-pass filters separate solids and nutrients, into channels, controlled via audio.
  Until it exists the loop stops at muddy water, so the region's first filter rack, and
  where it sits between burbler and grove, is part of this change once designed.
- Counts and distances above are a first pass, to tune in play.
- Reconcile `add-desert-opening`'s discovery spec ("Square VCO", then "Clock"): the
  square now comes from a bug and the clock is the burbler.
