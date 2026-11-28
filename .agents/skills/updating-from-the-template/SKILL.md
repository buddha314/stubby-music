---
name: updating-from-the-template
description: "Brings a game up to date with the us-outpost-31 template it was made from: fetches the template, lists what changed since the game last synced (.template-sync), copies the shared engine tools, ports the rest area by area around the game's own names and changes, and records the new sync point. Use when the template has changed and a game made from it should get those changes, or when asked to sync, update or catch up a game with the template."
---

# Updating from the template

> **Stub, used once** (stubby-music, 2026-11-27). Shared by every game on this stack (`tools/update_engine_tools.sh` copies it from the template): improve it in the template. Add evaluations (see *To do*).

## When
In a game repository (not the template), when the template has moved on.

## Steps
1. A GitHub template copy shares no history with the template, so changes are ported, never merged. Once per clone: `git remote add template https://github.com/VerdantAI/us-outpost-31.git`; then `git fetch template`.
2. Last sync point: `cat .template-sync`. Missing: look for the last shared-tools sync, whose commit message names the template commit (`git log --grep='from the template' --format='%h %s'`, e.g. *Sync engine tools ... from the template (116a927)*). Only for a game made from the template and never synced, fall back to the template commit before the game's first commit: `git log -1 --format=%H --before="$(git log --reverse --format=%cI | head -1)" template/main`. Confirm the choice with the user.
3. What changed: `git log --oneline <last>..template/main` and `git diff --stat <last> template/main`. Sort it: shared files (the list in `tools/update_engine_tools.sh`), engine pins (`compatibility.json`, `bevy_main_forks.json`), and everything else. Read the commit messages and any archived OpenSpec changes; tell the user what will come over before porting.
   **A game not made from the template** (stubby-music predates it and only adopted the shared tools) shares no game code with it: steps 4 and 5 still apply, but step 6 becomes adopting template features, a design choice. List them with what the game already has, and ask which to bring.
4. Shared files, unchanged: check out `template/main` in a worktree (`git worktree add ../template-sync template/main`) and run `tools/update_engine_tools.sh ../template-sync`. First check the game has not edited any of them since the last sync (`git diff --quiet <last>:<file> HEAD:<file>`); if it has, see Pitfalls.
5. Engine pins: if the template moved Bevy, avian or Jackdaw, follow `updating-the-engine` in the game; never copy the template's `Cargo.lock`.
6. Everything else, one area at a time (networking, menus, settings, audio, diagnostics, tools, docs, skills): `git diff <last> template/main -- <paths> > area.patch`, rewrite the template's names to the game's (`outpost_game`, `us_outpost_31`, `GAME_TITLE`, `APP_ID`), then `git apply -3 area.patch`. Resolve conflicts by keeping the game's own changes and bringing the template's intent; ask the user when they truly disagree.
7. Never take: the game's identity (names, `APP_ID`, title, Steam app id), its content (`docs/lore`, `docs/rules`, look book, art, `assets/`, scenes), its OpenSpec changes, or `Cargo.lock`.
8. Run `tools/ci_local.sh`; fix what the port broke.
9. Write the new sync point: `git rev-parse template/main > .template-sync`; remove the worktree; commit everything on a branch (`template-sync-<short sha>`) with the template range in the message; open a PR.

## Pitfalls (seen in this project)
- The template renames nothing for you: a patch naming `outpost_game` fails to apply until it is rewritten.
- Some template changes assume earlier ones; port in commit order when an area's patch will not apply.
- When the shared list itself grows, the game's old `update_engine_tools.sh` copies only the files it knows (and its own new version): run it a second time (stubby-music, 2026-11-27).
- A game may have changed a shared tool locally; `update_engine_tools.sh` overwrites it, so move that change to the template first (`sharing-tool-changes`).
- When a feature is ported into a game, port its tests too and run them in the game's own starting state: stubby-music's turned camera exposed a networking bug the template's tests could not see.
- **Games harden the template.** Anything a port exposes that is also true of the template (a bug, a test gap, a process lesson) is fixed in the template in the same round, with a test, and the lesson goes into the relevant skill. Then sync it back to the game.

## Done when
`tools/ci_local.sh` passes, `.template-sync` names the new template commit, and the PR lists what came over and what was deliberately left out.

## Files
- `.template-sync`
- `tools/update_engine_tools.sh`
- `tools/game_hooks.sh`
- `docs/ENGINE-UPDATES.md`

## To do
- [ ] Record what each feature adoption brought (for games not made from the template) so the next sync knows.
- [ ] `tools/template_sync.py`: fetch, show the range, write per-area patches with names rewritten, and update `.template-sync` (keep the judgement calls for the agent).
- [ ] Evaluations: a sync with only tool changes, one with a menu change the game had edited, one with an engine move.
