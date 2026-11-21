# Roadmap and direction

For agents and developers. This is the overall direction of the game and a first outline
of the road from here, written 2026-11-20 after folding in ideas from Burb Sweeper
(`docs/research/burb-sweeper-economy.md`) and a survey of other games
(`docs/research/game-mechanics-survey.md`). As in the design guide, each note is
**decided** (who, when) or **open**. This is direction, not requirements: every feature
still goes through OpenSpec, and rule types still need a Rule PR.

## The game in one paragraph

**Decided (user, 2026-11-20):** music and mythology restore the world, and a new society
is built on **art and science together**, in phases from **one human to a large
civilisation**. The pitch is **Satisfactory meets Farthest Frontier meets VCV Rack**:

| Part | What it brings | Where it lives |
| --- | --- | --- |
| Satisfactory | First-person discovery, resource nodes, recipes, milestones and phases, building by hand | resources, recipes, goals hub, build menu |
| Farthest Frontier | A settlement that grows: residents, needs, housing, jobs, seasons, town tiers | Dancers, settlement phases (new) |
| VCV Rack | Every machine is an instrument the player patches and plays; the factory is the patch | `bevy_stubby_rack`, `stubby_audio` |

"Satisfactory meets Inside" remains the *look* of the opening (haze, silhouettes, dread),
giving way to solarpunk colour as the world blooms. It is not the pitch.

## Pillars

Agents: check a proposal against these before writing it. Items marked *from Burb
Sweeper* are its doctrine, adopted here as direction (open until the user confirms them
individually); the rest are already decided in AGENTS.md or the design guide.

1. **Restoration, not extraction.** Resources come from listening, making and restoring,
   never from stripping the land. A mechanic that rewards taking from nature points the
   wrong way. (*from Burb Sweeper*, "recognition, not extraction".)
2. **Every machine is an instrument, and a rack plays only what it shows.** (decided)
3. **Progress is capability, not power.** A new phase gives new kinds of intervention, not
   bigger numbers. Goals are *states of the world* (a stream that runs clear for a day),
   not totals. (*from Burb Sweeper*.)
