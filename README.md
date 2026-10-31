# stubby-music

A modular synthesis factory game. The first scene is a silent, seeded desert mudflat.

Run the visual scene:

```sh
cargo run -p stubby_game -- --seed=42
```

WASD walks, the mouse looks freely, Space jumps, C crouches,
and hold Left Shift to sprint. Escape releases the cursor; click to resume.
The player starts standing on the terrain at eye height. Use another seed to
regenerate the mud plates, fine crack texture, and mountain silhouettes.
For faster development linking use `--features dev`; omit it in release builds.

The floor uses clipped Voronoi polygons with raised rims, recessed sidewalls,
subtle height variation, and a generated repeating fine-crack/grain albedo and normal textures on a
rough PBR material. No reference photograph or external textures are required.
The HDR sky is generated as a linear half-float cubemap with an amber low sun,
dusty horizon and slate zenith; the cubemap also supplies filtered environment lighting, alongside matching sunlight.

This is the initial art-study scene: a fixed terrain area roughly 216 m wide,
grounded first-person movement, haze and distant mountains. Streaming, sandstorm particles,
mood transitions, discovery objects and spatial audio are tracked in the opening proposal
and are not implemented yet.

To save a preview after shader warmup, append
`--screenshot=/tmp/stubby-desert-preview.png` to the game arguments.

Open the level in the shared Jackdaw editor with `tools/jackdaw.sh open`.
First-time setup and Play instructions are in [jackdaw/README.md](jackdaw/README.md).

The audio engine is `crates/stubby_audio`; it has no Bevy dependency. One shared
audio thread drives the lightning bugs’ clocks and short click patches. The opening
is silent until a bug is nearby. For a test patch, add `--audio-demo` or press F8.
To hear the engine alone on the default output device:

```sh
cargo run --release -p stubby_audio --example play
```

The first creature asset is the [lightning bug](assets/creatures/lightning_bug/README.md).
Each bug in the opening owns a seeded random audio clock and emits
white flashes independently, remaining invisible and silent between pulses. Nearby
clicks use the existing square VCO, envelope and VCA; spawn is outside hearing
range. An audio output device is required for the live clock.

Lightning bugs are also resource nodes: approach within 3 metres and press E to
recover a square-wave recipe ingredient. The [recipe foundation](crates/stubby_recipes/README.md)
provides validated data definitions, inventory, one-time resource yields and
atomic crafting. Press O for the recipe book: Square wave is a raw building block
for VCO, LFO and gate uses. Other waveform resources and production combinations
will be authored later.
