# stubby_recipes

A small inventory/resource/recipe model independent of Bevy and audio.

`rules::Rules::load` reads the rules files strictly and reports every problem with
file, ID and field; `shipped_catalog()` loads the files embedded from
`assets/recipes/`. `cargo run -p stubby_recipes --bin rules-fmt` formats them,
regenerates their JSON Schemas and updates `ids.lock` (see `assets/recipes/README.md`).
`Catalog::from_json` still validates a single-document catalog, for tests.
`Catalog::harvest` awards a resource yield once per stable world-instance ID,
using a `HarvestLedger`. `Catalog::missing_ingredients` reports deficits.
`Catalog::craft` checks all inputs and output quantity overflow before committing
any inventory changes. Duplicate ingredient lines are summed, not checked
independently. Failed crafting or harvesting changes neither inventory nor the
harvest ledger.

Data lives in `assets/recipes/` (`items.json`, `resources.json`, `mines.json`,
`recipes.json`, `migrations.json`). Lightning bugs yield the
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
must fit `u32`. IDs are `snake_case` (`[a-z][a-z0-9_]*`), unique across all files.
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
