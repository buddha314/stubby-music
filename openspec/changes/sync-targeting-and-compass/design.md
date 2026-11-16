## Context

Synchronize pickup outlines and compass guidance. Source: `add-collectible-outline and add-compass-bar`. Source task completion is evidence of reported status, not a fresh runtime certification. The audit records conflicts and deferred work.

## Decisions

Retain the existing engine boundaries and native asset layout. Extend the current owning capability, keeping this change as the cross-engine acceptance contract. Directly copying Godot implementation was rejected because it changes engine hosting, import semantics and often signal conventions. A second inventory, goal engine or DSP graph was rejected because the repository already owns these systems.

## Dependencies and order

Asset contracts precede new shell geometry and grown-state review. Stream growth precedes tree_grown milestone integration; goal schema precedes rows. Target ranking precedes outline presentation. Calendar and sky use simulation time, while sequencing, stream notes and musical lights use predicted heard sample timestamps. Coordinate overlapping tasks with existing owners; no implementation task is claimed by this docs change.

## Risks and validation

Existing saves and IDs need explicit compatibility decisions, generated schema must come from rules-fmt, and imported assets require licence/provenance checks. Use focused offline/domain tests and prefab validation. Any graphical validation must build stubby_jackdaw first and use tools/jackdaw_timed_test.py with its independent 20-second watchdog. This audit runs no graphical diagnostics.

## Outline spike (2026-11-16)

`bevy_mod_outline` 0.14.0 (MIT OR Apache-2.0), on Bevy main through a generated fork
(`bevy_main_forks.json`), drawn on the targeted pickup's root with `PropagateOutline`, so
every mesh and LOD below it is outlined. Mean frame rate over the prefab review (RX 7700S,
embedded Play): no outline 56.7 fps; jump-flood mode 35–43 fps (and slower start-up); vertex
extrusion with `AutoGenerateOutlineNormalsPlugin` 47.7 fps; **vertex extrusion without the
normals plugin and without the crate's default features 56.9 fps**, chosen. Without outline
normals a flat reed blade can be edged on one side; acceptable, revisit if shells show gaps.

The compass is top-level UI nodes scrolled with `UiTransform`. Every `Node` carries a
transparent `BackgroundColor`; fading it on a label draws a dark box, so labels fade their
text and only ticks and the caret fade their fill.

Open: a targeted bug is invisible between flashes (its visual scales to zero), so it shows
no outline then; the spec's dark silhouette needs a separate body. Opening facing east
(the Godot game rotated its level) would move every placement and review camera; not done.
