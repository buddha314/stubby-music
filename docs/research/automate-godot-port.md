# automate-godot: what to port for the settlement economy

Reviewed 2026-11-20 at `/home/buddha/src/automate-godot` (VerdantAI, commit `c39f147`,
18 commits). It is a Godot 4 addon in GDScript, about 1,100 lines, the Godot counterpart
of `automate-fvtt`, and the generalised form of Burb Sweeper's survival resolver
(`docs/research/burb-sweeper-economy.md`). Nothing in it runs in Rust, so using it here
means a port (user, 2026-11-20). The port is small; the value is the design, which is
already tested in two games.

## What it contains

| Module | Lines | What it does | Port? |
| --- | --- | --- | --- |
| `time/tick_dispatcher.gd` | 32 | The one seam between game time and rules: fans elapsed intervals out to subscribers **one interval at a time**, so a 30-day jump resolves exactly as 30 single days. | **Yes.** It is the Dancer schedule's "process every crossed checkpoint once, in order" rule. |
| `rules/rule.gd`, `rules/rule_engine.gd` | 100 | Pure, static interval resolution. Rules are producer, converter, consumer or upkeep, with per-unit `inputs` and `outputs`, multiplied by bound unit counts. Producers and converters create before consumers and upkeep spend; converters stop at whole affordable conversions; shortfalls clamp to zero and come back as `deficits`. | **Yes.** The settlement resolver's core: crops produce, Dancers consume, filters and kitchens convert, buildings take upkeep. |
| `workforce/workforce_pool.gd` | 86 | Atomic, idempotent named reservations of capacity, resizable, with a versioned snapshot for saving. Population, identity and job meaning stay with the host. | **Yes.** Burb Sweeper's single labour pool: building and tending share Dancers. |
| `harvest/yield_rules.gd` | 136 | One extraction model for every finite or renewing store: `yields` per unit, `capacity` (0 = infinite), `refill` (none, timed, per day, by deposit) and `take` (one or all). Pond, forest, basket, garden and seam are rows. | **As a design**, not code: our `resources.json` replenishment (`next_night`, `after`) and `mines.json` are narrower versions; generalise them when crops and stores arrive (a Rule PR). |
| `crafting/*`, `integrations/crafting_adapter.gd` | 235 | Recipe registry, stockpile with atomic multi-resource spend and shortfall report, and a prepare/complete job payload for host-scheduled crafts. | **No.** `stubby_recipes` already owns recipes, inventory and crafting. Borrow one idea: a timed craft as a portable job record that reserves its inputs at the start and completes once. |
| `terrain/bench_rules.gd` | 424 | Grading hillside benches for suburban lots. | No; Burb Sweeper specific. |
| `add-gym-adapter` (proposal) | — | Vectorised headless environments over a pure rules core, for reinforcement learning. | Later, if ever; the balance harness comes first. |

## How it would land here

**A Bevy-free core crate**, shaped like `stubby_recipes`: core logic with no Bevy, and a
thin Bevy plugin to drive it from the calendar. It is generic enough to publish as its own
crate later, as `automate-godot` is an addon; keep its API free of game names.

- **Layer:** a core (never Bevy), in `deny.toml` like the others. The settlement system
  (a new system crate) and the game's glue use it; `stubby_recipes` stays the owner of
  rows, so the economy's rules (crop yields, Dancer rations, upkeep) are rows there, read
  through `shipped_catalog()`, and handed to the engine as plain data.
- **Time:** the dispatcher steps on the calendar's checkpoints (dawn, midday, dusk, night;
  `stubby_world::calendar`), not on frames, and resumes from the last processed checkpoint
  on load. Musical timing stays on the audio clock; the economy is not musical time.
- **Quantities:** the economy's stock is settlement storage in real numbers, separate from
  the player's 16-box inventory of whole items.
- **Tests:** the source's own smoke tests are the spec: apply order, whole conversions,
  deficits, and the large-jump equals stepped-time check. Port those four and no more.

## Open

- The crate's name, and whether it lives in this repository or its own (VerdantAI or
  buddha314) for Burb Sweeper to share.
- Whether yields generalise `resources.json` now or when crops land.
- Whether the Godot addon stays the reference implementation or the Rust port becomes it.
