# Nature asset sourcing

Where nature assets can come from, what we may do with each, and what to use first. Surveyed
2026-11-06 from the user's local library and live catalogues. Licences change: **read the listing's
terms again before bringing anything in.** This is not legal advice.

## The short version

- **Use Poly Haven first.** It is CC0, and 112 of its 113 nature models are already on disk.
- **Its Namaqualand collection is the best fit:** 30 assets from an arid region, including dead
  trees, boulders and flowers.
- **There are no Megascans in the library.** They are free on Fab as of 2026-11-05, so claim what
  we might want, but they cannot go in this public repository.
- **Almost nothing fits our budget as it comes.** Plan to reduce and restyle every scan.

## The rule that decides most of this

This repository is **public and MIT licensed**, and the game loads loose `.glb` files from `assets/`.

| Licence | Commit to this repo | Ship in the game |
|---|---|---|
| CC0 | Yes | Yes |
| Fab Standard | No: standalone redistribution is barred, and only private sharing is mentioned | Yes, inside the game |
| BagaPie and other paid packs | No | Unknown: read their terms; see below |
| Unknown, or found online | No | No |

So anything not CC0 stays out of the repo until we decide how to handle it (see "Open decisions").

## What is already on this machine

The library is at `~/ProtonDrive/Asset-Libraries/`. Nothing there is in this repository.

