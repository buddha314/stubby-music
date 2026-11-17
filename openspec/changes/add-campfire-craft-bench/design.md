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
The camp is one placed buildable with a state: `unlit` → `lit`. Unlit, E opens its rack.
Lit, E opens the crafting screen, and the rack stays reachable from a tab or second key
in that screen so the music can still be changed. A lit fire stays lit (latched, as goals
are) unless open question 3 decides otherwise.

### D2. The rack is data, like the burbler's
The camp's profile has a `configure` block (`add-rack-configurations`):
- a `voice` place accepting `square_wave` (becomes `square_vco`, the spark's voice);
- a `seq` place accepting `snail_shell` (becomes `snail_sequencer`);
- a `volume` module, normalled to the fire (the "outlet" is the pit);
- a clock (question 1);
- two or three comb cables suggesting clock → sequencer CLOCK, sequencer GATE → volume
  TRIG, VCO OUT → volume IN.

No new module types. Steps are lit on the shell's keys, as on the burbler.

### D3. Lighting is judged on the heard clock
The game owns the rule (glue, not the rack or the audio crate). It reads `StepTick`
events (heard instant, step index, gate) for the camp's sequencer and the rack's
completeness. The fire catches after **one full bar** (steps 1–4 in order) in which the
gate fired on steps 2 and 4 and not on 1 and 3, with the voice patched to the fire.
That bar is the backbeat. The rule's pattern is data in the camp's profile
(`light_pattern: [false, true, false, true]`), so later fires can ask for other rhythms.
Each strike shows sparks at the step's stone at its heard instant, so the player sees
the rhythm. On lighting, the game reports `GoalEvent::ConditionMet(FireLit { buildable:
"campfire" })`.

### D4. The crafting screen follows Satisfactory's Craft Bench
It is a framed window with a title bar (the shared screen mode, Escape closes it),
like the rack screen's layout:
- left, a recipe list grouped by category, with a toggle for only the recipes the player
  can make now;
- right, the selected recipe: input tiles with `have / need` (amber when enough), an
  arrow, and the output tile with its count;
- under it, a craft button: click crafts one, holding repeats, and the count shows
  `x N`;
- a pin button that adds the recipe to the To-Do List (`GoalEvent::Pin`).

Crafting is instant for now. Satisfactory's bench takes time while the button is held.
A per-recipe craft time is a later rule type. The screen sends `CraftRequest` only, so
it holds no crafting rules.

### D5. Art
Claude builds it in Blender (`tools/blender/create_campfire.py`, headless, stylised PBR):
- **Pit:** four stones in a ring (the steps), with a small amber glow socket on each
  for the playhead. Ash and reed kindling sit inside.
- **Lean-to:** two forked poles and a ridge pole, with reed thatch sloping to the
  ground behind the fire and the open side facing it. It is about 2 m wide and 1.6 m
  high at the open edge.
- **States:** unlit, a spark burst (particles, from the game), and burning. The flame
  mesh is emissive and its point light flickers on the beat.

The scale stays readable in Satisfactory's industrial way, in Inside's muted palette:
the fire is the first warm light in the bleak desert. LOD0–2 nodes go in the one glb.

## Risks / Trade-offs

- [Players don't know what a backbeat is] → the comb's ghost cables and the stones show
  which steps strike; no text (AGENTS.md: not a text adventure). If players still stall,
  the camp could accept any rhythm first and the backbeat later.
- [Shelter not built yet] → until "Stop the wind" lands, placement ignores shelter
  (flagged in the hologram as always valid); the requirement still stands.
- [Two screens from one E] → D1's tab keeps one entry point; revisit after play.

## Open Questions

1. **Clock source.** Options: (a) a built-in clock in the camp's rack; (b) a `clock`
   item mounted in a place (the item exists); (c) the beat carried by a stream from a
   burbler, so the camp must be watered with music (streams as patch cables). Draft:
   (c) is the most in theme but needs stream-to-machine inputs; (b) works first.
2. **Reed cost** of the camp, and whether the lighting consumes the square wave and
   shell (mounted, so they stay in the rack and can be taken back) or not.
3. **Does the fire need the music to keep burning?** Latched (craft any time) or
   burning only while the rack plays (stop the music and it dies down to embers).
4. **Light quality:** does a lit fire clean or warm the air around the camp, the way
   foliage does (`clean-air-by-foliage`)?
5. **First recipes** at the bench (patch cables from bird nests is the one already named).
