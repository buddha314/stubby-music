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

The test builds an OMG path frame from a two-source simulation, constructs an actual
Firewheel graph with two procedural square-wave sources feeding one OMG node, and
renders 256 128-frame blocks at 48 kHz. Both renderers add into one Ambisonic bus and
one point-stereo pair; the node decodes once after summing. It checks finite, nonzero
output and guards each Firewheel callback with `assert_no_alloc`. OMG path setup occurs
while Firewheel constructs the processors, outside the callback.

With a closed rock room and two sources, the local debug test measured p50 1.54 ms,
p95 1.95 ms, and max 2.47 ms per 128 frames (2.67 ms callback period). This is near
the callback budget and needs release-profile/reference-machine measurements. The
earlier free-field one-source baseline was 159.6 / 213.9 / 298.8 µs (p50 / p95 / max).

Current limits: static `ParamBlock`, the speaker fallback (no HRTF asset), and no
Bevy/Avian geometry adapter or live parameter replacement. OMG's `set_params` allocates
temporary vectors, and replacing owned tap data in the callback could free memory;
updates need a bounded handoff. Geometry conversion, moving sources/listeners, and
in-game routing remain before judging production fit.
