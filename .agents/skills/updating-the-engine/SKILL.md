---
name: updating-the-engine
description: "Moves the game's pinned engine set (Bevy main, avian forks, the Jackdaw fork, Rust) forward safely, or checks that the pins agree. Use when asked to update Bevy, avian or Jackdaw, when a pin check fails, or when a needed fix is on a newer Bevy main."
---

# Updating the engine

> **Stub.** Shared by every game on this stack (`tools/update_engine_tools.sh` copies it from the template): improve it in the template. Add evaluations (see *To do*).

## When
Any change to the Bevy, avian or Jackdaw versions; never edit those versions by hand.

## Steps
1. Read `docs/ENGINE-UPDATES.md` first: Bevy moves only as far as avian builds (avian first).
2. `tools/jackdaw.sh ceiling`: the newest Bevy main commit the avian forks build on.
3. `tools/jackdaw.sh sync`: Bevy main -> avian forks -> Jackdaw fork -> this game, then `prepare`.
4. Regenerate the Bevy-main forks if their bases moved: `tools/bevy_main_forks.py ensure` (`bevy_main_forks.json`).
5. `tools/jackdaw.sh check`, then `tools/ci_local.sh`.

## Pitfalls (seen in this project)
- Every game's `bevy_main_forks.json` must stay identical: the fork branches are shared and force-pushed.
- jbuehler23/jackdaw is read-only: changes go to the buddha314 fork.
- Games on lightyear (the template's networking): its `net::transform_follows_position` workaround must stay until a fixed lightyear is pinned.

## Done when
`tools/jackdaw.sh check` and `tools/ci_local.sh` pass, including the game's physics and (where it has them) networking tests.

## Files
- `docs/ENGINE-UPDATES.md`
- `compatibility.json`
- `bevy_main_forks.json`
- `tools/jackdaw.sh`
- `tools/engine_sync.py`
- `tools/bevy_main_forks.py`

## To do
- [ ] Fold in the ENGINE-UPDATES checklist as a copyable checklist here.
- [ ] Evaluations: a sync with no change, a sync with a new Bevy commit, a failing pin.
