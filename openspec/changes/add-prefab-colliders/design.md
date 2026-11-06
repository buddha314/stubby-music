## Context

Findings from 2026-11-06, on Jackdaw `706f2437` and Bevy `c9c397ba`:

- **Avian is not the blocker.** Avian 0.8.0 targets Bevy 0.20. Our generated fork (`bevy_main_forks.json`, `buddha314/avian` branch `bevy-main`) already runs it on Bevy main, in `interaction.rs`.
- **Jackdaw marks its module unavailable** (`package.metadata.jackdaw.unavailable`), because its physics workspace takes avian from crates.io, which does not build against Bevy main.
- **The crate failed to load for a Jackdaw workspace reason.** Added to our workspace it fails with `workspace.package.edition was not defined`: Cargo reads Jackdaw's root workspace, which has no `edition`, and not the physics sub-workspace. With `exclude = ["integrations"]` in the root manifest it resolves. Without the line, the error returns.
- **It compiles.** `cargo check -p jackdaw_avian_integration` passes on our fork with one Bevy in the graph. That is compile-only; no game code used it.
- **The player ignores physics.** `controls.rs` adds directly to `transform.translation`, so a collider alone blocks nothing.
- **No prefab has a collider today.** The orange boxes around a selected oak in Play are Jackdaw's selection highlight (`draw_highlight`), not colliders.

## Goals / Non-Goals

**Goals:**

- Scrub oaks block the player in Play and in the standalone game.
- Use Jackdaw's `AvianCollider`, so colliders stay editable as ordinary prefab components.
- Keep one Bevy and one avian in the dependency graph.

**Non-Goals:**

- The Tnua player body, enhanced input and terrain collider (`add-desert-opening` 4.2a to 4.2c).
- Enabling Jackdaw's editor-side physics tools, unless they turn out to be needed.
- Colliders on reeds, brush, debris or bugs, which stay walk-through.

## Decisions

**Use `AvianCollider`, not a private collider type.** The research note rules out private types, and the real component means no later prefab migration. A private `Solid { radius, height }` component is the fallback only if prefabs cannot load `AvianCollider`.

**Fix the Jackdaw workspace, on our fork.** The `exclude` line is a bug fix, which our editor rule allows, and it adapts nothing to this game. Copying the crate into this repo was rejected: it would drift from Jackdaw. Per the AGENTS.md hard rule, nothing is filed or sent upstream; record it in the fork-carried list.

**Colliders on child entities at the `colliders.json` translation, with no `RigidBody`.** This is the shape the file already specifies, and `StaticCollidersPlugin` supplies the static body. Children must not be scale-animated nodes: AGENTS.md forbids physics bodies on visuals whose scale animates, including the oak's blooms.

**Interim stop-and-slide in `move_player`.** Shape-cast the player's horizontal motion against static colliders and slide along the hit. It is small and is deleted by 4.2b. Going straight to Tnua was rejected for now because it also needs the terrain collider and retuning the movement.

**Oaks first.** They prove the whole path before the other families.

## Risks / Trade-offs

- **Prefab loading is unproven.** The editor build does not include the Physics module; its log says scenes keep those components, and a saved prefab must still load in Play. Test early (task 2.3), and fall back to the private component if it fails.
- **Each upstream sync can disturb this.** The `exclude` line and the `unavailable` marker live in files upstream may change. Check them when syncing.
- **`interaction.rs` shares the spatial query.** New static colliders will appear in its reach query. It only accepts hits on its markers, so this should be harmless; confirm E targeting still works near an oak.
- **Interim movement can feel wrong.** Corners and the player's fixed height are the likely issues. Keep the existing `controls.rs` tests as the bar.
- **Oak collider radii suit full bloom.** They were computed from the model, so a dormant tree may feel wider than it looks. Tune in the `colliders.json` values if so.