4. **One set of verbs, widening scope.** Listen and collect, lay streams, patch, tune,
   play: the same verbs at Camp and City; what grows is their reach, from one rack to a
   village's rhythm to a city's ensemble. (Spore's separate minigames are the warning.)
5. **No death, no ending; drift instead.** Goals guide and never end the game (decided).
   Shortfalls are recoverable: Dancers move away, growth eases back, haze returns.
   Nobody starves, nothing is raided, and nobody is displaced.
6. **The manual precursor (the burner rule).** Every automated system first has a
   hand-played, capacity-limited version, and that limit is the hint toward the next step.
   (*from Burb Sweeper*.) Example: carrying water before streams, playing a rack before
   sequencing it.
7. **Each phase creates the next phase's problem.** (*from Burb Sweeper*.) Clean water
   invites crops; crops invite Dancers; Dancers need trade; trade needs shared tempo.
8. **Art and science are one gate.** Unlocks need both a made thing (a recording, a
   performance) and an understood thing (a measurement, a discovered source).
   Burb Sweeper's "double gate" of cultures and specialists, recast.
9. **The world is the readout.** The mix, the water's colour, the fog and the Dancers'
   movement show the simulation's state; the HUD is reference, not the main display.
   Flow is legible and consequences are shown before the player commits. (*from Burb
   Sweeper*: "the mix is a readout".)
10. **Thin kernel, fat data.** Rules are rows (decided); new behaviour is a new rule type
    through OpenSpec. Shared spatial ideas are few: **fields** (a radius effect: foliage
    cleaning air, hearth reach, a stage's resonance) and **networks** (graphs: patch
    cables, streams, trade routes, rail). (*from Burb Sweeper*'s kernel.)

## Phases

**Decided (user, 2026-11-20):** Camp → Village → Town → City, after the silent opening.
The transition guidelines are the user's; the content columns are proposals.

| Phase | Becomes the next when (decided, thresholds open) | Player's new capability (proposed) | Population |
| --- | --- | --- | --- |
| Opening | the player hears and collects the first sound | listening, the recorder | Omar alone |
| Camp | there is enough clean water to plant a crop | patch, lay streams, filter water, water trees, light the hearth | Omar alone |
| Village | three villages trade goods with one another | crops, the first Dancers, standing jobs, the first being woken | a handful of Dancers |
| Town | the player can build an art gallery, a concert hall and at least one light rail | several villages with their own clocks, trade routes, research, saved patches | dozens |
| City | (open-ended) | civic works that display the player's music; light rail; reclaiming early industry | hundreds |

### Camp (now)

Mostly built: the opening, burblers, streams, the campfire, purifiers, high-pass sources,
ripple pools, oaks and watering, foliage cleaning the air, the HUD. To finish it:

- **Saving and loading** (laid streams, racks, inventory, world time). Nothing long can be
  playtested without it; it is the largest gap.
- **Goals for the water loop.** The goals stop at the campfire: nothing guides the player
  through high-pass filter, oscilloscope, crafting, the dam and clear water (see the audit).
- **The vardo as the HUB** (`add-vardo-hub`, not opened yet): the goals terminal and
  storage.
- **A Camp exit that can be measured**: "enough clean water to plant a crop" needs a crop
  rule and a clarity condition.

### Village (next)

Proposed, from the research (rankings in the survey):

- **The first crop.** A field watered by a stream; its output is limited by its binding
  input (clean water, tending), and the field shows which. Nutrients come from the
  low-pass channel once its source is decided (Farthest Frontier's soil fertility, mapped
  onto our filters).
- **Dancers arrive on surplus.** Provisioned on the draft schedule (water at dawn and dusk,
  food at midday, shelter at night). They come when there is spare water, food and
  shelter, and drift away when short; no deaths (Farthest Frontier, Anno).
- **The hearth.** The campfire is the centre of each settlement; its reach sets where
  Dancers live (Against the Storm's hearth). It already keeps the clock, so Dancers dance
  to it.
- **Jobs as counts, and standing jobs.** The player sets how many Dancers tend, carry and
  build (Banished); recurring care is attached to a thing and ends when the need ends
  (Burb Sweeper's standing jobs). Building and tending share one labour pool.
- **Waking the first being.** A rack with targets, as the campfire's; it takes its seat in
  the amphitheatre and gives a module back (decided). The amphitheatre is the skill tree.
- **The restored world sounds restored.** Each grown plant and returning creature adds a
  voice to the local mix (Proteus).

### Town

- **Each village keeps its own clock**; trade needs the two to agree on tempo (both whole
  multiples of the master clock already). Trade routes are set once and run themselves,
  ideally along streams so trade is audible (Anno's islands, Timberborn's districts).
  Villages specialise by biome, which is why they trade.
- **Research from recordings.** A listening post (Satisfactory's MAM) researches what the
  field recorder captured; recording a source a few times learns it (Subnautica's
  fragments); some entries are understanding, not items (Outer Wilds). Keep research
  currencies to about five.
- **Dancer tiers by needs.** Basic needs gate a household's advance; cultural needs (a
  nightly song, a festival) lift mood but never block (Anno 1800).
- **Resonance as desirability.** Homes near sounding streams, clean air and flowering
  trees want to upgrade; a droning, unpatched machine is a nuisance (Farthest Frontier).
- **Patch sheets.** Save a rack's cables and knobs and apply them to another machine of the
  same kind (Factorio's blueprints), against late micromanagement.
- **Silent seasons.** Forecast stretches when burblers slow or the wind returns; still
  pools carry the music through them (Timberborn's droughts, without the deaths).

### City

- **Gallery and concert hall show the player's own work**: saved patches, recorded bars,
  scanner captures; a programme mixing three villages earns more (Civilization's great
  works and theming). Visiting Dancers are the tourism.
- **Light rail** as the long-haul logistics tier, not only a goal flag.
- **Reclaiming early industry.** Once a region runs clear, Camp-era dams and rough
  streams can be dismantled for a refund and replaced by self-sustaining springs
  (Terra Nil), never breaking what the player still relies on.
- **A megaproject that keeps growing** after its goal is met (Dyson Sphere Program), since
  the game never ends.

### Throughout

- **Mythic regions** open phase by phase: one biome's beings per stretch of the map, the
  vardo travelling between them (open whether it moves).
- **Overlays on the scanner** for air, clarity and tempo fields (Oxygen Not Included).
- **Charters** (later): optional settlement customs that trade one style for another, never
  moral dilemmas (Frostpunk's laws, recast).

## Foundations to build before Village

1. **Save and load**, with world time resuming from the last processed checkpoint (the
   Dancer schedule already assumes it).
2. **A headless settlement resolver** in a Bevy-free core, in the style of
   `stubby_recipes`: needs, stock, housing, labour, at the calendar's checkpoints.
   **Port it from `automate-godot`** (`docs/research/automate-godot-port.md`): its tick
   dispatcher, rule engine (producer, converter, consumer, upkeep, with deficits) and
   workforce pool are about 200 lines of GDScript, the generalised form of Burb Sweeper's
   resolver, and already exactly the shape the Dancer schedule needs. Record intended
   demand, unmet demand and the binding input, as Burb Sweeper's ledger does.
3. **A balance harness** that runs the resolver for hundreds of days under scripted
   strategies and reports provisioned Dancer-days and the limiting input (Burb Sweeper's
   harness; `docs/design/long-session-previews.md` already sketches the first experiment).
4. **Measurable phase conditions** (settlement-phase task 2.4) as goal conditions, so the
   goals hub drives the phases.

## Rules audit (2026-11-20)

`assets/recipes/` holds 8 items, 7 resources, 2 mines, 1 recipe, 3 buildables, 2 machine
rows, 8 goals, 3 air rows and the start inventory. All of it serves Camp. Findings, each
marked as a **Data** fix (rows only) or a **Rule** change (OpenSpec first):

| # | Finding | Kind |
| --- | --- | --- |
| 1 | **Goals cover only Camp.** None guide the water loop: collecting a high-pass filter and an oscilloscope, crafting the purifier kit at a burning campfire, building it across a stream, tuning it, clear water. | Data, plus Rule for a clarity condition |
| 2 | **No goal condition for water clarity** ("the stream below a dam reaches clear", "clear water for a day"). Needed by goal 1 and by the Camp → Village transition. | Rule |
| 3 | **"Light the fire" needs a snail shell** (the campfire's sequencer place) but no goal or hint points the player at one. | Data (a goal or a hint) |
| 4 | **`clock` item is orphaned.** The `clock_source` resource yields it, but the burbler is configured, never collected; only a test adds it. Retire it (with a migration row) or define a portable clock. | Data, user decision |
| 5 | **Two sources of truth for bugs.** `scrub_oak` produces `lightning_bug` with `productivity_factors`, validated but never run; the bugs actually come from the seeded layout's `lightning_bugs` rule. Fine for now; the production loop (`add-scrub-oak-production`) must replace one with the other. | note |
| 6 | **Oak growth skips cloudy.** Stages need clarity 0 (budding), 0.33 silty (leafed), 1.0 clear (bloom); cloudy (0.67) unlocks nothing. Possibly intended; worth a stage or a use. | Data, designer |
| 7 | **One recipe.** The purifier kit's 20 reeds is a draft awaiting the user. | Data, user decision |
| 8 | **Items carry no tier or phase.** Unlock gating by phase, alternates and research will need it. | Rule |
| 9 | **Missing rule types for the roadmap:** settlement phase and its conditions; crop or field (water demand, yield, binding input); food; Dancer needs schedule; housing capacity; labour and job counts; trade route; research and unlock; fields such as hearth reach and resonance. | Rule, one OpenSpec change each, in Village order |
| 10 | **Settlement draft conditions are not measurable yet** ("enough clean water", "trade", "can build") (`add-settlement-phase-reviews` 2.4). | Rule, user decision |

Elsewhere: many OpenSpec changes are partly done (`add-desert-opening` 12/40,
`add-rack-and-spatial-audio` 8/33, `add-inventory-and-goals-hud` 14/25). A triage pass that
archives finished ones and closes stale tasks would make `openspec list` a usable map
again.

## From Burb Sweeper: what to carry, what to leave

Carry: the needs accounting (intended and unmet demand per checkpoint), housing as
capacity, one labour pool, the binding input on production, upkeep (neglected plantings
ease back, as our oaks already do), the balance harness and its person-days measure,
standing jobs, workers whose speed is fixed so more work means more hands, component roles
on networks (source, conduit, tap, store), hidden natural features found by clues, diegetic
charts, the status overlay's three layers, and its governance docs (a systems map of
shipped/building/designed/idea, an open-questions register).

Leave: the climate clock and doomsday countdown, the race against time, death and air
damage, escalating disaster campaigns, and any loss condition. Burb Sweeper is a survival
game; Stubby Music is an open-ended simulator.

## Open decisions for the user

- Which Burb Sweeper doctrine items to adopt as pillars (marked above).
- The first crop, its clarity threshold and how "enough water" is measured.
- What a village is, which goods count as trade, and what a qualifying exchange is.
- Whether Dancers are simulated as households or individuals, and whether lineages
  prefer different tempos.
- Whether research uses recordings as currency, and how many kinds.
- Whether the vardo moves between regions.
- The low-pass and band-pass filter sources (still undecided).
