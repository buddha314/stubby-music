## 1. Art

- [ ] 1.1 Add `clearing` dawn, day, dusk and night keys and the third `light_quality` stop; bump `version`
- [ ] 1.2 Render three rows in `render_style_sheet.py` and re-render; check amber legibility in all `clearing` tiles
- [ ] 1.3 Compare dusk with the local reference (`docs/art/references/clearing_dusk.png`) and with the sampled palette
- [ ] 1.4 Update the style guide: three stages, what each means, and the `clearing` entry in the palette

## 2. Decisions

- [ ] 2.1 What drives the stage: plant density alone, or something more direct such as the storm ending
- [ ] 2.2 Whether `bloomed` dusk changes once all three rows are visible together

## 3. Game (through `add-desert-opening`)

- [ ] 3.1 The keys reader (3.4) handles any number of moods and the listed stops
- [ ] 3.2 The blend (3.6) uses the stops either side of light quality; sweep light quality in a bounded review to check for jumps
