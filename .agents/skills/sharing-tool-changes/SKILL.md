---
name: sharing-tool-changes
description: "Carries improvements to the shared engine tools from the us-outpost-31 template into each game (and back), keeping them byte-identical. Use when editing tools/jackdaw.sh, the pin checks, the fork generator or docs/ENGINE-UPDATES.md, in the template or in a game."
---

# Sharing tool changes

> **Stub.** Shared by every game on this stack (`tools/update_engine_tools.sh` copies it from the template): improve it in the template. Add evaluations (see *To do*).

## When
Any edit to a file in the shared list of `tools/update_engine_tools.sh`.

## Steps
1. Make the change in the template (VerdantAI/us-outpost-31), merged first.
2. In each game: `tools/update_engine_tools.sh`, review the diff, commit, PR (or a full `updating-from-the-template`).
3. Game-specific steps go in that game's `tools/game_hooks.sh`, never in a shared file.

## Pitfalls (seen in this project)
- A shared file edited in a game is overwritten by the next update: move the change to the template.
- `bevy_main_forks.json` must match in every game.

## Done when
The shared files are byte-identical (`diff` against the template) and each game's `tools/ci_local.sh` passes.

## Files
- `tools/update_engine_tools.sh`
- `tools/game_hooks.sh`

## To do
- [ ] List the games built from the template and their paths.
- [x] The stack-level skills and `tools/check_skills.py` are in the shared list (2026-11-27).
