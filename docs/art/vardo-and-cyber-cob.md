# Vardo and cyber-cob art direction

Art direction transcribed from the user-supplied `Vardo_Cyber_Cob_Art_Direction.zip` (2026-11-18). The vardo and its pony are a paired, lived-in working set: an Irish Traveller barrel-top living wagon with a sturdy piebald Irish cob. Keep the design practical and handmade, with restrained organic solarpunk and cyberpunk details.

The packaged images are exploratory concept art supplied by the user. They are neither historical evidence nor engineering drawings. Some images conflict with these written continuity decisions; use the written decisions below when they differ. In particular, do not copy unsupported historical labels, inconsistent wheel sizes, donkey substitutions, or multiple prosthetic limbs from an image. Use the [contact sheet](vardo-cyber-cob/contact-sheet.jpg) for breadth and the selected [wagon orientation study](vardo-cyber-cob/wagon-orientation-study.png), [vardo side study](vardo-cyber-cob/vardo-side-study.png), and [cyber-cob side study](vardo-cyber-cob/cyber-cob-side-study.png) for material and silhouette cues.

## Vardo

- Compact four-wheeled Irish Traveller barrel-top living wagon, not a lavish English Reading wagon. The continuous rounded roof must remain clearly readable.
- The **front is the towing end**: two shafts and a pivoting steering axle. Omit the conventional driver's bench. The **rear is the entrance**: half-door, removable steps, rack, and practical storage.
- Two wheels per axle. Front wheels are smaller; target rear diameter is roughly 1.3–1.6 times the front diameter. Keep axle positions, shaft attachment, steering clearance, and wheel sizes consistent across every view.
- Painted timber planks in dark forest green with weathered burgundy/red trim and muted cream/gold folk-floral accents. Show age, repairs, dirt, chipped paint, and practical ironwork.
- Include modest curtained windows, a small stove and chimney, lantern, wool blankets, and practical fixtures. Rear rack and pan box carry cookware and possessions; cargo remains modest.
- Broadly evoke a late nineteenth- to early twentieth-century working wagon, but label imaginative details as fictional rather than documented historical features.

## Grown roof and restrained technology

- The roof's living wood is structural: naturally curving ribs and root-like transitions grow into the wagon's framing. It must read as grown in place, not as vines laid over an ordinary roof.
- Moss, low vegetation, and a few sprouting leaves sit on the roof without obscuring its barrel profile. Organic framing follows window and door openings without blocking usable hinges or fittings.
- The base remains traditional and handmade. Add only a few soft cyan/teal leaf glows, one narrow magenta/purple ring on the chimney, and narrow cyan or magenta wheel-rim/hub accents. Do not turn the wheels into glowing discs.
- Balance cool accents with warm amber lantern and interior light. Brass and iron mechanisms are functional and weathered. No gratuitous armor or weapons.

## Cyber-cob

- One compact, sturdy, black-and-white piebald Irish cob/Traveller-type working horse. Use irregular coat patches, shaggy black/cream mane, rough tail, feathered lower legs, muddy hooves, strong shoulders, and gentle alert eyes. It is healthy and capable, plainly kept, and not a groomed show animal.
- Exactly **one** prosthetic limb: the near-side front leg (visible with the horse facing right). The other three legs are natural. Do not use the exploratory hind-leg alternative or change the injured side between views or variants.
- The prosthesis is rugged, load-bearing, articulated, and field-repaired, using battered steel/brass, leather support, bolts, mechanical joints, and a hoof-shaped foot. Cyan indicators are tiny and dim.
- A few fine, dim cyan/teal fiber-optic strands follow the hair direction through **both** mane and tail. Natural hair remains dominant; the horse must read as biological, with a calm working-horse demeanor.
- Use only a simple rope or worn-leather halter and practical harness as needed. No elaborate tack.

## Presentation and continuity review

- Isolated object studies use a clean neutral studio backdrop; illuminated variants may use charcoal. Use consistent three-quarter, side, front, and rear views. No people unless requested.
- Confirm: shafts at front; entrance and rack at rear; four wheels and two axles; smaller front wheels; coherent geometry in every view; structurally grown barrel roof; restrained light accents; exactly one near-side front prosthesis; three natural legs; sparse optical strands in both mane and tail.
- The pony is the vardo's approved animal. Do not substitute a donkey.
- References in this directory are design inputs only. Runtime assets must follow the repo's Blender-to-glTF pipeline and asset-family layout; the PNGs are not game assets.

## Reference files

