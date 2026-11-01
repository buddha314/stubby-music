# Jackdaw version policy

The game follows the shared Jackdaw **dev/0.20** branch, based on upstream
**release-0.20**. This existing supported line provides Avian Physics and the LOD
APIs the game uses. Fork main remains the upstream-development tracking line; do
not move this game to Bevy-main just to make version numbers look current.

`jackdaw/compatibility.json` records the exact shared source commit, upstream base,
Jackdaw release, Bevy version/source and Rust toolchain. Both game entry points
share one root Cargo workspace, one Bevy declaration and one Cargo.lock. The
adapter package remains to preserve saved-prefab type paths. Domain/audio code
remains independent of the game engine.

Keep the shared source and game as sibling directories. Run
`tools/jackdaw.sh check` to detect source or version drift. Prepare builds matching
shared editor/CLI binaries and records their local hashes; managed launches check
that receipt. It refuses to rebuild an editor the user is running. No per-game
editor is installed.

## Updates

Use focused `fix/<issue>` branches for bug fixes and publish them only to buddha314. For
supported-line updates, fetch upstream/release-0.20 and merge it into a temporary
branch from origin/dev/0.20. Keep our PIE light-probe fix and its regression.
Preserve shared history: do not force-sync/reset away local changes or rebase
published integration branches. Main can track upstream/main independently.

**We are prototyping.** Use one relevant build or focused regression for a sync.
Do not run full upstream suites, documentation builds or doctests by default, and
do not install extra testing infrastructure merely to run them. One bounded
20-second smoke test is enough when graphics are affected. Stop once relevant
checks pass. Earlier broad test runs were stopped by the user; do not resume them.

Publish the supported fork commit first. Update the game's source/engine/compiler
record and lockfile together. Promote the shared checkout only when clean and
idle, then build the editor and CLI from that same source. Keep the prior tested
revision for rollback. Do not touch another agent's unsaved game scene. A source
pin must not be silently rewritten to whatever checkout happens to be present.

## References

- [GitHub fork synchronization](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/syncing-a-fork)
- [Git topic and integration branches](https://git-scm.com/docs/gitworkflows)

Hard publication boundary: jbuehler23 is fetch/read-only. No pushes or PRs to that account; all publication goes only to buddha314.


## Promotion: orphan Assets row fix (2026-11-01)

Promoted fork `dev/0.20` to `3ca936dec8f09278598c8f9b128aea88cc3940fe`.
A deferred Project folder refresh could outlive its dock destination, leaving an
orphan UI root over the title/menu bar. Guard subtree creation at command execution
time, skipping retired destinations. Live BRP inspection confirmed the orphan;
removing that UI row restored the affected session's menu without scene edits.

One focused retired-panel regression passed. The editor executable built, and an
independent 20-second 60 Hz review reset the layout three times and opened Project:
the browser remained at the bottom and the menu visible. No relationship errors
or panics were logged. The watchdog ended the review at 20.023 seconds. The tested
executable was installed into the shared source's target/debug while the editor
was idle; no separate per-game editor was installed. Bevy and Rust pins are unchanged.

At the user's explicit direction, filed the general bug report with AI disclosure:
https://github.com/jbuehler23/jackdaw/issues/735 . No upstream PR or code push;
code publication remains limited to buddha314.
