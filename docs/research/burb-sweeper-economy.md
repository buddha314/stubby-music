# Burb Sweeper economy rules for Stubby Music

Burb Sweeper is a repository owned by the Stubby Music team, as confirmed by
the user. This review reads the local checkout at `/home/buddha/src/burb-sweeper`
at commit `b0af4724b20d8ca335a6c9232b828de0ffd7f2b5`. Its strongest precedent is an
aggregate economy where residents both consume supplies and provide the labour
needed to maintain and expand a settlement. [VerdantAI/burb-sweeper](https://github.com/VerdantAI/burb-sweeper)

## Implemented economy rules

| Rule | What the code does |
| --- | --- |
| Resident consumption | For a fractional day, demand equals population × per-resident daily rate × fraction elapsed. Default rates are one unit each of food, water and breathable air per resident per day. Residents also add one unit each of waste and unbreathable air per day by default. The resolver records unmet needs and clamps depleted stocks to zero. Heat or drought can raise water demand. |
| Housing and population | The map projects housing capacity from placed objects. Authored occupancy can seed the initial population. Later capacity changes affect the growth ceiling without reseeding the population. Healthy growth accrues fractional births at a rate that tapers near capacity. Shortages accrue declines with a multiplier for the number of unmet needs. Population accrual uses a day-start headcount for fractional steps. |
| A shared workforce | Build jobs reserve people from the same pool as the resident population. The pool deducts people recovering from exposure and people already committed to active jobs. A construction queue reports missing hands or resources, starts jobs in order, and spends the full resource cost at start. Stalled jobs retain their remaining time. |
| Resource flows | Survival and player action flows accumulate in one daily ledger. It records production, intended demand, construction, demolition returns, crafting, harvesting and applicable spoilage. Intended demand can exceed actual stock, which helps a designer see the shortfall. The climate clock estimates days remaining at present consumption from the tightest food, water or air stock. This estimate omits future inflows. |

Sources: [Survival resolver](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/survival_rules.gd) · [Runtime state, queue and ledger](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/global_state.gd) · [Map metrics and action previews](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/map_builder.gd#L1865) · [Build job stepping](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/build_job_rules.gd)

## Land, food and construction pressure

- **Rain and ground cover.** Rain is seeded by level and resolved day. Permeable land retains its share of rainfall. Staffed capture works can retain some runoff from impermeable areas. Forecast events can also contribute to separate stormwater storage. Solar inflow scales with the daylight fraction. [Rain calendar](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/rain_calendar.gd) · [Survival resolver](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/survival_rules.gd)
- **Food has a binding input.** The resolver multiplies potential harvest by the lowest available fraction of labour, power and irrigation water, then charges the corresponding share of power and water. Food forests and aquaponics have different yields, inputs and staffing costs. The balance harness aggregates those rates and land permeability from its board counts. The live map metric projection supplies housing, storage, electricity, stormwater and air services. The audit found the assignment of placed-crop yields, irrigation demand, upkeep demand and permeability totals in the harness. [Map metric projection](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/map_builder.gd#L1865) · [Harness aggregation](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/tools/balance_harness.gd#L378)
- **Restoration requires upkeep.** The resolver computes maintenance coverage and reports missed staffing below a threshold. The harness turns neglected plantings back into open ground. This creates a feedback loop where losing residents can reduce demand and erode production at the same time. The housing ladder increases capacity from 4 to 8 to 16 to 32 across single-family homes, duplexes, rowhouses and low-rises, with higher construction costs, crew sizes and durations. [Harness reversion](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/tools/balance_harness.gd#L301) · [Placeable definitions](https://github.com/VerdantAI/burb-sweeper/tree/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/placeables)

## Long runs and proposals

The headless balance harness uses the same survival resolver and a simplified
board. It runs strategies such as doing nothing, demolishing, densifying,
restoring and farming for up to 400 days. Each row reports person-days,
lifespan, ending population, build mix, permeability, staffing and failure
cause. Person-days gives credit for sustaining residents over time and exposes
a strategy that survives mainly by shedding population. The harness can compare
whole-day and quarter-day runs with tolerances for cumulative differences.
The source review did not run the balance harness. [Balance harness](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/scripts/tools/balance_harness.gd)

Satisfaction, specialist output, staffed logistics shifts and richer worker
behaviour appear in open proposals. The audited settlement resolver and
construction queue did not supply a three-village goods exchange, market
pricing, or wages. Stubby Music's inter-village trade milestone still needs
its own design. [Population and specialists tasks](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/openspec/changes/add-population-and-specialists/tasks.md) · [City workers tasks](https://github.com/VerdantAI/burb-sweeper/blob/b0af4724b20d8ca335a6c9232b828de0ffd7f2b5/openspec/changes/add-city-workers/tasks.md)

## Candidates for Dancers

1. **Account for needs at each settlement.** Track stock, required rations,
   fulfilled rations and unmet demand at each scheduled withdrawal. Keep
   shelter as capacity. Show the next provisioning shortfall and how much
   stock is available to trade after resident reserves.
2. **Tie farm output to the inputs the player can manage.** Start by testing
   clean irrigation and tending as constraints. Record crop output and the
   limiting input so the player can tell a dry field from an understaffed one.
   Supply comes from Stubby Music's water networks.
3. **Test one resident labour pool before assigning individuals to jobs.**
   Compare how building and tending compete for the same hands. If hauling
   uses that pool later, make its cost and priority visible. Dancer body
   choices and local clock animation can stay independent of the accounting.
4. **Compare repeatable scenarios.** Test a water shortage, a new crop, and a
   shipment that would leave a village without its next ration. Track
   provisioned Dancer-days, uncovered needs and actual goods delivered into
   the trade network. Verify that advancing across several ration checkpoints
   gives the same accounting as stepping each checkpoint in order.

These are proposals for consideration. Burb Sweeper's rates, air stock,
rainfall-only water supply and loss condition belong to its survival economy.
The [draft Dancer schedule](../../openspec/changes/add-settlement-phase-reviews/design.md#draft-dancer-consumption-schedule)
holds Stubby Music's current design.
