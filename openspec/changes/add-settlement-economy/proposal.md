## Why

Village is the next phase (`docs/design/roadmap.md`): crops, the first residents, and
recipes that need people to make them. The game has no model of people yet. The user
decided its shape on 2026-11-20:

- **Dancers are individuals**, not households or an aggregate.
- **A cottage holds at most four Dancers**, filled at random; larger dwellings for more
  Dancers come later.
- **Settlement size follows Farthest Frontier's population metrics** to begin with,
  adjusted as we go.
- **After a village is established, many recipes need a number of Dancers** to make them,
  and the game tracks each Dancer's task, as Farthest Frontier does.
- **The economy crate lives in this repository** unless it grows big.

Burb Sweeper's resolver and `automate-godot`'s rule engine already solve the accounting
(`docs/research/burb-sweeper-economy.md`, `docs/research/automate-godot-port.md`); this
change ports what fits into a Bevy-free core and adds the people.

## What Changes

- A new core crate, **`stubby_economy`** (never Bevy): a calendar-checkpoint dispatcher,
  the producer/converter/consumer/upkeep rule engine with deficits, and a labour ledger,
  ported from `automate-godot`; plus the Dancer roster, housing and tasking.
- **Dancers as individuals**: a stable ID, a seeded body shape and skin tone, a home, a
  profession, a current task and a local clock.
- **Housing**: dwellings are rows with a capacity; the cottage's is 4; new Dancers are
  housed at random, seeded and deterministic.
- **Provisioning** on the draft schedule (water at dawn and dusk, food at midday, shelter at
  night), every crossed checkpoint once and in order; shortfalls recorded, never deaths.
- **Arrivals** when there is spare shelter, water and food; none while short.
- **Professions and tasks, as Farthest Frontier**: everyone starts as a laborer; the player
  sets counts per profession, up to the work places buildings offer; builders are drawn
  from laborers up to a cap; each Dancer has one task at a time.
- **Crewed recipes**: a recipe may need a crew of Dancers of a profession for its
  duration; it waits, and says how many are missing, until they are free.
- **Settlement tiers** as rule rows, starting from Farthest Frontier's Town Center table.
- New rule types in `stubby_recipes`: dwellings, professions, recipe crews, settlement
  tiers, rations. Each lands with its schema, validator and rows.
- **A headless simulation tool** (`economy-sim`) that runs scenarios for many days and
  reports population, stock, shortfalls and tasks per checkpoint: the start of the balance
  harness the roadmap calls for.

Out of scope here: the Bevy system and its screens (a professions panel, Dancer models and
animation), crops, trade and the Dancers' art, which follow as their own changes.

## Impact

- New crate `crates/stubby_economy` (core layer), a `deny.toml` line, and the cores check
  in `tools/ci_local.sh`.
- `stubby_recipes`: new rule types and rows; recipes gain an optional crew with a
  duration (recipes are instant today).
- `openspec/changes/add-settlement-phase-reviews`: its population task (2.5) and
  transition conditions (2.4) take their starting values from here.
