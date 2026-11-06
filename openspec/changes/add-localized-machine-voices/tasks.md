## 1. Engine

- [x] 1.1 `Command::SetEmitter` and a spatial node per placed master output, removed with its module (claimed: claude)
- [x] 1.2 Offline test: pans toward the emitter's side, silent at volume 0, tolerates a late update (claimed: claude)

## 2. Game

- [x] 2.1 `hearing_radius_metres` in the rack `configure` block and in `create_burbler.py` (claimed: claude)
- [x] 2.2 `rack.rs` exposes the voice's output; streams remember their source burbler through a chain (claimed: claude)
- [x] 2.3 `place_voices`: nearest point of machine or streams, offset in camera space, hearing-radius fade (claimed: claude)

## 3. By ear

- [ ] 3.1 Patch a burbler in Play and confirm direction and range by ear; tune `hearing_radius_metres`
- [ ] 3.2 Walk the length of a stream and confirm the voice follows it
