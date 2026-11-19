# Omar: protagonist art direction

Art direction transcribed from the user-supplied `Omar_Character_Art_Direction.zip`
(2026-11-18). Omar is the player character: a well-travelled explorer of the desert. The
look is warm, humane and resourceful solarpunk with restrained practical sci-fi, not
glossy military armour or hard-edged cyberpunk.

The packaged images are AI-generated concept explorations made during the design
conversation. They are neither model sheets nor engineering drawings. Where an image and
the written decisions below disagree, follow the written decisions. **The master
reference is [master.png](omar/master.png)** (`10_omar_current_no_plant.png`): the
weathered iteration with neon goggles and canteen and **no plant**. Every earlier image is
an exploration or a detail sheet.

## Current scope: gloves and shadow

**Decided (user, 2026-11-18): for now the game needs only Omar's gloves and his
shadow.** The game is first person, so the player sees two things of Omar:

- **The gloves.** These are the hands that hold the hand item (the field recorder today,
  `stubby_player::hand`). Long midnight-navy gloves, practical, flexible and subtly
  segmented, with a worn-leather sheen. A short band of the coat's dark-mint sleeve
  (flared, with aged brass edging) may show at the wrist where the arm leaves the frame.
  Glove detail: [turnaround-gloves-palette.png](omar/turnaround-gloves-palette.png)
  (the glove close-up, right column) and
  [sleeve-and-glove-details.png](omar/sleeve-and-glove-details.png).
- **The shadow.** Omar's full silhouette is cast on the ground, though his body is never
  rendered to the camera. It must read as Omar, from the silhouette studies in
  [turnaround-gloves-palette.png](omar/turnaround-gloves-palette.png) (lower left): a tall
  head wrap with the tagelmust's fringed end hanging, a fitted torso, a knee-length coat,
  short flared sleeves, trousers puffed at the knee and gathered into heavy boots. The
  shadow should follow walking, crouching and holding the recorder, so the silhouette and
  the hands agree.

Everything else below is recorded for later: a third-person view, cutscenes, the vardo
camp, a mirror or a photo mode. Do not build it until the user asks.

## Core identity and silhouette

- **Omar**: an adult man with dark skin, his complexion referencing North Africa and
  Ethiopia. Clothing and eyewear conceal most of his skin.
- Approachable, capable, well-travelled.
- Silhouette: a tall orange head wrap, a fitted upper torso, a knee-length double-breasted
  coat, slightly flared sleeves, and loose sand trousers tucked into substantial boots.
- Treat the cultural references with the same care as the myths and the vardo: build from
  the references, not caricature. The tagelmust is a real Tuareg garment.

## Head and face

- A **bright, friendly orange tagelmust**, wrapped around the head and covering the mouth
  and lower face, with a dangling fringed end. Slight wear and fine textile patterning.
- Futuristic wraparound **goggles** in the manner of a mixed-reality headset: a dark
  reflective faceplate in a sculpted, light-coloured frame, with small cyan/teal and
  magenta/violet neon highlights that suggest a HUD.
- The mouth is never shown. No full helmet, bulky respirator or excess neon.

## Coat

- **Brightened dark mint / muted teal-green**, weathered but rich in colour, reaching just
  above the knee.
- Loosely chef-coat-inspired with a **banded standing collar**, restrained officer
  tailoring, a **double breast** and several **frog fasteners** across the chest.
- Sleeves slightly short and mildly flared, showing the long gloves.
- Worn edges, faded seams, fine stitching, small repairs, light dust. Understated brass
  fittings, delicate botanical and geometric embroidery.
- A functional belt and compact pouches. No oversized weapons or bulky armour.

## Arms and hands

- **Long midnight-blue gloves**, visible below the sleeves: practical, flexible,
  protective, subtly segmented.

## Trousers and footwear

- **Sand-coloured trousers**, loose and puffed at the thigh and knee, gathered into the
  boots.
- **Serious combat or hiking boots**: dark, scuffed, robust soles, lacing and functional
  fasteners, light dust and wear.

## Gear and light accents

- A practical **canteen** at the hip: weathered metal with restrained cyan/teal and
  magenta/violet luminous rings.
- A compact amber-lit lantern or canister is allowed on the shoulder or belt. Its warm
  amber complements the orange wrap.
- Lights are accents, never the main illumination. No large glowing panels or flashy
  armour.
- **No plant.** No foliage on the shoulder, the pack or anywhere on the equipment
  (`09_weathered_neon_with_plant.png` was superseded for this reason).

## Palette

| Part | Colour |
| --- | --- |
| Tagelmust | vivid warm burnt orange / saffron |
| Coat | brightened dark mint / muted teal-green |
| Gloves | midnight navy blue |
| Trousers | warm sand / beige |
| Boots and straps | dark worn charcoal leather |
| Metal | aged brass, weathered steel |
| Neon | restrained cyan/teal, occasional violet/magenta; amber for the lantern |

## Surface and mood

Used, maintained and field-ready: fine dirt, abrasions, patches, textile grain, worn
leather, slight sun fading. Keep the materials to PBR base colour, roughness and emissive,
as the style guide requires.

## Continuity: avoid

- No plant or living foliage on the character.
- Never expose the mouth, and never replace the wrap with a gas mask.
- Never shorten the coat above mid-thigh or turn it into a cape.
- Keep the frog closures, short flared sleeves, long gloves, sand trousers, heavy boots
  and wraparound goggles.
- Keep the brighter green and orange of the later iterations, and do not over-saturate
  the neon.

## Reference files

The source archive holds ten PNG explorations and a contact sheet (about 23 MB). This
directory keeps four of them. The full archive stays at the user's Downloads path.

| File | Source | Use |
| --- | --- | --- |
| `omar/master.png` | `images/10_omar_current_no_plant.png` | **current master**: palette, materials, gear |
| `omar/turnaround-gloves-palette.png` | `images/04_first_turnaround.png` | glove close-up, silhouettes for the shadow, palette swatches, front, side and back views |
| `omar/sleeve-and-glove-details.png` | `images/03_first_concept_sheet.png` | sleeve cuff over the glove, frog fasteners, embroidery, boots |
| `omar/contact-sheet.jpg` | `CONTACT_SHEET.jpg` | every iteration, overview only |

Earlier iterations (01 to 08) are subdued or alternative outfits. Use them for breadth
only. The PNGs are design inputs; runtime assets follow the Blender-to-glTF pipeline.
