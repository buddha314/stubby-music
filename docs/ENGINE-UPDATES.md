# Engine updates: Bevy, avian and Jackdaw as one set

*Shared by every game made from the template (VerdantAI/us-outpost-31), like the
tools it describes. Change it there and copy it out with
`tools/update_engine_tools.sh`.*

A game made from the template runs on three things that must move together:

- **Bevy**, from its git `main` branch;
- **avian** physics (plus its helpers), which has **no release for Bevy main**, so
  it comes from generated forks under buddha314;
- **Jackdaw**, the editor and its runtime, from our fork `buddha314/jackdaw`.

They are pinned as one set, recorded in `compatibility.json`. **Never change one of
them alone.** A Bevy that avian cannot build against leaves a game without physics,
and two Bevys in one graph do not link.

```
Bevy main ──▶ avian forks ──▶ buddha314/jackdaw main ──▶ this game
             (the ceiling)   (its Cargo.lock = the     (compatibility.json,
                              one Bevy revision)        Cargo.lock, jackdaw.toml)
```

The Jackdaw fork decides the Bevy revision, and it never pins past the newest
commit the avian forks build against (the **avian ceiling**). The full process for
moving that pin lives in the Jackdaw checkout: `docs/upgrading-bevy.md`. This game
only follows it.

## The files

| File | What it holds | Who writes it |
| --- | --- | --- |
| `compatibility.json` | Jackdaw fork revision, its Bevy source and revision, Rust | `sync` |
| `Cargo.toml` `[workspace.dependencies]` | Bevy (git main), avian3d, `jackdaw_*` at the fork `rev` | `sync` rewrites the `rev` |
| `Cargo.toml` `[patch.crates-io]` | the avian forks this game uses | by hand, from `tools/bevy_main_forks.py patch` |
| `Cargo.lock` | the exact Bevy, fork and Jackdaw commits | `sync` |
| `jackdaw.toml` `[jackdaw]` | the editor's version and `bevy_rev` | `sync` |
| `rust-toolchain.toml` | Jackdaw's compiler | `sync` |
| `bevy_main_forks.json` | which crates are forked, from which release (`base`) | by hand, **identical in every game** |

`tools/check_engine_pins.py` fails if any of these disagree. CI runs it, and every
`tools/jackdaw.sh` command runs it first.

## Everyday commands

```bash
tools/jackdaw.sh open      # the shared editor (../jackdaw) on this project
tools/jackdaw.sh build     # jd build: the game, plus the component schema the editor reads
tools/jackdaw.sh smoke     # one bounded 20 s Play; passes when Play reaches `running` with no panic
tools/jackdaw.sh check     # pins + shared checkout agree (changes nothing)
tools/ci_local.sh          # everything, before pushing
```

The shared editor lives in one checkout beside the games (`../jackdaw`, or
`$JACKDAW_SOURCE`) and serves every game. Never install a per-game editor.

## Following a Jackdaw/Bevy move: `tools/jackdaw.sh sync`

Run it when the Jackdaw fork has moved (or to pull upstream Jackdaw in), with the
editor closed and this repo clean:

1. **Upstream, avian first.** It merges new upstream Jackdaw (jbuehler23, read-only)
   into the fork through a PR on buddha314. If upstream moved Bevy, it first probes
   that commit against this game's avian forks (`scripts/bevy_ceiling.py probe` in
   the Jackdaw checkout). If they don't build there, it stops and merges nothing.
2. It fast-forwards the shared checkout to the fork's `main`.
3. It regenerates the avian forks (`tools/bevy_main_forks.py ensure`). That is a no-op
   unless `bevy_main_forks.json` changed.
4. It re-pins this game: every file in the table above.
5. It verifies the pins, builds the `jackdaw.toml` package, checks every workspace
   target, then rebuilds the shared editor (`prepare`).

**If the game stops compiling**, Bevy changed an API. Bevy's migration guides say
exactly what changed:

```bash
B=~/.cache/bevy-ceiling/bevy.git      # created by bevy_ceiling.py
git -C $B diff --name-only --diff-filter=A <old rev> <new rev> -- _release-content/migration-guides
git -C $B show <new rev>:_release-content/migration-guides/<file>.md
```

