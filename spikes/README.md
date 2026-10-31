# Audio engine spike

Throwaway code for `openspec/changes/add-audio-engine` task group 2. Do not build on
it; the decision and numbers are recorded in that change's `design.md` (D4).

- `spike_common`: the `Backend` trait both candidates implement, plus the offline
  measurement suite (tone, clock timing, click ratios on edits, audio-thread
  allocations via `assert_no_alloc`, cost at 201 modules).
- `spike_fundsp`: fundsp `Net` frontend/backend, with and without fades.
- `spike_firewheel`: Firewheel driven by hand, with a scalar sine and with fundsp's
  sine hosted in a Firewheel node.

Run from this directory; `spikes/` is its own Cargo workspace, separate from the game.

```bash
cd spikes
cargo run --release -p spike_fundsp           # offline report
cargo run --release -p spike_firewheel        # offline report
cargo run --release -p spike_firewheel -- live   # 8 s listening test on the default device
cargo run --release -p spike_fundsp -- live
```

Timings swing about 2× between runs on a busy machine; compare variants within a
single run.
