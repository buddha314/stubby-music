# Props

## Boulders (`tools/blender/create_boulders.py`)

Rounded, weathered granite after the Mojave's (Joshua Tree), from Blender's Rock Generator
add-on at fixed seeds (`add-echo-grotto`). Five variants make the echo grotto, a cluster
in the opening scene with a small grotto at its base; their near meshes are acoustic rock
(glTF extras `"acoustic": "rock"`). CC0: generated geometry.

![grotto](../../previews/props/rocks/grotto.png)
![inside](../../previews/props/rocks/grotto_inside.png)

| Variant | Role | Size (m) |
| ------- | ---- | -------- |
| `boulder_0` | back | 9 × 7 × 6.5 |
| `boulder_1` | left | 7 × 6 × 5.5 |
| `boulder_2` | right | 6.5 × 6 × 5 |
| `boulder_3` | the leaning top, roofing the alcove | 8 × 6 × 3.6 |
| `boulder_4` | a small rock in front | 3 × 2.5 × 2 |

## Rock formations (`tools/blender/create_rock_formations.py`)

![rock formations](../../previews/props/rocks/rock_formation_5/preview.png)

The boulders' granite as whole clusters, placed by the seeded opening
(`assets/world/opening.json`, rule `rock_formations`): `rock_formation_0` a pile,
`_1` a scatter, `_2` a window rock (a slab bored through), `_3` a slot between two fins,
`_4` a bridge (a capstone on two supports) and `_5` a tor (a bored boulder and a slot).

- Each opening is an `Opening_<k>` socket with extras `resource: rock_opening`, so it
  gives a high-pass filter (`add-high-pass-sources`); passages run along the model's Z.
- The near rocks (`rock_formation_<k>_LOD0`) are acoustic rock and become the formation's
  solid colliders, so the openings stay walkable; `colliders.json` lists per-rock boxes.
- The Rock Generator is a user extension: run Blender without `--factory-startup`, after
  `blender -c extension install -s extra_mesh_objects` once per Blender version.

```sh
~/.local/bin/blender -b -P tools/blender/create_rock_formations.py -- 2    # one formation
```

## Desert props (`tools/blender/create_desert_props.py`)

The bleak opening's scatter, each with its bloomed counterpart (`add-art-direction` 2.6).

![bleak](../../previews/props/desert/sheet_bleak.png)
![bloomed](../../previews/props/desert/sheet_bloomed.png)

| Family | Bleak | Bloomed | Triangles |
| ------ | ----- | ------- | --------- |
| `desert_brush_<k>` | low grey twiggy dead shrub | small leaf clusters, a few luminous magenta flowers | 2.9k–4.4k |
| `desert_stump_<k>` | bleached snag, jagged break, flared roots | wood darkens with damp, moss, luminous cyan mushrooms at its foot | 1.3k–1.7k |
| `desert_debris_<k>` | scatter of curled, broken mud plates | sprouts pushing up between the plates | 0.2k–0.3k |

**Bloom** (0 bleak .. 1 bloomed) follows the local light quality, the field that blends
the world from bleak to bloomed. Each `Bloom_<n>` node scales in over its own slice
of the range (extras `from`, `to`), so a prop blooms gradually; `DesertBrush_Twig`,
`DesertStump_Wood` and `DesertDebris_MudPlate` blend bleak → bloomed colour with it.
Props ship bleak.

Three seeded variants per family, plus per-instance scale, yaw and tilt from the world
seed and instance ID. Contract: `desert/asset.json`.

```sh
~/.local/bin/blender -b --factory-startup -P tools/blender/create_desert_props.py
~/.local/bin/blender -b --factory-startup -P tools/blender/create_desert_props.py -- stump 1
```
