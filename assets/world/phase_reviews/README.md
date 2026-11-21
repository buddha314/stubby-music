# Settlement phase review scenes

These four Jackdaw scenes show the same site on the route from a camp to a
solarpunk city. Open a scene in Jackdaw or select it in the standalone
watchdog. Each starts from the default opening view at the same seed and
solar hour (noon). Walk east along the canal from the burbler toward the commons.
The campfire on the north bank and the window rock on the south bank stay
in place in every scene.

| Scene | Existing assets to reuse | Blockout and art backlog |
| --- | --- | --- |
| [Camp](camp.bsn) | `machines/mudpit_burbler`, `structures/campfire`, `streams/stream_segment`, `flora/reeds`, window rock in `props/rocks/rock_formation_2` | The vardo stands on the north bank with the cyber-cob at its hitch (2026-11-21). |
| [Village](village.bsn) | Camp families plus `machines/reed_water_purifier` and `flora/scrub_oak` | The scene presets oak growth and uses a water brush for the future clear channel. The vardo and cob, two cottages (the first with its homestead props) and six Dancers: two dance by the fire, one tends the oak by the first dam, two talk on a porch (2026-11-21). Extend the working watershed in a separate simulation review. |
| [Town](town.bsn) | Village families continue through the settlement | Scene brushes show the commons, low workshops and houses on both banks. Design the first production building families and their street fronts. |
| [City](city.bsn) | The original camp, grove and town commons remain on the approach | Scene brushes show the civic canal, plaza, hall, towers and roof gardens. Design the larger buildings, street-level access and night lighting. |

Keep shared prefabs in their existing folders. The scene files own placement
and brush massing. The brushes indicate layout, height and sightlines. The
water brushes are visual surfaces. Use the existing purifier reviews for
patching, sound, and clarity simulation. The oak growth values are authored
starting states for the review.

After building `stubby_jackdaw`, use the watchdog from the repository root.
Replace `village` with `camp`, `town` or `city` to compare. The helper captures
a screenshot near the end of its 20-second window.

```sh
python3 tools/jackdaw_timed_test.py --standalone --phase-review village --name village
```

See the [phase OpenSpec](../../../openspec/changes/add-settlement-phase-reviews/design.md)
for the capability of each phase and the representative acoustic paths.
