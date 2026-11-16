## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 `stubby_audio`: a master-locked clock (beat and phase from the sample counter, whole multiple, tick on wrap); `ClockProc` uses it; offline render test that two clocks share downbeats.
- [x] 1.2 `ParamSpec::whole`; the clock's `bpm` is whole; the engine rounds whole parameters.
- [x] 2.1 `bevy_stubby_rack`: the BPM knob steps in whole numbers (it already rounded `bpm`; every clock's tempo parameter is `bpm`, and the engine rounds whole parameters itself).
- [x] 3.1 Bounded graphical check: the burbler still sounds and pulses after patching.
