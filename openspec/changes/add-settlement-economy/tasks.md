Ordered so each group builds on the one before; every task ends in a check. Claim a task by
adding `(claimed: <agent>)` to its line in the PR with its first commit. Tests stay minimal
(AGENTS.md): one focused test per rule we own, run with `cargo test -p stubby_economy`;
`tools/ci_local.sh` once before pushing.

## 0. Decide (user)
- [x] 0.1 Dancers are individuals (user, 2026-11-20)
- [x] 0.2 A cottage holds at most four, filled at random; larger dwellings later (user, 2026-11-20)
- [x] 0.3 Town size from Farthest Frontier's population metrics, adjusted as we go (user, 2026-11-20)
- [x] 0.4 Established villages have crewed recipes; Dancers' tasks are tracked as in Farthest Frontier (user, 2026-11-20)
- [x] 0.5 The crate lives in this repository unless it grows big (user, 2026-11-20)
- [ ] 0.6 Which of our buildings fill the market, school, trading post and trading centre roles (until then, tier rows name placeholder building IDs)
- [ ] 0.7 Whether Town counts Dancers per village or across the trading villages
- [ ] 0.8 Arrival and leaving rates; the first professions beyond laborer and builder
- [ ] 0.9 Confirm "random" means random placement into cottages of four, not a random capacity per cottage

## 1. The crate
- [ ] 1.1 Create `crates/stubby_economy` (edition 2024; `serde`, `rand_chacha`; no Bevy), add it to the workspace, and a `deny.toml` line (wrappers: `stubby_game`, `stubby_jackdaw` and the future settlement system crate)
- [ ] 1.2 Add it to the cores' no-Bevy `cargo tree` check in `tools/ci_local.sh` and the CI workflow
- [ ] 1.3 Module layout: `time`, `rules`, `ledger`, `roster`, `housing`, `provision`, `labour`, `crews`, `tiers`, `state` (the whole settlement as one serialisable value); crate docs name `automate-godot` as the source of the ported parts
- [ ] 1.4 AGENTS.md: the crate in the repo map and in the layer table as a core

## 2. Ported from automate-godot (`docs/research/automate-godot-port.md`)
- [ ] 2.1 `Checkpoint` (dawn, midday, dusk, night) and a monotonic checkpoint index from world time (`stubby_world::calendar` values passed in as numbers, not as a dependency)
- [ ] 2.2 Dispatcher: advancing to a world time yields each crossed checkpoint once, in order; the last processed index is part of the state
- [ ] 2.3 Rules: producer, converter, consumer and upkeep with per-unit inputs and outputs, resolved per checkpoint against bound unit counts; producers and converters first; whole conversions; zero-clamped stock with deficits reported
- [ ] 2.4 Ledger: named reservations of Dancers, atomic and idempotent, resizable, with a versioned snapshot
- [ ] 2.5 Port the four smoke tests: apply order, whole conversions, deficits, and a jump equal to its steps (spec: "Checkpoints resolve in order", "Rules produce before they consume")

## 3. The roster
- [ ] 3.1 `DancerId` (never reused) and `Dancer`: body shape, skin tone, home, profession, task, clock ID
- [ ] 3.2 Appearance drawn from the world seed and the Dancer's ID (four shapes, four tones; counts come from rows)
- [ ] 3.3 Counts (population, per profession, idle, homeless) derived from the roster, never stored
- [ ] 3.4 The settlement state round-trips through serde unchanged (spec: "Dancers are individuals")

## 4. Housing
- [ ] 4.1 Dwellings: placed instances of a dwelling kind with a capacity from rows
- [ ] 4.2 A new Dancer goes to a dwelling with a free place, drawn from a seeded stream; same seed and history, same homes (spec: "A cottage houses at most four")
- [ ] 4.3 A removed dwelling leaves its Dancers homeless until a place frees up; homelessness is the night checkpoint's shelter shortfall

