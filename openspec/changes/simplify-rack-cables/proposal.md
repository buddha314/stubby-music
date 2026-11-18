## Why

Configuring a burbler with a sequencer got too complicated (user, 2026-11-18, screenshot):
spare cables on a comb beside the player's own, a sequencer place the burbler's work
does not need, and no way to see where a patched cable starts and ends. The user decided
the direction.

## What Changes

- **No cable comb.** Every cable is one of the player's patch cables from the inventory:
  patching takes one, unplugging gives it back. The rack frame loses its comb.
- **A rack offers only what its machine needs.** The burbler's work is a heartbeat:
  clock, VCO and volume. It loses its optional sequencer place and melody patch. The
  campfire, which needs a sequencer, keeps one.
- **Hovering a patched cable lights it end to end**, so its two ends are plain.
- **A known patch guides its second end.** With a cable's first end in one end of a
  suggested patch, only the other end's jack shows, blinking; the cable takes the patch's
  colour.
- A cable starts on a jack; only its far end snaps from nearby.

## Capabilities

### New Capabilities
- `rack-cables`: hovering, guidance and the per-machine places above.

### Modified Capabilities
None accepted. The in-flight `add-rack-configurations`, `add-mudpit-burbler`,
`add-reed-water-purifier` and `sync-snail-shell-sequencer` are updated in place.

## Impact

`bevy_stubby_rack` (rack.rs, machine_config.rs), the burbler profiles and
`create_burbler.py`, the rack frame (`create_hud_rack.py`, `rack_frame.glb`), the
scripted rack review (`prefab_review.rs`), AGENTS.md.
