# Resource and recipe catalog

`catalog.json` holds stable item IDs, resource yields and production recipes.
The first resource is `lightning_bug`; each world instance currently provides two
`square_wave` items (quantity is configurable). No production recipe is assumed yet. The general validator,
inventory and transaction APIs live in `crates/stubby_recipes`.

Square wave is in development. Keep the stable item/metadata boundary while its
asset is built. Collection feedback plays once per waveform, not per yielded item.

## Source kinds

`resources` are exhaustible pickups that yield item stacks once. `mines` are
infinite source definitions that produce resource instances. The shipped chain is:
`scrub_oak` mine → finite `lightning_bug` → `square_wave` items.

The oak definition names water quality and light quality as productivity factors.
Actual production and environmental scaling are deferred in
`openspec/changes/add-scrub-oak-production/`. No rates or tree asset are implied.
