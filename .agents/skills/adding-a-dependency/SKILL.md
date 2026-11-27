---
name: adding-a-dependency
description: "Adds a Rust crate to the game without breaking the single pinned Bevy: forks Bevy-dependent crates, re-pins Bevy, undoes unrelated Cargo.lock drift, and clears the licence check. Use whenever a crate is added to any Cargo.toml in the workspace."
---

# Adding a dependency

> **Stub.** Shared by every game on this stack (`tools/update_engine_tools.sh` copies it from the template): improve it in the template. Add evaluations (see *To do*).

## When
Adding or upgrading any dependency.

## Steps
1. Prefer an existing crate to custom code; check whether it depends on Bevy (`cargo tree` after adding, or its Cargo.toml).
2. Bevy-dependent and no Bevy-main release: add a fork entry to `bevy_main_forks.json`, run `tools/bevy_main_forks.py ensure`, and add the crate to `[patch.crates-io]` in the root `Cargo.toml`. Optional dependencies need forks too (Cargo locks them).
3. Add it under `[workspace.dependencies]` with a comment saying why, and `x.workspace = true` in the member crate.
4. Resolve, then pin Bevy back: `cargo update -p bevy@0.20.0-dev --precise <rev from compatibility.json>`.
5. Read `git diff Cargo.lock`: only the new packages should appear. Put back anything else that moved (`cargo update -p <pkg>@<new> --precise <old>`, or point the lock entry back at the version still present) and confirm with `cargo metadata --locked`.
6. `cargo deny --log-level error check`: a new licence needs an entry in `deny.toml` with the reason (and in `about.toml`'s `accepted`, where the game generates notices with cargo-about).

## Pitfalls (seen in this project)
- Resolving can move part of Bevy to a newer commit that needs a newer Rust (seen 2026-11-27: `bevy_ecs` requiring Rust 1.100); always re-pin.
- Collateral drift seen: `glam` for avian's `obvhs`, `windows` for `gpu-allocator`/`gilrs-core`.

## Done when
`python3 tools/check_engine_pins.py`, `cargo deny check` and `tools/ci_local.sh` pass; the lock diff is only the new crate and its own dependencies.

## Files
- `Cargo.toml`
- `bevy_main_forks.json`
- `deny.toml`
- `about.toml` (where present)
- `docs/ENGINE-UPDATES.md`

## To do
- [ ] A helper that re-pins and reports lock drift automatically.
- [ ] Evaluations: a pure-Rust crate, a Bevy plugin crate, a crate under a new licence.
