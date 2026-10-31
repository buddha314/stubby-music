## Context

Two agents now work in parallel: Claude makes assets in Blender and owns the look;
Codex writes services and game logic. The look must therefore cross the boundary as
data and models, not as code either side has to reach into.

## Decisions

**Lighting is data, authored by art.** Every lighting value the game uses for the day
and the bloom lives in `assets/lighting/lighting_keys.json`, in sRGB, with a
`version`. The game interpolates; it does not choose colours. Changing the look is a
data change that needs no Rust.

**Two axes, eight keys.** Time of day is global. Light quality is local (10 m cubic
cells, chiefly plant and animal density, which grows from music). The keys sit at
the corners: {bleak, bloomed} × {dawn, day, dusk, night}. Four times are the fewest
that give a believable sunrise and sunset; two moods keep authoring tractable. More
keys can be added later without changing the format's shape.

**The style sheet is the review.** Assets and keys are judged together: every module
under every key, backlit, the hardest case for legibility. A key change or a new asset
is not done until its tiles have been looked at.

**Generators, not hand-edited files.** Every asset is produced by a headless Blender
script, so it can be regenerated when the style moves, and the style will move.

## Risks / Trade-offs

- [The Cycles style sheet and Bevy will not match exactly] → the sheet judges
  relationships (legibility, contrast, palette), not absolute values; check in the
  game once Codex's keys reader lands.
- [Lux values are tuned by eye, not measured] → acceptable for a stylised game; the
  exposure per key absorbs it.
