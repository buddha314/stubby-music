# Player controller, physics and input for Bevy

Research, 2026-11-01. Question: is there an existing first-person camera and movement
crate we can use instead of our own `crates/stubby_game/src/controls.rs`? Versions and
Bevy requirements were read from the crates.io API on 2026-11-01, against **Bevy 0.20.1**.

**Decision (user, 2026-11-01): go straight to the long-term solution.** Adopt a physics
engine and an established character controller now, rather than extending our own
controller and switching when collision is needed.

## Where we are

`controls.rs` (235 lines) is a hand-written controller: WASD, Shift sprint, Space jump,
C crouch, mouse look, cursor capture, and the player's feet following the terrain's
height function. There is no physics engine, so the player walks through the burbler,
the scrub oaks and every prop. A factory builder needs the player to stop at machines.

## Options

| Option | Version (Bevy) | What it is | Verdict |
| --- | --- | --- | --- |
| [`avian3d`](https://crates.io/crates/avian3d) | 0.8.0 (`^0.20`) | The Bevy-native ECS physics engine; 466k downloads. | **Adopt.** It is Jackdaw's physics: Jackdaw's Physics module authors avian colliders and rigid bodies in the editor, and [`jackdaw_avian_integration`](https://crates.io/crates/jackdaw_avian_integration) targets avian 0.8 and Bevy 0.20. Editor and game share one engine. |
| [`bevy-tnua`](https://crates.io/crates/bevy-tnua) + [`bevy-tnua-avian3d`](https://crates.io/crates/bevy-tnua-avian3d) | 0.33.1 + 0.13.1 (`^0.20`, avian `^0.8`) | The established "floating" character controller: walking, running, jumping, crouching, slopes and stairs, as data-driven actions. 117k downloads; updated 2026-10-30. | **Adopt** for the player. |
| [`bevy_enhanced_input`](https://crates.io/crates/bevy_enhanced_input) | 0.27.2 (`^0.20`) | Action-based input modelled on Unreal's Enhanced Input: contexts, actions, bindings, modifiers. | **Adopt** for rebindable controls (the Satisfactory key map in AGENTS.md: E, O, Q, F, Tab, B). |
| [`leafwing-input-manager`](https://crates.io/crates/leafwing-input-manager) | 0.22.0 (`^0.20`) | The other established input manager; 610k downloads. | Equally viable. We prefer BEI for its Unreal-style contexts: gameplay versus the rack screen. |
| Bevy `free_camera` (`bevy_camera_controller`, feature flag) | ships with 0.20.1 | Bevy's first-party fly camera: no gravity, no collision. | **Use** as the debug/spectator camera. Nothing to add. |
| [`bevy_fps_controller`](https://crates.io/crates/bevy_fps_controller) | 20.0.0 (`^0.20`) | Source-engine movement (air strafing, bunny hops) on avian or rapier. | Works, but its movement suits a shooter, not Satisfactory-style walking. |
| [`bevy_rapier3d`](https://crates.io/crates/bevy_rapier3d) | 0.37.0 (`^0.20`) | The other physics engine. | Not chosen: Jackdaw uses avian. |
| [`bevy_ahoy`](https://crates.io/crates/bevy_ahoy) | 0.2.0 (avian `^0.7`, Bevy 0.19) | A newer kinematic controller on avian and BEI. | Not on Bevy 0.20 yet. |
| [`smooth-bevy-cameras`](https://crates.io/crates/smooth-bevy-cameras), [`bevy_dolly`](https://crates.io/crates/bevy_dolly), [`bevy_flycam`](https://crates.io/crates/bevy_flycam) | Bevy 0.16 / 0.15 / 0.19 | Camera helpers. | Behind Bevy. |

## The long-term setup

- **Physics:** `avian3d` 0.8. The terrain becomes a heightfield collider built from the
  same generator that makes the mesh, so it stays deterministic from the seed.
- **Player:** a dynamic rigid body with a capsule collider, moved by `bevy-tnua`
  (walk, sprint, jump, crouch), with the first-person camera at eye height (1.65 m,
  1.0 m crouched). Mouse look stays ours: Tnua moves the body, not the view.
- **Input:** `bevy_enhanced_input` actions for movement and every key in AGENTS.md's
  control map, with a separate input context for the rack screen (cursor free,
  movement off).
- **Colliders on placed things:** authored in Jackdaw with its Physics module
  (`[modules] physics = true` in `jackdaw/jackdaw.toml`; `AvianCollider` and
  `RigidBody::Static` on prefabs: oak trunks, the burbler's banks, stumps, modules).
  This is Jackdaw's documented workflow, not an editor change. Small things the player
  walks through (reeds, brush, debris, bugs) get no collider.
- **Debug camera:** Bevy's `free_camera`, toggled in dev builds.
- **Fidelity to today:** the same keys and feel. Speeds, eye heights and the existing
  `controls.rs` tests carry over as the acceptance bar.

Check each crate's Bevy compatibility again before adding it, and pin exact minor
versions (AGENTS.md).

## Jackdaw development compatibility check (2026-11-01)

The versions above support the standalone game's **released Bevy 0.20.1**.
They do **not** establish compatibility with the editor adapter's pinned Bevy
main (`c9c397ba1a73e6cfffe957318e0359330b701865`). The shared Jackdaw checkout
at `e73b8d35` explicitly declares Physics unavailable in
`crates/jackdaw_project_build/src/modules.json` and in the physics integration's
package metadata: `avian has no release for Bevy main`.

Verified with the documented, read-only preview:

```sh
jd import jackdaw --modules physics
# jd import: module `physics` is not compatible: avian has no release for Bevy main
```

Avian's [official manifest](https://github.com/avianphysics/avian/blob/main/crates/avian3d/Cargo.toml)
uses released `bevy`, `bevy_curve`, `bevy_math` and `bevy_shape` 0.20 dependencies.
Adding it directly to the adapter would introduce a second, incompatible Bevy
rather than enable the editor's collider components. Task 4.2d remains open;
`physics = true` is deliberately not written into a manifest the shared editor
cannot build. No private collider types, replacement physics, or editor fork is
introduced to bypass this check.

Resume when the **shared** Jackdaw Physics module supports its pinned Bevy:
run the preview again, apply the documented module integration, verify that the
game has one Bevy and one Avian dependency graph, then test
`AvianColliderBridgePlugin` with a static prefab collider. Claude can then author
the shapes already specified in each family's `colliders.json`. The terrain,
Tnua controller and enhanced-input migrations remain tasks 4.2a–c.

### After version normalization

`normalize-jackdaw-versions` removes the released-Bevy/adapter split: both game
entry points now follow the shared Jackdaw revision in one workspace and lockfile.
The released-Bevy versions in the research table are historical compatibility
information, not permission to upgrade Bevy independently. Physics remains gated
by the selected shared editor's module support. See `jackdaw-version-policy.md`.

### Supported-line alignment checkpoint

The game now follows the shared dev/0.20 branch (upstream release-0.20), rather
than bleeding-edge Bevy main. Its exact Jackdaw commit is recorded in
`jackdaw/compatibility.json`; both game entry points use its Bevy 0.20.0 and Rust
1.99.0. That line has Physics support. Adding the actual runtime plugin and game
colliders remains a separate task; no physics implementation is claimed here.
