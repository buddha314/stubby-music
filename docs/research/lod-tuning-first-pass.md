# First LOD tuning pass — 2026-11-16

Branch `tune/opening-lods`, based on the sunrise opening (`e6e4b7f`). This pass tunes
import metadata and adds repeatable evidence capture. Geometry, authored level
names and generated simplification ratios/errors are unchanged.

## Reference settings

45-degree vertical FOV, unit placement scale, metres measured using imported model
size. Jackdaw quantizes size, so actual distances differ slightly. Crossfade width
is 12% of each switch distance (6% on each side), including final disappearance.
Runtime FOV and static placement scale still affect the distances.

| Family | Handover centres | Final disappearance centre |
| --- | --- | --- |
| Scrub oak | 45 m, read from family profile | 240 m |
| Reeds | 30 m, read from family profile | 120 m |
| Burbler | 30 / 80 m | 240 m |
| Desert brush | 20 / 50 m | 90 m |
| Desert stump | 25 m | 100 m |
| Shell, unit scale | 1 m | 12 m |
| Shell, placed 3x | about 3 m | about 36 m |

Previously every final threshold was 1% of screen height and every fade was Snap.
That meant about 450 m for oak variant 0, 229 m for reeds 0, 784 m for the placed
burbler, and 10.9 m for a shell. The shell used a copied reed threshold, switching
to its coarse mesh around 1.4 m; it now retains the detailed mesh into the normal
collection distance and crossfades near 3 m. Other handover centres are preserved.

The authoring source is `tools/assets/lod_policy.json`; flora handovers are read
from their existing `asset.json` profiles. `tools/jackdaw_lods.py --switch` applies
it to existing metadata without contacting the editor. `--check` verifies exact
agreement and CI runs it. Unknown LOD-configured models fail instead of silently
inheriting an unrelated cutoff. Normal game loading never rewrites settings.

## What was reviewed

Build `stubby_jackdaw`, then run only the bounded diagnostic:

```sh
python3 tools/jackdaw_timed_test.py --lods flora --name=lod-flora-after
python3 tools/jackdaw_timed_test.py --lods machines --name=lod-machines-after
```

The adapter's opt-in tripod driver uses the real production oak, reeds, burbler
and shell; it adds temporary brush/stump fixtures that are absent from the opening.
Each set records 12 shots and completes before the independent watchdog stops the
process at 20 seconds. The review holds noon for legible silhouette comparisons;
it does not save changes to the dawn opening. No unseen audio patches are added.

Both sets were captured before and after. Evidence lives under
`/tmp/stubby-lod-lod-{flora,machines}-{before,after}/` as PNGs and `events.jsonl`.
Watchdog logs/reports are `/tmp/jackdaw-flicker-lod-*.{log,json}`. The four runs took
20.018–20.025 seconds each with expected exit -9, display mode restored, and no
asset errors or panics. The review's complete event reports 12 shots in each run.
The tuned machine set was repeated after integrating the concurrent sequencer
merge (`70f638b`) and its refreshed burbler asset; all 12 shots completed and the
reported burbler geometry counts/cutoff behavior were unchanged.

## Observed results

- Oak transition at 45.4 m requests both levels. Its tuned band is approximately
  42.4–47.8 m. Near/dormant and grown states were inspected; far state uses the
  791-triangle authored level instead of the 11,152-triangle near level. During a
  crossfade both levels draw, so cost rises briefly rather than magically vanishing.
- At 270 m the tuned oak has zero CPU-visible submitted mesh triangles, compared
  with an old final cutoff around 450 m. Jackdaw may retain a ready level beyond
  the draw cutoff due to its hysteresis; ready/wanted masks are not draw counts.
- Reeds transition around 28.3–31.9 m and use the six-triangle far representation.
  A harvested far clump stays visually hidden instead of reappearing as standing
  reeds. Its zero-scale mesh can still be submitted, so the reported index count
  does not by itself prove useful visible geometry.
- Shell at 3 m now requests/draws both 9,234- and 1,032-triangle levels within its
  short handover band; at 12 m only the coarse level draws. At 40 m it draws none.
  The old shell was already coarse at 3 m and outside its draw range at 12 m.
- Burbler at 300 m draws no mesh candidates after tuning. Its final fade spans
  about 231.8–261.4 m for the actual placed size, down from about 784 m.
- Stump at 110 m now draws none; the prior cutoff was about 110.4 m without fade.
- Widening vertical FOV to 70 degrees moves the oak handover to about 25.1–28.3 m,
  as expected for screen-size selection, without a second game distance selector.

`submitted_triangles` counts index triangles on CPU-visible render entities,
including descendants of animated scene instances. It is not GPU timing or a
rasterized-pixel count. Early baseline machine logs counted only flat parts and
reported zero for the visibly present animated burbler; these zero baseline counts
are invalid and must not be used as performance evidence. The driver now follows
scene-instance descendants. No frame-rate improvement is claimed.

## Engine findings and next pass

1. **Live root-scale changes leave stale switches.** At scale 1.5, the already-loaded
   oak still had its unit-scale 45 m handover instead of roughly 67.5 m. The same
   behavior was present before tuning. The pinned Jackdaw `place_lod_ranges` due-set
   reacts to model/live-level changes but does not include changed GlobalTransform.
   Initial production scales and all production placement data are untouched by
   this pass; dynamically resizing an existing instance needs an editor/runtime bug
   fix. The review deliberately records this limitation rather than hiding it with
   a game-specific LOD implementation.
2. **Generated ratios are not proven geometry savings.** In the tuned review,
   burbler stayed at 15,280 submitted triangles across its three requested levels,
   brush at 3,016 and stump at 1,310 before their final cull. The authored oak/reed/
   shell levels reduce geometry as expected. Audit generated handle substitution,
   topology/border constraints and the animated scene-instance route before
   claiming the configured 50%/25% ratios actually reduce rendering cost. In
   particular the pinned `place_instance` instantiates the original glTF scene for
   animated models; further diagnosis is needed before assigning a single cause.
3. Review motion continuously through the fade bands and all art variants/angles,
   including the actual dawn/fog conditions and headset FOV. Current evidence is
   representative fixed shots at one seed and noon. It is a starting calibration,
   not final visual sign-off or indefinite display-stability testing.

## Checks

- Three authoring-policy tests: idempotence/level preservation, invalid-distance
  rejection without writes, and rejection of unconfigured models.
- Existing prefab metadata validation parses the real Jackdaw schema, including the
  new crossfade and every named authored level.
- Strict OpenSpec validation, asset references, version consistency and changed
  Rust/Python syntax/format checks.
