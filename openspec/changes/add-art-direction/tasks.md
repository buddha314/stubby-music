All tasks here are art (Claude) unless marked. Game code that reads these assets is
tracked in the owning change.

## 1. Style guide and lighting (this PR)

- [x] 1.1 `docs/art/style-guide.md` with theme, pillars, palette, legibility, materials, naming, pipeline, budgets, checklist, changelog (claimed: claude)
- [x] 1.2 `assets/lighting/lighting_keys.json`: bleak and bloomed × dawn, day, dusk, night (claimed: claude)
- [x] 1.3 `tools/blender/render_style_sheet.py` and `docs/art/style_sheet.png`; `docs/art/palette.png` (claimed: claude)
- [ ] 1.4 Check the keys in the game once its reader lands (`add-desert-opening` 3.4), and retune where Bevy and Cycles disagree

## 2. Asset backlog

- [x] 2.1 Scrub-oak tree for `add-scrub-oak-production` (`tools/blender/create_scrub_oak.py`, `assets/flora/`; claimed: claude): dormant (winter) → full bloom on one growth value driven by clean water; six seeded variants plus per-instance variation; nodes for bug emission points, parts the game can sway or pulse to music, far LOD
- [ ] 2.2 Bloom flora set (replaces the style sheet's stand-in stalks): small plants in the accent palette, music-reactive, lunarpunk-luminous at night
- [x] 2.2a Stream as staff music: a five-line staff along the water carrying notes downstream as musical nutrients; dry and broken when bleak, flowing and luminous when bloomed; game-drivable note flow (speed, colour, density) so the game can match the music carried — done as the stream segment: staff lines on the bed, revealed as the water clears (`add-mudpit-burbler` 1.7) (claimed: claude)
- [ ] 2.3 First animal concept: a creature that grows from music, reads as bleak then bloomed
- [x] 2.4 Patch cable and jack plug: plug matching the module jacks; the cable drawn by the game as a fully elastic VCV Rack-style curve (`tools/blender/create_patch_cable.py`, `assets/cables/`) (claimed: claude)
- [ ] 2.5 Envelope, VCA and Constant module assets on `module_kit`
- [x] 2.6 Desert props for the bleak opening: dead brush, bone-dry stumps, cracked-mud debris; each with a bloomed counterpart (`tools/blender/create_desert_props.py`, `assets/props/`) (claimed: claude)
- [ ] 2.7 Add new flora and props to the style sheet as they land
- [x] 2.8 Migrate module lights (VCO LEDs, glyphs, clock lamp) from LED-like to jellyfish/fairy luminescence per the style guide
- [ ] 2.9 Mudpit burbler and its rack parts: tracked in `add-mudpit-burbler`
