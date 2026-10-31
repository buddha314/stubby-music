# stubby_recipes

A small inventory/resource/recipe model independent of Bevy and audio.

`Catalog::from_json` validates item, world resource and recipe definitions.
`Catalog::harvest` awards a resource yield once per stable world-instance ID,
using a `HarvestLedger`. `Catalog::missing_ingredients` reports deficits.
`Catalog::craft` checks all inputs and output quantity overflow before committing
any inventory changes. Duplicate ingredient lines are summed, not checked
independently. Failed crafting or harvesting changes neither inventory nor the
harvest ledger.

Data lives in `assets/recipes/catalog.json`. Lightning bugs yield the
`square_wave` item. This is a waveform resource for recipes, not an automatically
connected sound source. The production recipe list starts empty: the current entries are raw waveform
resources, analogous to coal, rather than assumed finished-module recipes. Tests use a clearly test-only output to exercise
recipe mechanics without defining game progression.

A recipe definition has this shape:

```json
{
  "id": "recipe_id",
  "name": "Recipe name",
  "inputs": [{"item": "ingredient_id", "quantity": 2}],
  "outputs": [{"item": "output_id", "quantity": 1}]
}
```

Both item IDs must exist in the catalog. Quantities must be positive and totals
must fit `u32`. IDs are nonempty ASCII letters/digits, underscores or hyphens.
There are no crafting durations, machines, persistence or unlock rules yet;
those can be layered on this transaction boundary.

In the game, E recovers the square wave from the nearest uncollected lightning
bug within 3 metres. The HUD reports inventory and nearby interaction. The bug
continues its existing flash behaviour; recovering its pattern does not kill it
or connect audio. Press O for the recipe book, which remembers Square wave and its VCO, LFO and
gate roles even after inventory is consumed. A future crafting UI or machine sends `CraftRequest { recipe }` and
receives `CraftCompleted` only when the transaction succeeds.

Source objects distinguish exhaustible resources from infinite mines. Use
`Catalog::source` to create validated instances, then `harvest_source` for finite
collection; mine objects explicitly reject that operation. `MineDefinition`
references a produced resource, so a scrub oak produces finite bugs rather than
directly granting item stacks. Infinite capacity is separate from production rate.
Objects serialize with a `kind` tag; mine objects have no depletion flag/count.
