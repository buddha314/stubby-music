## 0. Decide (user)
- [x] 0.1 Dancers are individuals (user, 2026-11-20)
- [x] 0.2 A cottage holds at most four, filled at random; larger dwellings later (user, 2026-11-20)
- [x] 0.3 Town size from Farthest Frontier's population metrics, adjusted as we go (user, 2026-11-20)
- [x] 0.4 Established villages have crewed recipes; Dancers' tasks are tracked as in Farthest Frontier (user, 2026-11-20)
- [x] 0.5 The crate lives in this repository unless it grows big (user, 2026-11-20)
- [ ] 0.6 Which of our buildings fill the market, school, trading post and trading centre roles
- [ ] 0.7 Whether Town counts Dancers per village or across the trading villages
- [ ] 0.8 Arrival and leaving rates; the first professions beyond laborer and builder

## 1. The core (`stubby_economy`)
- [ ] 1.1 Create the crate (core layer): `deny.toml` line, cores check in `tools/ci_local.sh`
- [ ] 1.2 Port the checkpoint dispatcher, rule engine and labour ledger from `automate-godot`, with its four smoke tests
- [ ] 1.3 The roster: Dancers with seeded appearance, home, profession, task and clock; derived counts
- [ ] 1.4 Housing by capacity with seeded random placement
- [ ] 1.5 Provisioning on the ration schedule; shortfalls; arrivals; leaving
- [ ] 1.6 Professions by count, work places, builder cap, one task per Dancer
- [ ] 1.7 Crewed recipes through the ledger
- [ ] 1.8 Settlement tiers from rows

## 2. Rules (`stubby_recipes`)
- [ ] 2.1 Rule types: `dwellings`, `professions`, recipe `crew`, `rations`, `settlement_tiers`; schema, validator, `rules-fmt`
- [ ] 2.2 Rows: the cottage (4), laborer and builder, the draft rations, FF's four tiers

## 3. Later changes (not this one)
- [ ] 3.1 The settlement system crate and the professions panel
- [ ] 3.2 Dancer models, appearance and clock-driven animation
- [ ] 3.3 The first crop; trade between villages
