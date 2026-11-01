## Why
The second chore is "Water the tree": the burbler's stream has to reach a scrub oak.
Streams follow the ground, and nothing lets the player shape it, so they need to dig: a
channel from a stream to where water should go. Recorded now (2026-11-01) so the chore
has something to depend on; the mechanic is not designed yet.

## What Changes
- The player can dig a channel in the mudflat that a stream can run along.
- A stream run to a tree waters it: the tree's water quality rises, which feeds its bloom
  (`add-scrub-oak-production`) and the `tree_watered` condition
  (`add-inventory-and-goals-hud` D11).

## Capabilities
### New Capabilities
- `digging`: shaping the ground so streams can be led where water is needed.

## Impact
Terrain (generated mudflat), stream laying (`add-mudpit-burbler` 2.7), the scrub oak's
water quality, a digging tool (possibly a new hand item, `hand.rs`), and art for dug
ground. All open; see design.
