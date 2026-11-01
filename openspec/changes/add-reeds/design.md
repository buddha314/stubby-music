## Decisions
- **The stream's ingredient.** One stream segment (4 m) costs **6 reeds**. Laying a
  planned stream spends 6 reeds per segment; the hologram shows invalid when the player
  cannot afford the plan. The laid segment is outlined by reeds on both banks
  (`Reeds_L`, `Reeds_R` in `stream_segment`).
- **Very easy to get, like leaves in Satisfactory.** Reeds must never be the bottleneck
  for building streams. Clumps are abundant wherever there is wet ground, yield
  generously, return quickly, and holding E collects successive nearby clumps.
- **Frequent renewable.** A reed clump is a finite resource instance with a short
  elapsed-time replenishment policy (`add-resource-replenishment`): harvest takes the
  whole clump, it shows stubble, and it returns standing when the interval passes.
  It is not a mine and has no production rate.
- **Common.** Clumps are placed far more often than lightning bugs, so the asset is
  cheap: about 1.4k–1.7k triangles near and a 6-triangle far LOD of crossed strips.
- **Wet ground.** Clumps grow where there is water or mud: burbler pools, stream banks
  and damp hollows. Exact placement rules belong to world generation.
- **Varied procedurally**, like the scrub oak: four seeded variants, plus per-instance
  scale, height, yaw, sway phase and greenness from the world seed and instance ID.
- **Dry to green.** One greenness value (0 dry, 1 green) recolours stalks and blades.
  It follows local water or light quality; which one is the game's call.
- **Regrowth is visible.** Between harvest and return the game may scale `Standing`
  up from stubble height, so a clump can be seen coming back.

## Open questions
- Yield per harvest and the return interval. Suggested starting point: 6 reeds per
  clump (one segment's worth), returning after 60 s of game time; tune in play so a
  few minutes of gathering builds a long stream.
- Whether reeds feed any other recipe.
- Whether reeds near cleaner water yield more or return faster.

## Asset contract
See `assets/flora/README.md` and `assets/flora/reeds/asset.json`: `Interact`,
`Standing` with `Sway_<n>`, `Stubble`, `LOD_Far`, the states table and greenness colours.
