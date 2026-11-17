## 1. Art (Claude)
- [x] 1.1 Field recorder asset: body, screen with level bars, transport and record button, X/Y capsules in a guard, shell-fur windguard (14 alpha-clipped shells), record fairy light; profile and previews (`tools/blender/create_field_recorder.py`)

## 2. Game (Codex)
- [x] 2.1 H brings it up into the hand (lower right of view) and puts it away; the hand holds one item from a list (`hand.rs`); the `FieldRecorder_FurShell*` meshes get `NotShadowCaster` (built by Claude at the user's request)
- [x] 2.2 `Light_rec` answers each successful sound capture (attack/decay envelope); the `Level_<n>` bars meter the world bus, dark in silence (`stubby_player::recorder`, sync-recorder-feedback; the bars follow real sound rather than a collection animation, as decided in the Godot game)
