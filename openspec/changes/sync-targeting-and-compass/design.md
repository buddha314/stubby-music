## Context

Synchronize pickup outlines and compass guidance. Source: `add-collectible-outline and add-compass-bar`. Source task completion is evidence of reported status, not a fresh runtime certification. The audit records conflicts and deferred work.

## Decisions

Retain the existing engine boundaries and native asset layout. Extend the current owning capability, keeping this change as the cross-engine acceptance contract. Directly copying Godot implementation was rejected because it changes engine hosting, import semantics and often signal conventions. A second inventory, goal engine or DSP graph was rejected because the repository already owns these systems.

## Dependencies and order

Asset contracts precede new shell geometry and grown-state review. Stream growth precedes tree_grown milestone integration; goal schema precedes rows. Target ranking precedes outline presentation. Calendar and sky use simulation time, while sequencing, stream notes and musical lights use predicted heard sample timestamps. Coordinate overlapping tasks with existing owners; no implementation task is claimed by this docs change.

## Risks and validation

Existing saves and IDs need explicit compatibility decisions, generated schema must come from rules-fmt, and imported assets require licence/provenance checks. Use focused offline/domain tests and prefab validation. Any graphical validation must build stubby_jackdaw first and use tools/jackdaw_timed_test.py with its independent 20-second watchdog. This audit runs no graphical diagnostics.
