# Lightning bug

The first creature asset is a white flash with an insect silhouette. Its body,
wings and light are completely hidden between pulses. No idle glow or continuous sound is
emitted. The flash attacks in 15 ms and decays to zero over a total of 180 ms.

- Source: `assets-src/blender/lightning_bug.blend` (metres, isolated asset scene).
- Runtime: `assets/models/lightning_bug.glb`, with a `white_flash` scale animation.
- Profile: `asset.json`. The game embeds this profile at build time.
- Preview: `preview.png` shows the flash at its peak; scrub frames 1–12 in Blender
  to see it collapse to invisibility at 60 FPS.

Rebuild from the repository root:

```sh
~/.local/bin/blender -b --factory-startup -P tools/blender/create_lightning_bug.py
```

Or export an edited source:

```sh
~/.local/bin/blender -b assets-src/blender/lightning_bug.blend -P tools/blender/export_gltf.py
```

The opening includes three lightning bugs, each with its own RandomClock (IDs 1–3). `stubby_audio` runs
its seeded `RandomClock` on the audio callback, selects each interval in
0.65–2.4 seconds using a seed derived from the world seed and bug ID, and emits sample-stamped `ClockTick` events. Its gate drives a short click through existing
square-VCO, envelope and VCA modules, rather than being sent directly to output. The first flash follows a resting interval.

The game broadcasts `SamplePulse` messages and exposes `PulseBus::level`; other
assets can use `PulseFollower.clock` to select the same timeline. Followers share
the predicted tick instant, not an independent timer. The authored animation is
for asset preview; the game drives visibility, scale and local light from the
sample-clock envelope, preserving synchronization despite frame jitter. If the
audio device cannot open, bugs remain hidden and a diagnostic explains why.

Run the scene and optionally capture both real pulse states:

```sh
cargo run -p stubby_game -- --seed=42 --pulse-preview=/tmp/lightning-bug
```

The isolated Jackdaw adapter shares these source files and models. Its
`assets/models` symlink points to the game's runtime models. In the editor, Play
shows the flashes; the edit viewport remains the static ground proxy.

`PulseClock` is the common clock trait, with `PeriodicClock` and `RandomClock` implementations. Each bug owns its random state; reusing a follower clock ID is the explicit way to synchronize another asset to that bug.

## Resource yield

Lightning bugs provide the `square_wave` recipe ingredient. Approach within
3 metres and press E to recover one pattern from that world instance. The
inventory counter updates; that instance cannot yield again. This does not alter
the bug's flash or automatically create an audible oscillator. Resource and
recipe definitions live in `assets/recipes/catalog.json`.

A nearby bug makes a brief 2200 Hz square-wave click (0.5 ms attack / 4 ms decay)
using the existing fundsp-backed oscillator and Firewheel graph modules. Gain
falls to zero at 3 metres, so the initial spawn remains silent. In the recipe
book this resource is **Square wave**, reusable for VCO, LFO and gate roles; it
is not a prebuilt oscillator module.
