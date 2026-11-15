## Context

Six Jackdaw crates are path dependencies of two members:

| Member | Crates |
| --- | --- |
| `stubby_game` | `jackdaw_terrain` (no default features) |
| `stubby_jackdaw` | `jackdaw_terrain`, `jackdaw_prefab`, `jackdaw_runtime` (`pie`), `jackdaw_scene_types`; dev: `jackdaw_bsn` |

`check_jackdaw_versions.py` *requires* those paths to resolve into the shared checkout
and separately checks that the checkout's HEAD equals `compatibility.json`. So the
build follows the checkout, and correctness depends on two things agreeing that Cargo
never sees together. CI checks out the fork at the pinned revision into a sibling
directory to recreate the layout.

Jackdaw builds a game project as an ordinary Bevy app (`jackdaw_project_build`:
"the game keeps its own bevy, its own features, and its own toolchain") and runs it as
its own process, talking to the editor over the PIE protocol. Only editor extensions
compile against the editor's prebuilt SDK dylib. So the game's Jackdaw crates need the
same *revision* as the editor, not the same files on disk.

The fork's workspace defines no `[patch]` sections we would lose by consuming it as a
git dependency (Cargo ignores patches in dependency workspaces), and its crates take
Bevy from the same git `main` the game uses, so `Cargo.lock` keeps one Bevy.

## Goals / Non-Goals

**Goals:**
- The repository builds from a fresh clone, at exactly the recorded Jackdaw revision.
- One place to change the pin; the sync tool does it.
- Editor-fix development stays fast, with no manifest edits.

**Non-Goals:**
- Changing how the shared editor is built, installed or promoted.
- Publishing or vendoring Jackdaw crates.
- Changing the Bevy pin or the generated Bevy-main forks.

## Decisions

### `rev` in `[workspace.dependencies]`, not `branch` and not per member
`rev = "<sha>"` makes the manifest state the pin, so a `cargo update` can never move
Jackdaw by accident, and it matches `compatibility.json` one-to-one. Declaring the
crates once in the workspace means the pin is one line, and members only add
features (`jackdaw_runtime = { workspace = true, features = ["pie"] }`).
*Alternatives:* `branch = "main"` with the lock pinning the revision (the way Bevy is
handled today). Rejected for Jackdaw: Bevy follows `main` on purpose, but Jackdaw must
equal the editor exactly, and a branch dependency hides that in the lock. Per-member
`rev` lines: rejected; five places to keep equal.

### Local override through `[patch]` in an untracked `.cargo/config.toml`
Cargo applies `[patch."<git url>"]` from configuration files as it does from
`Cargo.toml`. `tools/jackdaw.sh link` writes one entry per Jackdaw crate the workspace
uses, pointing at `~/src/jackdaw/crates/<name>` (or `$JACKDAW_SOURCE`); `unlink`
deletes the file and runs `cargo update` for those crates so the lock returns to the
git source. `.cargo/config.toml` is gitignored.
*Alternatives:* editing `Cargo.toml` to paths while developing (rejected: it gets
committed); Cargo's legacy `paths = [...]` override (rejected: it cannot change a
crate's dependencies and is discouraged by Cargo).

### The checker validates the source, not the layout
The path rule becomes: every `jackdaw_*` package in `Cargo.lock` has source
`git+https://github.com/buddha314/jackdaw.git?rev=<pin>#<pin>`, the workspace
declaration names that URL and `rev`, and no member uses `path` for a Jackdaw crate. A
`jackdaw_*` package without a source in the lock means a link is active, which gets
its own message pointing at `unlink`. The existing checkout-drift check stays, but
only on the managed-launch path (`tools/jackdaw.sh` launch and `prepare`), where the
editor actually has to match.

### The sync tool pins through Cargo
`repin` already rewrites `compatibility.json`, `jackdaw.toml` and the toolchain. It
also rewrites the workspace `rev` and runs `cargo update -p <each jackdaw crate>`
before pinning the Bevy forks and Bevy, so the lock resolves Jackdaw first.

## Risks / Trade-offs

- [The first build fetches the fork's whole repository] → a one-time clone into
  `~/.cargo/git`; later revisions fetch incrementally.
- [The fork must be pushed before the game can pin it] → already the policy (fixes go on
  fork main); `link` covers the time before a push.
- [Someone commits a lock made while linked] → the checker fails on it in CI and
  locally, naming `unlink`.
- [Jackdaw crates gain a dependency the root lock resolves differently from Jackdaw's
  own lock] → the same exposure exists today with paths; the existing Bevy-source and
  editor-receipt checks catch a mismatched Bevy.

## Migration Plan

1. Add the workspace declarations at the current pin (3c891ed, or whatever the sync has
   recorded when this lands) and switch members; `cargo build -p stubby_game -p
   stubby_jackdaw` and `cargo test -p stubby_jackdaw --test prefabs` pass.
2. Update the checker and its tests, then the sync tool, `link`/`unlink`, CI and docs.

Rollback: revert the manifests and the checker; nothing else depends on the change.

## Open Questions

- Should `link` be the default for the main checkout at `~/src/stubby-music`, where the
  editor is developed, and worktrees use the pin? (Proposed: no; explicit is safer.)
