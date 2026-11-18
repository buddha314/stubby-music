# OMG Audio + Firewheel live-signal spike

An isolated first-stage adapter experiment for OMG Audio at revision
`4f8ff13e48fb9e96a59b0b1a607b9d33ac01a869`. It is intentionally excluded from the
production workspace and pins upstream with a Git revision; do not add these direct
Git dependencies to the game workspace because `deny.toml` restricts dependency
sources.

Run it from the repository root:

```sh
cargo test --manifest-path spikes/spike_omg_audio/Cargo.toml
```

The test builds an OMG path frame from a one-source simulation, constructs an actual
Firewheel graph with a procedural square-wave source and the OMG node, and renders
sixteen 128-frame blocks at 48 kHz. It checks finite, nonzero output and guards each
Firewheel callback with `assert_no_alloc`. OMG path setup occurs while Firewheel
constructs the processor, outside the callback.

Current limits: one source, static `ParamBlock`, the speaker fallback (no HRTF asset),
and per-node decode of the diffuse Ambisonic bus. There is no Bevy/Avian geometry
adapter, live parameter replacement, shared multi-source decode, quality comparison,
or timing benchmark yet. These are required before judging production fit.
