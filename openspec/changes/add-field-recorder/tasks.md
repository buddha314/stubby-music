## 1. Art (Claude)
- [x] 1.1 Field recorder asset: body, screen with level bars, transport and record button, X/Y capsules in a guard, shell-fur windguard (14 alpha-clipped shells), record fairy light; profile and previews (`tools/blender/create_field_recorder.py`)

## 2. Game (Codex)
- [x] 2.1 H brings it up into the hand (lower right of view) and puts it away; the hand holds one item from a list (`hand.rs`); the `FieldRecorder_FurShell*` meshes get `NotShadowCaster` (built by Claude at the user's request)
- [ ] 2.2 While E collects: pulse `Light_rec` (profile `lights`) and light `Level_<n>` bars; rest dark otherwise