Port the game on a branch and rerun `sync`; the rerun is cheap. The sync prints a
rollback command if you'd rather back out.

**Then verify, in proportion:** `tools/ci_local.sh` (the template's includes a
headless physics test: a dropped cube must come to rest), then `tools/jackdaw.sh
build` and one `tools/jackdaw.sh smoke`. Commit the pins and any port on a branch, and open a PR.

## Asking how far Bevy can go: `tools/jackdaw.sh ceiling`

This bisects Bevy main for the newest commit at which this game's avian forks still
compile, and names the commit and crate that break past it. It takes minutes and
changes nothing. Use it before asking for a Jackdaw re-pin, or to check whether
upstream avian or parry has caught up. Moving the Jackdaw fork itself is done in the
Jackdaw checkout (its `docs/upgrading-bevy.md`), never from a game.

## The avian forks

`bevy_main_forks.json` lists the crates with no Bevy-main release. For each one,
`tools/bevy_main_forks.py ensure` takes the published release (`base`), points its
Bevy dependencies at Bevy's git main in one generated commit, and **force-pushes**
the fork's `bevy-main` branch. Those branches are **shared by every buddha314 game**
(stubby-music, this template, every game made from it).

- **Keep every game's `bevy_main_forks.json` identical.** `ensure` refuses to
  replace a branch that was generated from a different `base`, because that means
  two games disagree, and one would silently change the other's physics. To move
  every game to a new release on purpose, update the JSON in all of them and run
  `ensure --rebase-shared` once.
- **Patch only what the game uses.** `[patch.crates-io]` lists the forks in this
  game's graph. An unused patch makes Cargo warn on every command (the template
  leaves out `bevy_mod_outline`).
- **One avian.** The game takes `avian3d` from the workspace. Jackdaw's own Physics
  module (`jackdaw.toml` `[modules]`) depends on jbuehler23's avian fork, a second
  avian, so the pin check fails while `Cargo.lock` holds two. Keep the module off,
  or, to use it, add `[patch."https://github.com/jbuehler23/avian"]` entries that
  point `avian3d` and its helpers at our forks, so both resolve to one.
- **A fork that needs a real code change** (say, when Bevy moves glam) is done once,
  for every game: on its own branch under buddha314, recorded as that fork's `base`.
  Never commit by hand on `bevy-main`; the next `ensure` replaces it. See the Jackdaw
  checkout's `docs/upgrading-bevy.md`, step 3.

## Game-specific steps: `tools/game_hooks.sh`

The tools are identical in every game, so a game's own steps go in an optional
`tools/game_hooks.sh`, which `tools/jackdaw.sh` sources. It may define
`game_env` (run before anything builds or runs the game, e.g. exporting a native
SDK's path), `game_prepare` (in `prepare`, before the editor build, e.g.
generating assets) and `game_ready` (in `open`; return non-zero to force a
`prepare`). A game without special steps needs no hooks file.

## Updating the tools themselves

Improve the tools in the template, then run `tools/update_engine_tools.sh` in each
game. It copies the shared files unchanged, never touches `game_hooks.sh`, and
runs the pin-check tests against the game. Never edit a shared tool in one game
only.

## Working on the editor and the game together: `link`

`tools/jackdaw.sh link` points the game's `jackdaw_*` crates at the shared checkout
(an untracked `.cargo/config.toml`), so an unpushed Jackdaw fix can be tried in the
game. `tools/jackdaw.sh unlink` returns to the pin. The pin check fails while
linked, so a linked lockfile cannot be committed by accident.

## Rules

- Never push to jbuehler23 (Jackdaw or avian), by any route. All publication goes to
  buddha314 (forks) or this game's own remote.
- Never pin a game to a Bevy the Jackdaw fork has not pinned, and never edit the
  `bevy` / `avian3d` / `jackdaw_*` lines in a member crate: members inherit them
  from the workspace (`*.workspace = true`).
- We are prototyping. Verify enough to be sure and no more: `ci_local.sh`, one
  `build`, one `smoke`. Do not run Bevy's or Jackdaw's full suites for a game sync.
