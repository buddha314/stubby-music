## Why

The standalone game and editor adapter select different Bevy sources. The shared
Jackdaw path dependencies are not pinned by Cargo.lock, and upstream updates are
currently integrated manually without a reproducible promotion check. This makes
plugin compatibility and regression diagnosis unnecessarily difficult.

## What Changes

- Make a tested Jackdaw source commit the version authority for the game.
- Unify game and editor adapter into one Cargo workspace, one Bevy dependency source,
  one lockfile and the Rust toolchain selected by Jackdaw.
- Record the shared editor commit, upstream base, Bevy version/source and compiler in a
  machine-readable compatibility record, with read-only validation before launch.
- Document and check the existing fork workflow: upstream reference, tested fork
  main, focused fixes, temporary sync worktrees, merge-based upstream integration.
- Use the existing Physics-supported dev/0.20 line based on upstream release-0.20,
  retain the PIE projection fix, and promote it after one relevant compatibility build.

## Capabilities

### New Capabilities
- `jackdaw-toolchain`: Jackdaw-led dependency selection, compatibility validation,
  and safe upstream integration/promotion.

## Impact

Root and adapter Cargo manifests/lockfiles, Rust toolchain, operator tooling,
agent guidance and Jackdaw documentation. Audio and recipe crates remain Bevy-free.
No game-specific editor features or replacement physics implementation. Physics
availability must be rechecked against the selected Jackdaw version, not assumed.