| Folder | Size | Nature content | Licence |
|---|---|---|---|
| `Poly Haven` | 11 GB | 409 of 521 models, 801 of 869 textures, 955 of 997 HDRIs; each model has a `.blend`, textures and `info.json` | CC0 ([terms](https://polyhaven.com/license)) |
| `BagaPie` | 49 GB | 10 asset volumes: trees, bushes, cliffs, flowers, grass, ground, leaves, moss, plants, rocks, stumps, wood, plus a Stylized category | **Unconfirmed**, see below |
| `Fab` | 1.4 GB | None found: urban and medieval props and scenes, no Megascans | Per listing |
| `Gumroad` | 633 MB | 31 texture files, not reviewed | Unconfirmed |

Tools installed: the Poly Haven Blender add-on, and Blender 5.3 Alpha. A checkout of Unreal Engine
source is also here, with its Megascans and Bridge plugins, but no downloaded Megascans were found.

### BagaPie: check before using

No licence for the asset volumes was found. The add-on zip's README says "All files of this addon are
under GPL/MIT license", and its `LICENSE` is GPL 3, but the volumes are separate downloads, sold
separately, so this may not cover them. If it did, they could even go in this repository, so it
is worth asking. The closest asset terms found are for [BagaIvy](https://antbaga.github.io/BagaIvy_Documentation/license),
a sister product: its models are royalty free, but sharing individual assets is barred, and in
real-time work assets must be "cooked" so they cannot be retrieved as files. If BagaPie matches, our
loose `.glb` files may not qualify. Ask the creators, or [check the listing](https://abaga.gumroad.com/l/GcYmPC),
before any BagaPie asset goes into the game. Also check the Stylized category first: it may need
the least restyling.

## Poly Haven in detail

Nature models: 37 rocks, 57 plants, 20 trees and 32 ground cover (some in more than one group).
Across nature-related models, the median is about 77k triangles. Polycounts below are the catalogue's.

| Use | Assets | Triangles |
|---|---|---|
| **Bleak: dead wood** | `dead_quiver_branch_01`, `_02` | 15k each |
| | `dry_branches_medium_01`, `dead_quiver_trunk`, `tree_stump_01`, `tree_stump_02` | 17k, 34k, 41k, 62k |
| | `dead_tree_trunk`, `dead_tree_trunk_02` | 102k, 156k |
| **Bleak: rock** | `moon_rock_02`, `_03`, `_04`, `_06`, `_07` | 6k to 14k |
| | `namaqualand_boulder_02` to `_06`, `namaqualand_stones_01`, `namaqualand_rocks_01` | 107k to 205k |
| **Clearing: first life** | `shrub_sorrel_01` | 3k |
| | `cheiridopsis_succulent`, `wild_rooibos_bush` | 199k, 77k |
| | `shrub_02`, `_03`, `_04`, `crystalline_iceplant` | 15k to 60k |
| **Bloomed: flowers** | `flower_empodium` | 6k |
| | `flower_gazania`, `flower_ursinia` | 15k to 60k |
| **Hero trees** | `quiver_tree_01`, `quiver_tree_02` | 270k, 154k |

Only 24 of the 113 are under our 15k budget, and several of those are food or props.

**Textures:** `dry_mud_field_001`, `brown_mud_dry`, `cracked_red_ground`, `dry_ground_01`,
`dry_riverbed_rock`, `bark_brown_01`, `bark_brown_02`. The cracked ground is generated in-engine, so
treat the mud ones as reference for it.

**Skies:** 214 sunrise and sunset HDRIs on disk, 28 in the catalogue as sky-only (for example
`industrial_sunset_puresky`). The game's sky is generated, so these are reference for the keys.

## Megascans and Fab

- **Free now:** [Quixel's site](https://quixel.com/), in a post dated 2026-11-05: "We're making the
  full Quixel Megascans library free on Fab." It also says the terms now allow AI training on them.
  An earlier free period [ended after 2024](https://www.cgchannel.com/2024/10/epic-games-has-made-megascans-free-to-all-but-only-until-the-end-of-2024/), so
  **claim soon**, whatever we decide. We could not confirm whether this offer has an end date.
- **Any engine:** the [Fab licence](https://www.fab.com/eula) says use "is not limited to Unreal
  Engine". Projects containing the assets can be sold.
- **Not in this repo:** it bars redistributing assets "on a standalone basis", and allows sharing
  "via a private repository" with collaborators. It does not address public repositories.
- **Check each listing:** assets from Bridge or under legacy licences can have
  [older, narrower terms](https://forums.unrealengine.com/t/megascans-update-march-2025/2419561), and some
  listings, such as starter tree packs, are
  [reported as Unreal-only](https://tmhsdigital.github.io/Free-Game-Dev-Assets/entry/megascans-starter-content/). Fab's transition FAQ
  could not be read during this survey.
- **Megaplants** are built for Unreal's experimental Procedural Vegetation Editor, so assume they will
  not carry over to Bevy easily.

## Other sources

| Source | What | Licence | Status |
|---|---|---|---|
| [ambientCG](https://docs.ambientcg.com/license/) | PBR materials: ground, rock, sand, mud, bark | CC0 | Terms read |
| [Quaternius Stylized Nature MegaKit](https://quaternius.com/packs/stylizednaturemegakit.html) | 116 stylized models, glTF; the Standard and Pro tiers are free, Source is paid | CC0 | Terms read |
| [KayKit Forest](https://kaylousberg.itch.io/kaykit-forest) | 100+ stylized models in the free tier, glTF | CC0 | Reported in search; confirm |
| Kenney | Stylized kits; our patch clicks are already Kenney's | CC0 | From memory; confirm |
| Sketchfab | Per model, including CC licences and a NoAI tag | Varies | Check each |
| BlenderKit, botaniq, SpeedTree | Commercial or mixed libraries | Varies | Not reviewed |

Stylized packs need far less reduction, but need checking against our look: their shapes may read
as cute, not as Inside's dread.

## Bringing an asset in

1. **Check the licence and record it.** Add the source URL, author and licence beside the asset,
   as `assets/audio/sfx/patch_snap/LICENSE.txt` does.
2. **Open it in Blender.** Prefer the Poly Haven add-on or the on-disk `.blend`.
3. **Reduce.** Prop or tree: at most 15k triangles and 8 materials, with a far level of at most 3k.
   Bake detail from the original into normals if needed.
4. **Restyle.** Move it onto our palette (`docs/art/style-guide.md`), using only base colour, roughness
   and emissive. A photoscan left as it is will clash with the rest of the world.
5. **Check it reads** at all three distances, then export with a script, as other assets do.
6. **Follow the usual pipeline:** `assets-src/blender/<category>/`, `assets/models/<category>/`, level
   of detail in the one glb, and a prefab under `assets/prefabs/`.
7. **Keep our own hero assets ours.** The scrub oak and reeds are made for this game's mechanics, with
   states and seeded variants. Scans are for supporting scenery and reference.

## Open decisions

- **Is photoscanned scenery right at all?** The look is stylised, "Satisfactory meets Inside". A test
  with one restyled dead trunk beside the oak would answer this cheaply.
- **Where do non-CC0 assets live?** Options: stay CC0 only (simplest), or a private assets repository
  or submodule, which complicates building from a public checkout.
- **Does the game need cooked assets?** Packing would help with terms like BagaIvy's, and needs
  checking against Bevy and Jackdaw.

Sources also consulted: [Fab licences and pricing](https://dev.epicgames.com/documentation/en-us/fab/licenses-and-pricing-in-fab),
[Epic forum on Fab redistribution](https://forums.unrealengine.com/t/fab-com-free-assets-license-question/2337478).
