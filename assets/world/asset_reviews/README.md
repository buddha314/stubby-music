# Asset review scenes

Small Jackdaw scenes for looking at new assets in the game's own light, before they have
a place in a level. Each places a set of prefabs near the opening view; they are not levels
and the game never loads them on its own.

| Scene | What it shows |
| --- | --- |
| [vardo.bsn](vardo.bsn) | The vardo, the player's base, beside a campfire |
| [homestead.bsn](homestead.bsn) | The Dancer cottage with its homestead props (table, stools, bench, barrels, bucket, planters, hammock) and a campfire for scale |

Build `stubby_jackdaw` first, then from the repository root:

```sh
python3 tools/jackdaw_timed_test.py --asset-review homestead --name homestead
```

The watchdog runs the standalone game for 20 seconds with the seeded layout off and saves
a screenshot to `/tmp/stubby-asset-homestead.png`. To walk around instead, run the game
directly: `target/debug/stubby_jackdaw --seed=42 --no-seeded-layout
--scene=world/asset_reviews/homestead.bsn`, or open the scene in Jackdaw and press Play.
