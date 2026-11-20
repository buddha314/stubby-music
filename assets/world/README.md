# World: the seeded opening

`opening.json` is the world configuration both game entry points start from
(`stubby_world::world_generation::WorldConfig::shipped`; `--world-config=<file>` runs
another). The layout it describes is generated from the world seed when the scene spawns
(`crates/stubby_jackdaw/src/seeded_layout.rs`), around the authored objects in
`assets/scene.bsn`, which it never moves (`add-hybrid-world-placement`).

- `seed`: the default world seed (`--seed=<n>` overrides it). Tests check the opening's
  constraints on seed 17.
- `scatter`: the rules, resolved **in file order**; each may keep distances from what is
  already there (authored objects and earlier rules' placements, matched by model stem):
  - `burblers`: 65 m apart, 16 m inside the ground;
  - `reed_beds`: 2.8 to 14 m from a burbler, densest at its pool;
  - `oak_groves`: 20 to 40 m from a burbler, 7 m apart;
  - `rock_formations`: 25 m from burblers, 30 m from the grotto's boulders, 30 m apart;
  - `desert_props`: brush, stumps and debris, 5 m from burblers.

  A rule's fields: `variants` (the models it picks from), `density` (per square metre),
  `resolution` (the grid it samples), `min_spacing`, `scale` (a range), `clearance` (its
  half extent, kept clear of everything placed before it) and `distances` (`stems`,
  `min_m`, `max_m`, `denser_nearer`). The region is the cracked ground, ±108 m.
- `stream_features`: things that form along every laid stream (`ripple_pools`: one per
  12 m on average, at seeded Poisson-random places per segment).
- `protected_areas`: kept clear (the spawn, and the view from it to the authored burbler).
- `authored_footprints`: how much room each authored kind keeps around itself.
- `edits`: a designer's suppressions and moves of generated objects, by their stable key.

Flags: `--no-seeded-layout` (only the authored scene), `--layout-review=<dir>` (an
overhead and a reed-bed screenshot, under `tools/jackdaw_timed_test.py --command`). The
editor's edit view does not show the seeded layout; Play does.
