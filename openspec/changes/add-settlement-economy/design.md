## Context

The settlement phases (Camp → Village → Town → City) and the Dancers' draft schedule are in
`add-settlement-phase-reviews`. Precedents: Burb Sweeper's needs accounting and
`automate-godot`'s pure rule engine. Farthest Frontier (FF) is the model for population,
housing, professions and town size.

## Decisions

### D1. One Bevy-free core, in this repository (user, 2026-11-20)

`crates/stubby_economy`, a core like `stubby_recipes`: no Bevy, no assets, no rule files.
It takes plain data (rules, roster, stock) and returns plain results. It moves to its own
repository only if it grows big enough to share with Burb Sweeper. Its API names no game
item; the game's rows give it meaning.

Ported from `automate-godot` (`docs/research/automate-godot-port.md`):

- **Dispatcher**: steps the calendar's checkpoints, one at a time and in order, so a jump of
  N days resolves exactly as N single days; resumes from the last processed checkpoint.
- **Rule engine**: producer, converter, consumer and upkeep rules with per-unit inputs and
  outputs; producers and converters before consumers and upkeep; whole conversions only;
  shortfalls clamp to zero and are reported as deficits.
- **Labour ledger**: named, atomic, idempotent reservations of Dancers, with a snapshot for
  saving.

The four port checks cover apply order, whole conversions, deficits and jump versus
stepped time. A ledger check covers atomic resizing and snapshot restore.

The dispatcher takes a schedule in whole simulation seconds from the host.
Dawn is at zero within the cycle. Midday, dusk and night are distinct offsets.
The first test uses solar midnight for the night check, after the dusk ration.
The chosen night offset remains a provisioning rule for the host. Save the
cursor alongside stock and roster state. If resolving a checkpoint fails, that
checkpoint stays pending. The caller commits its changes only on success.

The labour ledger tracks named reservations by count at this layer. Roster
and crew logic will select the individual Dancers. Existing reservations survive
a capacity drop. Releasing or reducing a reservation remains possible during an
overcommit, and starting or expanding a reservation requires free capacity.

### D2. Dancers are individuals (user, 2026-11-20)

Each Dancer has: a stable ID; a body shape and a skin tone, drawn from a seed and kept
across reloads (`add-settlement-phase-reviews`: four of each to start); a home dwelling; a
profession; a current task; and a local clock ID. The roster is the source of truth;
counts (population, laborers, idle) are derived from it, never stored beside it.

### D3. Housing: cottages of at most four, filled at random (user, 2026-11-20)

A dwelling is a rule row with a capacity. The **cottage** holds at most **4**. A new Dancer
is housed in a dwelling with a free place chosen at random from a seeded stream (the world
seed and the Dancer's ID), so the same game houses the same Dancers in the same places.
Larger dwellings are rows added later; FF's chain (shelter, homestead, large house,
manor) is the model for upgrading.

### D4. Town size from FF's population metrics, to begin with (user, 2026-11-20)

FF sizes a town by its Town Center tier, each needing a population, a number of houses of a
type and certain buildings ([Farthest Frontier Wiki, Town Center](https://farthestfrontier.wiki/wiki/Town_Center);
another guide gives 125 instead of 150 for tier 3, so check against the current game):

| FF tier | FF requires | Our step | Starting test |
| --- | --- | --- | --- |
| 1 | nothing | Camp | none |
| 2 | 8 shelters, 1 market, 30 villagers | an **established** Village | 30 Dancers, 8 cottages, plus a building in the market's role |
| 3 | 25 homesteads, 1 school, 1 trading post, 150 villagers | Town | 150 Dancers, 25 second-tier dwellings, plus buildings in the school's and trading post's roles |
| 4 | 25 large houses, 1 theater, 1 trading center, 300 villagers | City | 300 Dancers, 25 third-tier dwellings, plus the concert hall (the theater's role) and a trading centre |

These combine with the user's phase guidelines (a crop for Village; three villages
trading for Town; gallery, concert hall and light rail for City). They are rows
(`settlement_tiers`), tuned in the rules designer; which of our buildings fill FF's roles
is open, as is whether Town counts Dancers across the trading villages.

### D5. Professions and tasks as FF (user, 2026-11-20)

From FF's guide ([Villagers](https://farthestfrontier.com/?p=1752)):

- Everyone starts as a **laborer**: hauling, gathering reeds and clearing. Laborers are the
  pool the other professions draw from.
- The player sets **how many** Dancers work in each profession (a professions panel, FF's
  P screen), never steering individuals. A profession can have no more Dancers than the
  work places its buildings offer; a profession with no building is unavailable.
- **Builders** are drawn from laborers automatically when there is building to do, up to a
  cap the player sets.
- Each Dancer has **one task at a time** (working a place, building, hauling, crewing a
  recipe, idle), recorded on the roster so the game can show who is doing what.

### D6. Crewed recipes (user, 2026-11-20)

Once a village is established (D4, tier 2), a recipe row may name a **crew**: a profession
and a count. Starting it reserves that many free Dancers of the profession (D1's ledger) for
its duration and sets their task; it releases them when done. Without enough free Dancers
it does not start and reports how many are missing. Recipes without a crew (the player's
own crafts at the campfire) work as today.

### D7. Shortfalls are drift, never death

Provisioning follows the draft schedule (water at dawn and dusk, food at midday, a home at
night). A checkpoint that cannot be met records the shortfall per Dancer and per need. While
a settlement is short, no one arrives; prolonged shortfall makes a Dancer leave (the rate is
open). No Dancer dies.

### D8. Rules stay rows

New rule types in `stubby_recipes` (a Rule PR each, with schema and validator): `dwellings`
(capacity), `professions` (name, work places per building), recipe `crew`, `rations` (need,
checkpoint, quantity) and `settlement_tiers` (D4). The economy crate reads them through the
game's glue, as the rack reads machine rules.

## Risks

- Individual Dancers at City scale (hundreds) must stay cheap: the core holds plain
  vectors; only Dancers near the camera get models and animation.
- FF's numbers are a starting point for a different game; expect to retune once crops and
  provisioning run.

## Open

- What establishes a village beyond D4's starting test; whether population counts per
  village or across a town.
- Which of our buildings fill the market, school, trading post and trading centre roles.
- The leaving rate under prolonged shortage, and arrival rates.
- The first professions beyond laborer and builder (the first crop's tender, a water
  carrier).
