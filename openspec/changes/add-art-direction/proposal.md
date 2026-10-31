## Why

The game's look now has a theme and an arc: **the music of nature brings light to the
world**. The player grows plants and animals by playing music to them, and where life
grows the bleak, hazy desert blooms (streams, drawn as staff music, carry that music
to them as nutrients) into solarpunk by day and lunarpunk by night. Art
and game logic are now split between agents (Claude: Blender assets and the style
guide; Codex: services and game logic), so the look needs a written contract: a style
guide, art-authored lighting data the game reads, and a backlog of assets the game
work is waiting on.

## What Changes

- `docs/art/style-guide.md`: a living style guide with a changelog (theme, pillars,
  lighting axes, palette, scale, three-distance legibility, materials, naming,
  pipeline, budgets, review checklist), plus `palette.png`.
- `assets/lighting/lighting_keys.json`: eight lighting keys (bleak and bloomed ×
  dawn, day, dusk, night) that the game interpolates by time of day and blends by
  local light quality. Art-owned; game code reads it (`add-desert-opening` 3.4–3.7).
- `tools/blender/render_style_sheet.py`: renders every exported module under every
  key to `docs/art/style_sheet.png`, the review sheet for any asset or key change.
- An asset backlog owned by art, starting with the scrub-oak tree that
  `add-scrub-oak-production` needs, then flora, streams as staff music, and the first animal.

## Capabilities

### New Capabilities
- `art-direction`: the style guide, the lighting keys contract, the style sheet
  review, and how new assets are made and accepted.

### Modified Capabilities
<!-- none; the day/night cycle and light quality field are added to the atmosphere
spec in add-desert-opening, where the code that reads the keys is specified -->

## Impact

- New: `docs/art/`, `assets/lighting/`, `tools/blender/render_style_sheet.py`.
- `add-desert-opening`: the global `Mood` is replaced by a global day/night cycle and
  a local light quality field (10 m cells). Tasks 3.4–3.7 and 4.6 rewritten for Codex.
- `AGENTS.md`: theme, look and feel, and the Claude/Codex ownership split.