The source archive contains 36 large PNG explorations (~100 MB unpacked). This directory keeps the contact sheet and three representative studies for efficient review. The full source archive remains at the user-provided Downloads path; the selected images retain their original creative intent but have shortened repository filenames.

| File | Use |
| --- | --- |
| `vardo-cyber-cob/contact-sheet.jpg` | `CONTACT_SHEET.jpg`; overview only |
| `vardo-cyber-cob/wagon-orientation-study.png` | `images/a_clean_studio_illustration_look_highly_detailed.png`; paired wagon views |
| `vardo-cyber-cob/vardo-side-study.png` | `images/a_detailed_watercolor_ink_technical_illustration_s.png`; side silhouette and aged timber/iron materials |
| `vardo-cyber-cob/cyber-cob-side-study.png` | `images/a_detailed_hyper_realistic_concept_art_photoreal.png`; coat, mane, single prosthetic leg, and subtle light direction |

## The vardo model (2026-11-20)

`tools/blender/create_vardo.py` builds it from the rules above: `assets/structures/vardo/`
(glb with `vardo_LOD0`/`_LOD1`, prefab, profile, colliders), previews in
`previews/structures/vardo/` (three-quarter, rear three-quarter, side, front, rear, top,
night). 11.2k triangles near, 208 far. Front wheels 0.9 m, rear 1.3 m (ratio 1.44); the
shafts and a turntable at the towing end, the entrance at the rear with a half-door (its top
leaf open inward, the interior lit), removable steps, a pan box with pans, a ladder rack
with a basket and a wool blanket, and a lantern. The roof's end ribs grow down the corner
posts into the sill. Accents are restrained: three small cyan leaf glows, one magenta ring on
the chimney, thin cyan rim lines. Every material declares an acoustic class (painted planks,
trim, roof and wheels `wood`, iron `metal`, windows `glass`). Sockets: `Door`, `Chimney_top`,
`Hitch` (where the cob stands, for the pair review). In the game:
`python3 tools/jackdaw_timed_test.py --asset-review vardo`.

## The cyber-cob model (2026-11-20)

Built on a rigged, animated CC0 base rather than from scratch (user, 2026-11-20): "Horse" by
Quaternius (Ultimate Animated Animals, CC0 1.0; `assets-src/third_party/quaternius/`). Its
50-joint rig and gentle clips are kept (Idle, Idle_2, Idle_Headlow, Eating, Walk, Gallop; no
attacks or deaths); `tools/blender/create_cyber_cob.py` restyles and extends the mesh:
piebald coat (black head with a white blaze, white lower legs), a black-and-cream mane and
tail, a body widened through the barrel and shoulders into a cob, feathered fetlocks,
muddy hooves, a worn leather halter with a rope lead, a collar with brass hames, a back pad,
belly band and brass shaft tugs, a few dim cyan fibre-optic strands in mane and tail, and
one prosthetic foreleg (leather cuff with brass straps on the stump, brass knee housing,
steel pylon and piston rod, ankle joint, hoof-shaped foot, one tiny dim cyan indicator).
Every added part is skinned to the rig, so every clip moves it. 1.48 m at the withers, 4.5k
triangles, one skinned level. Previews: `previews/creatures/cyber_cob/`.

**The prosthesis is on the horse's own right foreleg** (confirmed by the user, 2026-11-20): the
leg facing the viewer with the horse facing right, as the study shows. ("Near-side" above
meant the side nearest the viewer, not the equestrian near side.)

**Feathering** (fixed 2026-11-20 after review): long hair from the back and sides of the
fetlock falls over the top of the hoof, and the hoof's front stays clear; the first pass
splayed a full ring of cards to the ground and hid the hooves.

**In the game** it stands at the vardo's `Hitch` in `--asset-review vardo`. Its `Idle` clip
plays in time with the music: phase-locked to the nearest running clock within 30 m, or to
the master clock (user, 2026-11-20), as any placed model with an `Idle` clip now does.

## The harness (2026-11-21)

The pair now meets as a shaft harness does. The shafts end at the cob's shoulder and ride
in two brass **tugs**, rings hung from the back pad on short leather straps where the shafts
pass (`SHAFT_AT_PAD_M` in `create_cyber_cob.py`). A leather **trace** each side, part of the
vardo, runs from the cob's hames back along its flank, bowing clear of the barrel, to the
ends of the swingletree (`Vardo.traces`), so it meets the cob wherever it stands at the
`Hitch`. The pair sheet, rendered from the exported models with the cob at the hitch:
`tools/blender/render_vardo_pair.py` -> `previews/structures/vardo/pair_sheet.png`.

