## 1. Proposal and baseline

- [x] 1.1 (claimed: codex) Record fork topology, current revisions and compatibility policy
- [x] 1.2 Validate this proposal before implementation

## 2. Normalize project versions

- [x] 2.1 (claimed: codex) Unify Cargo workspaces, Bevy source and lockfile; retain saved prefab type paths
- [x] 2.2 Record Jackdaw/Bevy/compiler revisions and add a read-only compatibility checker with mismatch tests
- [x] 2.3 Enforce the checker in project tooling and document upstream sync/promotion/rollback
- [x] 2.4 One focused compatibility build and source/version validation

## 3. Upstream integration

- [x] 3.1 (claimed: codex) Select supported upstream release-0.20 on an isolated branch, preserving the existing PIE fix
- [x] 3.2 One focused build: both game entry points compile against supported Jackdaw 0.20; no broad suites
- [x] 3.3 Supported release-0.20 has Physics enabled and the required native LOD APIs; original PIE fix retained
- [x] 3.4 Promote only a validated candidate to the shared editor and record its exact revision; retain bounded graphical diagnostics

## User correction: minimal testing (2026-11-01)

The user stopped the broad upstream test/documentation runs as excessive for this
prototype. All owned validation jobs were terminated. Do not resume full suites;
use the shared AGENTS.md prototype policy. Existing focused checks and the bounded
smoke test passed. The unrelated Bevy-main sync remains uncommitted and unpromoted. The supported dev/0.20 branch is now the active alignment target.

## Alignment checkpoint

Both game entry points compile with Jackdaw 0.20.1-rc.1 / Bevy 0.20.0 / Rust
1.99.0. Shared editor and CLI build successfully from the recorded dev/0.20
commit. jbuehler23 is read-only, with the upstream push URL disabled; publication
is only to buddha314. Stop after committing/pushing this alignment. Physics and
machine configuration are deliberately not implemented in this checkpoint.
