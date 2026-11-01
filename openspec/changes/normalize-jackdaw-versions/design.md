## Selected compatibility set

Jackdaw is the authority. Use buddha314/jackdaw dev/0.20, based on upstream
release-0.20, with our existing PIE light-probe fix retained. This supported line
has Avian Physics and the native LOD APIs the game uses. Its lockfile selects
Bevy 0.20.0 and its toolchain is Rust 1.99.0. The exact fork commit is recorded in
jackdaw/compatibility.json; path dependencies alone never count as a source pin.

## Decisions

- Standalone game and editor adapter share one Cargo workspace, lockfile, exact
  Bevy declaration and compiler. Keep the adapter's reflected type paths so saved
  prefabs remain compatible. Audio and recipe crates remain engine-independent.
- The compatibility record identifies source/upstream commits, supported branch,
  Jackdaw version and matching Bevy source/version. A read-only checker rejects
  checkout, manifest or lockfile drift. Managed launches also check a local receipt
  tying editor/CLI binary hashes to the selected source commit.
- The shared source stays /home/buddha/src/jackdaw, with origin buddha314 and
  upstream jbuehler23. Active game development uses dev/0.20 and receives supported
  upstream release-0.20 updates. Fork main can follow upstream/main independently.
  Use focused fix branches and temporary sync worktrees; merge rather than reset
  away fixes or rewrite shared history.
- Promote only a clean, idle shared checkout. Keep a known-good revision for
  rollback. Never modify another agent's unsaved game scene.
- We are prototyping: use one focused compatibility build. Do not run broad
  upstream suites or documentation/doctest passes by default. The earlier broad
  main-sync runs were stopped by the user and are not to be resumed.

## Scope

Finish and publish alignment first, then return to Avian integration and the
nearby-machine configuration hook. No editor adaptation to game-specific needs,
new scheduled automation, or replacement physics implementation.

Hard publication boundary: jbuehler23 is fetch/read-only. No pushes or PRs to that account; all publication goes only to buddha314.
