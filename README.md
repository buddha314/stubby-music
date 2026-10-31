# stubby-music

A modular synthesis factory game. The first scene is a silent, seeded desert mudflat.

Run the visual scene:

```sh
cargo run -p stubby_game -- --seed=42
```

WASD moves, hold right mouse to look, Q/E lowers/raises the inspection camera,
and Shift moves faster. The camera starts at eye height. Use another seed to
regenerate the mud plates, fine crack texture, and mountain silhouettes.
For faster development linking use `--features dev`; omit it in release builds.

The floor uses clipped Voronoi polygons with raised rims, recessed sidewalls,
subtle height variation, and a generated repeating fine-crack/grain albedo and normal textures on a
rough PBR material. No reference photograph or external textures are required.
The HDR sky is generated as a linear half-float cubemap with an amber low sun,
dusty horizon and slate zenith; the cubemap also supplies filtered environment lighting, alongside matching sunlight.

This is the initial art-study scene: a fixed terrain area roughly 216 m wide,
free-flight inspection, haze and distant mountains. Streaming, sandstorm particles,
mood transitions, discovery objects and audio are tracked in the opening proposal
and are not implemented yet.

To save a preview after shader warmup, append
`--screenshot=/tmp/stubby-desert-preview.png` to the game arguments.

Open the level in the shared Jackdaw editor with `tools/jackdaw.sh open`.
First-time setup and Play instructions are in [jackdaw/README.md](jackdaw/README.md).

The audio engine is `crates/stubby_audio`. It has no Bevy dependency, and the game
does not use it yet. To hear it on the default output device:

```sh
cargo run --release -p stubby_audio --example play
```
