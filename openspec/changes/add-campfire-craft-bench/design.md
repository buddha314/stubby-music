## Context

Satisfactory's first crafting station is the Craft Bench: a building you walk up to,
press E, pick a recipe from a list and hold to craft. Our world asks that every
machine be an instrument, and that music be how the world gets better. The camp joins
the two. It is the bench, but it stays dark and useless until the player plays the fire
alight.

Sources: the user's note (2026-11-17: "our crafting bench will be a small scene of a
campfire with a lean-to style open tent; this is crafted using a voice and a sequencer
to light the fire"); `wait-for-stubs/docs/design/goals.md` ("Build a camp", the flint on
2 and 4); `wait-for-stubs/docs/design/wall-of-bad-ideas.md` ("Ring of stones");
Satisfactory's Craft Bench layout (`docs/research/satisfactory-goals-and-inventory.md`,
"Layout").

## Goals / Non-Goals

**Goals:** a camp the player builds, lights by patching and playing a backbeat, then
crafts at; the first use of the snail-shell sequencer; the last step of "Build a camp".

**Non-goals:** wind and shelter simulation (owned by `add-weather-and-dust-storm` /
"Stop the wind"; the camp only asks whether a spot is sheltered); what the camp gives
beyond crafting (respawn, rest at night: later); the recipes themselves (designer data).

## Decisions

### D1. One asset, two roles
The camp is one placed buildable with a state: `unlit` → `lit` → back to `unlit`. Unlit,
E opens its rack. Lit, E opens the crafting screen, and the rack stays reachable from a
tab or second key in that screen so the music can still be changed. **The fire burns
only while its clock runs** (user, 2026-11-17): when the clock stops, it dies down to
embers and must be lit again. The `fire_lit` goal condition still latches.

### D2. The rack is data, like the burbler's
The camp's profile has a `configure` block (`add-rack-configurations`):
- a `voice` place accepting `square_wave` (becomes `square_vco`, the spark's voice);
- a `seq` place accepting `snail_shell` (becomes `snail_sequencer`);
- a `volume` module, normalled to the fire;
- no clock module of its own: its clock is a running burbler or a flowing stream within
  `clock_reach_m` of the camp (D7);
- two or three comb cables suggesting clock → sequencer CLOCK, sequencer GATE → volume
  TRIG, VCO OUT → volume IN.

No new module types. Steps are lit on the shell's keys, as on the burbler.

### D3. Lighting is judged on the heard clock
The game owns the rule (glue, not the rack or the audio crate). It reads `StepTick`
events (heard instant, step index, gate) for the camp's sequencer and the rack's
completeness. The fire catches after **one full bar** (steps 1–4 in order) in which the
gate fired on steps 2 and 4 and not on 1 and 3, with the voice patched to the fire.
That bar is the backbeat. The rule's pattern is data in the camp's profile
(its `configure.targets`: `seq.gate_1..4` at 0, 1, 0, 1; it was `light_pattern` until
`add-reed-water-purifier` 1.3), so later fires can ask for other rhythms.
Each strike throws sparks from the bundle (`Sparks` node) at its heard instant, so the
player sees the rhythm. On lighting, the game reports `GoalEvent::ConditionMet(FireLit { buildable:
"campfire" })`.

### D4. The crafting screen follows Satisfactory's Craft Bench
It is a framed window with a title bar (the shared screen mode, Escape closes it),
like the rack screen's layout:
- left, a recipe list grouped by category, with a toggle for only the recipes the player
  can make now;
- right, the selected recipe: input tiles with `have / need` (amber when enough), an
  arrow, and the output tile with its count;
- under it, a craft button: click crafts one, holding repeats, and the count shows
  `x N`; a recipe made by hand (`by_hand`) fills a progress bar instead, and makes one
  when it is full (`add-crafting-durations`, 2026-11-27);
- a pin button that adds the recipe to the To-Do List (`GoalEvent::Pin`).

Crafting is instant for now. Satisfactory's bench takes time while the button is held.
A per-recipe craft time is a later rule type. The screen sends `CraftRequest` only, so
it holds no crafting rules.

### D5. Art
Claude builds it in Blender (`tools/blender/create_campfire.py`, headless, stylised PBR):
- **A bundle of sticks** (user, 2026-11-17, replacing the stone ring and lean-to):
  fourteen dry sticks leaned into a teepee about 1 m high over a 0.55 m bed of ash,
  kindling at the foot. `Fire` (flame sheets, emissive amber) and `Embers` (glowing
  coals) ship hidden at start_scale 0; `Light_fire`, `Sparks` and `Interact` nodes.
- **States:** unlit, a spark burst (particles, from the game), and burning. The flame
  mesh is emissive and its point light flickers on the beat.

The scale stays readable in Satisfactory's industrial way, in Inside's muted palette:
the fire is the first warm light in the bleak desert. LOD0–2 nodes go in the one glb.

### D7. The clock comes from nearby water (user, 2026-11-17)
The camp has no clock of its own. It takes the beat of the nearest **running burbler or
flowing stream** within `clock_reach_m` of it, 5 m to start. The reach is profile data,
to be tuned and never hard-coded. Through a stream it is the beat of the burbler that
feeds it. That beat drives the camp's sequencer CLOCK. No beat in reach means no strikes
and no fire, and a burning fire dies when its beat stops (D1).

### D8. Costs and air (user, 2026-11-17)
The camp costs **25 reeds** (`buildables.json`). A lit fire does **not** change air or
light quality: only foliage cleans the air.

### D9. Recipes are made at a tempo (user, 2026-11-17)
Every recipe may carry a BPM range; the bench's tempo is its clock's (D7). You can only
build certain things at a given BPM, so the player sets their streams correctly: rhythm
management is part of crafting. The rule is data (`recipes.json` `bpm: {min, max}`,
`stubby_recipes::BpmRange`), checked by `Catalog::craft_at` with the clock's tempo, all or
nothing (`RecipeError::OffTempo`). `CraftRequest` carries the bench's `bpm`. Tempos sit on
a **24 BPM grid** for now (`stubby_recipes::BPM_STEP`): recipe ranges are validated as
multiples of 24, the burbler clock's BPM knob has `step: 24` in its module data (the
engine still runs any whole BPM, so this stays a game rule), and new burblers seed on it
(48 to 192).

## Risks / Trade-offs

- [Players don't know what a backbeat is] → the comb's ghost cables and the step keys show
  which steps strike; no text (AGENTS.md: not a text adventure). If players still stall,
  the camp could accept any rhythm first and the backbeat later.
- [Shelter not built yet] → until "Stop the wind" lands, placement ignores shelter
  (flagged in the hologram as always valid); the requirement still stands.
- [Two screens from one E] → D1's tab keeps one entry point; revisit after play.

## Open Questions

Answered by the user on 2026-11-17: the clock source (D7), the reed cost (D8), whether
the fire keeps burning (D1) and the air (D8). Still open:

1. **First recipes** at the bench (patch cables from bird nests is the one already named).
2. Whether the square wave and shell stay mounted in the rack (and can be taken back),
   as on the burbler. Draft: yes.
