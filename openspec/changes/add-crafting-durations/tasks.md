Hand crafting (1–4) needs nothing new and can start now; delegation (5) waits for the
economy core (`add-settlement-economy` 1–7) and the settlement system.

## 0. Decide (user)
- [x] 0.1 Recipes take time; early ones feel almost instant (user, 2026-11-20)
- [x] 0.2 A progress bar, Satisfactory's bench as the example (user, 2026-11-20)
- [x] 0.3 Two ways to craft: by hand at a station, or delegated to Dancers (user, 2026-11-20)
- [ ] 0.4 Delegated crafts' inputs and outputs: settlement storage, the player's inventory, or both
- [ ] 0.5 Queues: one per station or per crew, and their length

## 1. Rules
- [ ] 1.1 `by_hand { seconds }` on recipes, and the rule that a recipe allows at least one way (with `crew` from `add-settlement-economy` 9.1); positive durations; `rules-fmt`
- [ ] 1.2 Rows: `craft_reed_water_purifier` by hand, 0.75 s
- [ ] 1.3 `assets/recipes/README.md`: durations and the two ways

- [ ] 1.4 Verify recipe forms display, validate and save hand-craft seconds and crew profession, count and quarter notes through the Rules Designer

## 2. The progress bar
- [ ] 2.1 `stubby_ui::progress_bar`: track, fill, stepped or smooth, optional label, in the HUD style

## 3. Hand crafting
- [ ] 3.1 The crafting screen's craft button fills the bar while held (real time), in 0.25 s steps
- [ ] 3.2 Space latches it; letting go pauses; changing recipe or closing resets
- [ ] 3.3 `CraftRequest` sent when the bar fills; holding on repeats while inputs last; `x N`
- [ ] 3.4 A craft that becomes impossible stops and empties the bar
- [ ] 3.5 One focused test of the hold timing (fill, pause, resume) without a window
- [ ] 3.6 Update the purifier end-to-end review to hold Craft, and a bounded screenshot of the bar under the watchdog

## 4. Docs
- [ ] 4.1 AGENTS.md and `add-campfire-craft-bench`: the craft button is timed

## 5. Delegation (after the economy core)
- [ ] 5.1 The Dancers mode on a station's screen: how many, queue, a bar per running job, missing Dancers for a waiting one
- [ ] 5.2 Jobs reserve crew and inputs and deliver once through `stubby_economy` (`add-settlement-economy` 7)
- [ ] 5.3 A world-space bar over a station while a job runs
- [ ] 5.4 A recipe that allows both shows both
