# Modules

Synth modules as world-scale Eurorack modules: 3U panels, ×10 scale (1.285 m tall,
1.375 m with the crown). Every module comes from a generator script in
`tools/blender/`, which is the source of truth. It writes the `.blend` to
`assets-src/blender/`, the `.glb` to `assets/models/`, and `asset.json`,
`preview.png` and `preview_far.png` (the same view from about 12 m) here.

## VCOs (`tools/blender/create_vco.py`)

| Square | Saw | Triangle | Sine |
| --- | --- | --- | --- |
| ![square](square_vco/preview.png) | ![saw](saw_vco/preview.png) | ![triangle](triangle_vco/preview.png) | ![sine](sine_vco/preview.png) |

8HP oscillators: one FREQ knob, a V/OCT input, and an OUT output on the dark patch
outputs conventionally sit on. Each waveform must read from a distance, so it appears
at three scales:

- **Silhouette:** the case's crown *is* the waveform, traced in amber along its front
  edge. Silhouettes survive haze, where detail does not.
- **Faceplate:** a large amber glyph on a dark recessed plate.
- **Sides:** the same glyph on both sides (mirrored on the left, so it reads correctly
  from outside), so it reads from any angle.

Square is the opening's first discovery. The engine has `SineVco` and `SquareVco`
today; saw and triangle are ready for when it gains those oscillators.

```sh
~/.local/bin/blender -b --factory-startup -P tools/blender/create_vco.py           # all four
~/.local/bin/blender -b --factory-startup -P tools/blender/create_vco.py -- saw    # one
```

The script only creates its own scene, so it is also safe to run inside an open
Blender (for example through MCP); there it writes only the `.blend`.

### Game-facing nodes

Named empties carry glTF extras the game reads (`<Prefix>` is `SquareVco`, `SawVco`,
`TriangleVco` or `SineVco`):

| Node | Extras | Use |
| ---- | ------ | --- |
| `<Prefix>_Root` | `asset_id`, `module_kind`, `waveform`, `scale` | Identify the module |
| `Port_voct`, `Port_out` | `port`, `direction` | Cable sockets; local +Z points out of the panel |
| `Knob_freq` | `param`, `param_min`, `param_max`, `taper`, `rotation_degrees` | Rotate about local Z to show the parameter |
| `Light_out` | `light`, `rest_strength`, `peak_strength` | Drive the LED's emissive strength from the rack's light value |

Origin at the base centre; the faceplate faces glTF +Z. Tilting a module or sinking it
into the mud happens at placement (`placement` in `asset.json`), so one asset serves
every pose. `crates/stubby_game/tests/module_assets.rs` checks these nodes in every
exported VCO.
