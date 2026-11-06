## Why

A patched burbler plays at the same level in both ears wherever the player stands, so its sound fills the world and cannot be found by ear. Finding things by ear is how the opening works. The full spatial plan (`add-rack-and-spatial-audio`) waits on a rack engine that does not exist yet, so machine voices need a smaller step now.

## What Changes

- `stubby_audio` gains `Command::SetEmitter`, which places an output cabled to the master in the world, using Firewheel's basic spatial node.
- A rack's voice is heard from the nearest point of its machine or the streams laid from it, panned relative to the camera.
- A hearing radius in the machine's `configure` block fades the voice to silence, on the lightning bug click's curve.

## Capabilities

### New Capabilities

- `machine-voice-placement`: where a machine's voice is heard from, and how far it carries.

### Modified Capabilities

None.

## Impact

- **`crates/stubby_audio`:** a new command, the `spatial_basic_node` Firewheel feature, and an offline stereo test.
- **`crates/stubby_game`:** `rack.rs` exposes the voice's output; `streams.rs` places it each frame.
- **`assets/machines/mudpit_burbler/asset.json`, `tools/blender/create_burbler.py`:** `hearing_radius_metres`.
- **Not changed:** lightning bug clicks keep their distance-only fade.
