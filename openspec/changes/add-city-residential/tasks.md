Art and asset pipeline. Source of truth: `docs/art/city-residential.md`.

## 1. Discovery

- [x] 1.1 (claimed: Claude) Save the brief (`docs/art/prompts/city-residential.md`) and the references; audit the City kit and write the delta manifest.

## 2. The validation bay

- [x] 2.1 (claimed: Claude) Build the pieces one mixed-use bay needs and validate their glTF (acoustic extras on every material).
- [x] 2.2 (claimed: Claude; `create_city_mixed_bay.py`) Assemble the bay; check the stack snaps (bands, slabs, corners, balconies, stair tower); render day, dusk, night and the furnished interior.
- [x] 2.3 (claimed: Claude; 2026-11-28, `--asset-review city_mixed_bay`, 20 s watchdog, no errors) Load the bay in the game.

## 3. The kit

- [x] 3.1 (claimed: Claude) Retail and office frontages, residential facades A and B, rounded corners, balcony kit, dwelling shell.
- [x] 3.2 (claimed: Claude) Domestic furniture and fabric, access core, communal roof pieces, interior and distant proxies.

## 4. The neighbourhood

- [x] 4.1 (claimed: Claude) Street wall, rounded corner block, stepped garden tower and taller tower from one generator; furnished homes; inland street and courtyards; skyline.
- [x] 4.2 (claimed: Claude) Assembly test row (every bay type on one grid) and a small mixed block with both corner types.
- [x] 4.3 (claimed: Claude) Residential review cameras (street looking up, canal stack, balcony, overview, corner) in day, dusk and night.
- [x] 4.4 (claimed: Claude; 2026-11-28, `breq`, vsync on, temporary diagnostics plugins as `docs/research/performance-profiling.md` describes) Measure in the game: homestead review ~3,900 entities at 60 fps; the validation bay ~6,900 at 60 fps; the full district ~82,000 at 30-35 fps with dips to 7-10. One building is free; the whole district as one scene is not (`add-city-kit` 5.5).
