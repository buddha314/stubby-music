## Context

`ClockProc` wraps a `PeriodicClock` whose position starts at 0 when the processor is built and advances one sample at a time. Firewheel gives every block `info.clock_samples`, the engine's sample counter, which is also what `bevy_stubby_audio::MasterClock` reads (1 BPM = one beat per `60 * sample_rate` samples).

## Decisions

**D1. Derive, don't accumulate.** For multiple `m` and sample `s`, the master-locked clock's beat index is `floor(s * m / (60 * rate))` and its phase `(s * m mod 60 * rate) / (60 * rate)`, computed in `u128`. Nothing accumulates, so nothing drifts, and every clock's index advances at `s = k * 60 * rate` for every master beat `k`.

**D2. Tick on wrap.** A tick is emitted at the first sample of each new beat: when the phase wraps at an unchanged multiple. A tempo change moves the phase to the new multiple's position without a tick, so turning the knob never double-fires.

**D3. Whole tempos.** `ParamSpec` gains `whole: bool`; the clock's `bpm` is whole (1..=999). The engine rounds whole parameters it receives. The rack's BPM knob already rounds `bpm` (every clock's tempo parameter has that name), so the panel display and the engine agree; threading `whole` into the panel profiles was rejected as churn with no behaviour change.

**D4. Gate.** Unchanged: high for the first half of each beat.

**Rejected:** a separate master node that clocks follow by message (adds a dependency between nodes and a block of latency); resetting clocks at patch time (clocks would again disagree).

## Risks

A newly patched clock starts mid-beat. That is the point (shared downbeats), but the burbler's sound may wait up to a beat after the last cable.
