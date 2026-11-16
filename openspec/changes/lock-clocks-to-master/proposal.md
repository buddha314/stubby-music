## Why

The user decided (2026-11-16) on a core 1 BPM master clock that times all change, as the Godot game decided on 2026-11-13. The day cycle already runs on it (`sync-calendar-and-night-sky`, D12), but every tempo clock in the engine still runs freely from the moment it is added: two burblers at the same BPM drift apart by when they were patched, and no two clocks share a downbeat.

## What Changes

- An engine `Clock` derives its beat and phase from the engine's sample counter as a **whole multiple of the 1 BPM master**: a 116 BPM clock is ×116. Its beats are exact (integer arithmetic, no drift), and every clock lands together on each master beat, once a minute.
- Tempo is a **whole number of BPM**. The engine declares the clock's `bpm` parameter whole; the rack's BPM knob steps in whole numbers and its display shows them.
- Changing a clock's tempo keeps it on the master grid; it jumps to where the new multiple is now, with no extra tick.
- Seeded random clocks (lightning bugs) stay free-running, as in the Godot game.

## Capabilities

### New Capabilities
- `master-clock`: the 1 BPM master clock and how tempo clocks lock to it.

### Modified Capabilities
None in accepted specs.

## Impact

- `stubby_audio` (core): `clock.rs` (a master-locked clock), `modules.rs` (`ClockProc`, `ParamSpec::whole`).
- `bevy_stubby_rack`: the BPM knob's whole-number stepping, now documented as this contract.
- `bevy_stubby_audio`: `MasterClock` constants already exist (#143); unchanged.
- Newly patched clocks start mid-beat, at the master grid's position, instead of on a beat. The burbler's first tick therefore waits at most one beat (≤1.5 s at 40 BPM).
