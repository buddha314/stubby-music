# Modules

Synth modules as world-scale Eurorack modules: 3U panels, ×10 scale (1.285 m tall
before anything on top). Every module comes from a generator script in
`tools/blender/`, which is the source of truth. The scripts share
`tools/blender/module_kit.py` (materials, case, knobs, jacks, labels, glyph strokes,
preview and export). It writes the `.blend` to
`assets-src/blender/`, the `.glb` to `assets/models/`, and `asset.json`,
`preview.png` and `preview_far.png` (the same view from about 12 m) here.

## Clock (`tools/blender/create_clock_lamp.py`)

| Close | About 12 m |
| --- | --- |
| ![clock](clock_lamp/preview.png) | ![clock from 12 m](clock_lamp/preview_far.png) |

The opening's second discovery, and the clock that manifests as a pulsating light.
Instead of a waveform crown it carries a caged glass beacon (1.685 m tall overall),
a silhouette nothing else has and a large emissive volume that can pulse visibly
through haze. The faceplate follows the VCO family: a narrow tick-train glyph
(deliberately unlike the square's 50% wave) on the front and both sides, a BPM knob,
and GATE and PHASE outputs. The engine's Clock has no inputs.

The `Lamp` node carries the flash the game drives from the clock's sample-stamped
ticks, using the same envelope fields as the lightning bug (`attack_seconds`,
`flash_seconds`, `rest_strength`, `peak_strength`, `light_lumens`,
`light_range_metres`, `pulse_driver`). The previews show it at the peak of a flash.

```sh
~/.local/bin/blender -b --factory-startup -P tools/blender/create_clock_lamp.py
```

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
| `Lamp` (Clock only) | `light`, `pulse_driver`, flash envelope | The beacon; flash its emissive strength and a point light on each tick |

Origin at the base centre; the faceplate faces glTF +Z. Tilting a module or sinking it
into the mud happens at placement (`placement` in `asset.json`), so one asset serves
every pose. `crates/stubby_game/tests/module_assets.rs` checks these nodes in every
exported module. The Clock's equivalents are `Port_gate`, `Port_phase`, `Knob_bpm` and
`Light_gate`.
