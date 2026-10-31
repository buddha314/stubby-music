## Why
Loose lightning bugs are finite pickups, but their source mines are infinite. Separate these concepts before adding renewable resource production.

## What Changes
- Add exhaustible-resource and infinite-mine source objects.
- Add validated mine catalog definitions and a scrub-oak definition producing lightning bugs.
- Add an editor-registerable MineNode component without spawning placeholder trees or implementing production.

## Capabilities
### New Capabilities
- `resource-sources`: typed source identities, exhaustion and mine definitions.

## Impact
Pure recipe/domain model, catalog and Bevy components. Existing collection remains finite.
