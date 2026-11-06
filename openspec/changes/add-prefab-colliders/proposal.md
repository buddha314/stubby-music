## Why

The player walks straight through scrub oaks and every other solid world asset. Collider shapes were worked out in each family's `colliders.json` but never authored, because Jackdaw's Physics module looked unavailable on Bevy main. Testing on 2026-11-06 showed the block is a one-line Jackdaw workspace bug, so this can now be done with Jackdaw's own component. It is tracked here for later.

## What Changes

- Fix the Jackdaw fork so `jackdaw_avian_integration` loads inside our workspace (`exclude = ["integrations"]` in its root `Cargo.toml`).
- Add that crate and `AvianColliderBridgePlugin` to the game, on our existing avian fork.
- Author Jackdaw's real `AvianCollider` on the six scrub oak prefabs, on child entities, from `assets/flora/scrub_oak/colliders.json`.
- Make `move_player` stop and slide against static colliders, as an interim step until the player body (`add-desert-opening` 4.2b).
- Then repeat for the other solid families: the burbler's hot-mud pool and stumps.

## Capabilities

### New Capabilities

- `prefab-colliders`: solid placed assets block the player through colliders authored on their prefabs; small assets stay walk-through.

### Modified Capabilities

None.

## Impact

- **Jackdaw fork:** one line in the root `Cargo.toml`, a workspace bug fix. No game-specific editor change.
- **`crates/stubby_jackdaw`, `crates/stubby_game`:** a new path dependency and plugin; `controls.rs` movement; `Cargo.lock`.
- **`assets/prefabs/flora/scrub_oak_*.bsn`:** a child collider entity on each.
- **`add-desert-opening` 4.2d:** this change carries that task's collider work.
- **Docs:** `docs/research/player-controller-and-physics.md` still says physics is gated.
