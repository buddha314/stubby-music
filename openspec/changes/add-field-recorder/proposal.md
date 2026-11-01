## Why
Collecting needs a device in the player's hand. The world is about finding sound, so
the collector is a handheld stereo field recorder with a fur windguard: the tool a
location recordist carries, generic and unbranded.

## What Changes
- Art: `tools/blender/create_field_recorder.py` → `assets/models/equipment/field_recorder.glb`
  and `assets/equipment/field_recorder/` (profile, previews). Real size, origin at the grip;
  X/Y capsules under a shell-fur windguard; a record fairy light and a level-meter screen.
- Game: hold it in first person while playing, and use it for collection: the record light
  pulses and the meter moves while E collects.

## Capabilities
### New Capabilities
- `field-recorder`: the held collecting device.

## Impact
New `equipment` asset category; first-person held-item display and collection feedback in
`stubby_game` (Codex).
