## Why

The player needs a place to craft, Satisfactory's Craft Bench. In this world it is a
**campfire**: a bundle of sticks, leaned into a teepee over a bed of ash (user,
2026-11-17; it replaces the first sketch, a stone ring with a lean-to). Lighting the fire is the
last step of the first major goal, "Build a camp" (decided 2026-11-13 in the sibling
Godot project, `wait-for-stubs/docs/design/goals.md`). The goal reads: the wind takes
everything, so grow a windbreak (water a scrub oak until it buds), then light a fire
in its shelter.

The fire is lit with music, not a key press. The player patches **a voice and a
sequencer**: a spark (a square wave from a lightning bug) struck on beats 2 and 4, like
a hi-hat on the backbeat, from a snail shell's four steps. This was the first planned use
of the snail-shell sequencer (`sync-snail-shell-sequencer`). Every machine is an
instrument, and the camp is the first one whose music *makes* something.

**Rhythm management** (user, 2026-11-17): every recipe has a BPM range, and the bench's
tempo is its clock's, the nearest running burbler or the stream it feeds. You can only
build certain things at a given BPM, so the player sets their streams to the tempo of
what they want to make. Tempos are on a **24 BPM grid** for now (24, 48, ... 120, 144,
... 240), so matching is easy and the popular 120 and 144 are on it.

## What Changes

- **The camp is a buildable.** Built with Q from 25 reeds, as a stream segment is
  (`buildables.json`). It is placed unlit: a bundle of sticks over ash, kindling at its
  foot. Once shelter exists, placement is valid only in a budding oak's lee
  (`sync-major-minor-milestones` "Stop the wind", `add-weather-and-dust-storm`).
- **It has a rack, like every machine.** E at the unlit camp opens its rack (the shared
  `bevy_stubby_rack`), with a place for a voice (a `square_wave` mounts as the spark's
  Square VCO) and a place for a sequencer (a `snail_shell`). Its clock is a running burbler or
  flowing stream within a tuned reach (5 m to start), and its output is normalled to the fire. Nothing is hidden. The rack plays
  only what it shows.
- **Lighting it.** The fire catches when a full bar strikes on beats 2 and 4: the
  sequencer's gate fires the spark voice on those steps, timed by the audio clock. Each
  strike throws sparks from the bundle at its heard instant. A wrong pattern or a broken
  patch only sparks. A new goal condition, `fire_lit`, latches (a rule PR in
  `stubby_recipes`).
- **Lit, it is the craft bench.** E at a lit camp opens the crafting screen, laid out as
  Satisfactory's Craft Bench: a framed window titled with the camp's name; on the left a
  recipe list grouped by category with an "only what I can make" filter; on the right
  the selected recipe's inputs as `have / need` tiles and its output; a craft button
  (hold to repeat); pin to the To-Do List. It crafts through `CraftRequest`. The
  recipes are designer rows in `recipes.json`.
- **Recipes have a BPM range** (rule type, built): `recipes.json` rows may carry
  `bpm: {min, max}`, multiples of 24; `Catalog::craft_at` makes a recipe only when the
  bench's clock is in range and takes nothing otherwise (`RecipeError::OffTempo`).
  The screen shows each recipe's range and the clock's current BPM. The burbler's BPM
  knob clicks through the 24 grid, and new burblers seed on it (48 to 192).
- **The fire burns only while its clock runs**: when the beat stops it dies to embers
  and must be lit again.
- **The fire is a sound and a light.** It crackles from the audio engine (an acoustic
  profile, `add-rack-and-spatial-audio`). It does not change air or light quality.
- **Art (Claude, built):** `tools/blender/create_campfire.py`: fourteen sticks leaned into
  a teepee over a bed of ash, kindling at the foot; `Fire` and `Embers` ship hidden
  (start_scale 0); `Light_fire`, `Sparks` and `Interact` nodes; near and far levels; a
  Jackdaw prefab, a collider and the rack's `configure` block as data in its profile.
  Previews: unlit, burning, embers.

## Capabilities

### New Capabilities
- `campfire-craft-bench`: the camp buildable, lighting it by a sequenced spark, and
  crafting at the lit fire.

### Modified Capabilities
None in accepted specs. It extends in-flight changes: `add-stream-building` (another
buildable), `add-rack-configurations` (another rack), `add-inventory-and-goals-hud`
(`fire_lit`, pins from the bench), `sync-major-minor-milestones` (step 6 of "Build a
camp"), and `sync-snail-shell-sequencer` (its first use).

## Impact

- Rules: `buildables.json` (camp row), `goals.json` (the step), the `fire_lit` condition
  kind (schema and validator), first `recipes.json` rows.
- Crates: the camp's binder and condition report in `stubby_game` glue; the crafting
  screen as presentation in `stubby_game` (or `stubby_ui` widgets), over
  `bevy_stubby_recipes`; the rack profile is data (`configure` block); no new system
  crate.
- Assets: `assets/structures/campfire/` (glb, profile with `configure`, prefab),
  `assets-src/blender/structures/campfire.blend`, `tools/blender/create_campfire.py`,
  an icon. Every outside asset is CC0 only.