## 5. Provisioning, arrivals and leaving
- [ ] 5.1 Rations as consumer rules on their checkpoints (water at dawn and dusk, food at midday, a home at night), from rows
- [ ] 5.2 Record each checkpoint's intended demand, what was met and the shortfall, per need and per Dancer (Burb Sweeper's ledger)
- [ ] 5.3 Reserves: stock needed for the next day's rations is held back; only the rest counts as surplus (for arrivals now, trade later)
- [ ] 5.4 Arrivals: only with a free home and the next day's rations covered for everyone; at most one per checkpoint until the rate is decided (spec: "Dancers arrive only to a settlement that can keep them")
- [ ] 5.5 Leaving after prolonged shortfall, at a rate from rows, off by default; never death (spec: "Shortfalls never kill")
- [ ] 5.6 A jump equals its steps with arrivals and shortfalls in play

## 6. Professions and tasks
- [ ] 6.1 Professions from rows, with the work places each building kind offers
- [ ] 6.2 Everyone starts as a laborer; the player's count per profession is clamped to its work places; a profession without a building is unavailable (spec: "Professions are set by count")
- [ ] 6.3 Which Dancers fill a profession is deterministic (the longest-idle laborer first, then the lowest ID)
- [ ] 6.4 Builders drawn from laborers while there is building to do, up to the player's cap, and returned after
- [ ] 6.5 `Task`: working a place, building, hauling, crewing a recipe, idle; exactly one per Dancer, and a query for who is doing what

## 7. Crewed recipes
- [ ] 7.1 A crew request (profession, Dancers, seconds) reserves free Dancers through the ledger, takes the inputs and sets their task
- [ ] 7.2 Not enough free: nothing is taken and the report says how many are missing (spec: "Crewed recipes reserve Dancers")
- [ ] 7.3 Completion at its world time delivers the outputs and releases the crew, exactly once, across jumps and reloads
- [ ] 7.4 Crewed recipes only in a settlement at the established tier or above

## 8. Settlement tiers
- [ ] 8.1 Evaluate a settlement's tier from rows: population, dwellings of a kind, required buildings; the highest met tier wins
- [ ] 8.2 The established scenario (30 Dancers, 8 cottages, the required building) (spec: "Settlement tiers are rows")

## 9. Rules (`stubby_recipes`; a Rule PR, schema first)
- [ ] 9.1 Rule types and validation: `dwellings` (capacity), `professions` (building work places), `rations` (need, checkpoint, quantity), `settlement_tiers` (population, dwellings, buildings) and a recipe's optional `crew` (profession, dancers, seconds)
- [ ] 9.2 `rules-fmt`: schema regenerated, new IDs in `ids.lock`; the designer's forms follow from the schema
- [ ] 9.3 Rows: the cottage (4), laborer and builder, the draft rations, Farthest Frontier's four tiers with placeholder buildings, one example crewed recipe
- [ ] 9.4 `assets/recipes/README.md`: how to add a dwelling, a profession and a crew

## 10. A headless simulation tool
- [ ] 10.1 `cargo run -p stubby_economy --bin economy-sim -- <scenario.json>`: runs a scenario (seed, rows, starting stock, dwellings, scripted actions at world times) for N days and writes one row per checkpoint (population, stock, shortfalls, tasks)
- [ ] 10.2 Three scenarios from the economy review: a water shortage, a crew that leaves the village short, and growth to the established tier; record their output in `docs/research/`
- [ ] 10.3 Report provisioned Dancer-days and the binding shortfall per run (Burb Sweeper's measure)

## 11. Close
- [ ] 11.1 Update the roadmap's foundations and `add-settlement-phase-reviews` 2.4/2.5 with what was built
- [ ] 11.2 `openspec validate --strict`, then archive once the follow-up changes (section 12) are opened

## 12. Follow-up changes (opened separately, not tasks here)
- [ ] 12.1 `add-settlement-system`: the Bevy system crate driving the core from the calendar, saving and loading it, and the professions panel (Farthest Frontier's P screen)
- [ ] 12.2 `add-dancers`: models, the 16 appearances, clock-driven animation, and only nearby Dancers drawn
- [ ] 12.3 `add-first-crop`: fields, water demand and the Camp → Village transition
- [ ] 12.4 `add-village-trade`: routes between villages and the Village → Town transition
