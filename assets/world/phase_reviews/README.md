# Settlement phase review scenes

These four Jackdaw scenes show the same site on the route from a camp to a
solarpunk city. Open a scene in Jackdaw or select it in the standalone
watchdog. Each starts from the default opening view at the same seed and
solar hour (noon). Walk east along the canal from the burbler toward the commons.
The campfire on the north bank and the window rock on the south bank stay
in place in every scene.

| Scene | Existing assets to reuse | Blockout and art backlog |
| --- | --- | --- |
| [Camp](camp.bsn) | `machines/mudpit_burbler`, `structures/campfire`, `streams/stream_segment`, `flora/reeds`, window rock in `props/rocks/rock_formation_2` | Bring in the vardo and cyber-cob when their runtime assets are ready. |
| [Grove](grove.bsn) | Camp families plus `machines/reed_water_purifier` and `flora/scrub_oak` | The scene presets oak growth and uses a water brush for the future clear channel. Extend the working watershed in a separate simulation review. |
| [Village](village.bsn) | Grove families continue through the settlement | Scene brushes show the commons, low workshops and houses on both banks. Design the first production building families and their street fronts. |
| [City](city.bsn) | The original camp, grove and village remain on the approach | Scene brushes show the civic canal, plaza, hall, towers and roof gardens. Design the larger buildings, street-level access and night lighting. |

Keep shared prefabs in their existing folders. The scene files own placement
and brush massing. The brushes indicate layout, height and sightlines. The
water brushes are visual surfaces. Use the existing purifier reviews for
patching, sound, and clarity simulation. The oak growth values are authored
starting states for the review.

After building `stubby_jackdaw`, use the watchdog from the repository root.
Replace `grove` with `camp`, `village` or `city` to compare. The helper captures
a screenshot near the end of its 20-second window.

```sh
python3 tools/jackdaw_timed_test.py --standalone --phase-review grove --name grove
```

See the [phase OpenSpec](../../../openspec/changes/add-settlement-phase-reviews/design.md)
for the capability of each phase and the representative acoustic paths.
