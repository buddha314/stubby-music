## Decisions

### D1. Two ways to craft (user, 2026-11-20)

| | By hand | Delegated to Dancers |
| --- | --- | --- |
| Where | at a crafting station (the campfire today) | at a station in an established village |
| Who | the player, standing there | a crew of Dancers of a profession |
| Time | real time, while the button is held (UI time, AGENTS.md) | worksite quarter notes, unattended |
| Inputs | taken when the bar fills; nothing taken if let go | reserved when the job starts (`automate-godot`'s prepare/complete) |
| From and to | the player's inventory | the settlement's storage (open: or the player's) |
| Recipes | those with `by_hand` | those with `crew` |

A recipe may allow either or both. Early recipes are by hand only; many later ones need
a crew (user, 2026-11-20), and some may allow both, so a player can make one now by hand
or hand the batch to Dancers.

### D2. Durations are rows; early ones feel almost instant (user, 2026-11-20)

`by_hand: { "seconds": 0.75 }` and `crew: { "profession": ..., "dancers": N, "quarter_notes": NQ }`.
Seconds, not Satisfactory's hammers, but the bar steps in Satisfactory's 0.25 s so short
recipes read as a few quick ticks. Starting guide: the first recipes 0.25 to 1 s; later
hand recipes a few seconds; crewed recipes a budget of worksite quarter notes. The purifier kit starts
at 0.75 s, like Satisfactory's iron ingot.

### D3. Hand crafting follows Satisfactory's bench

- Holding the craft button (mouse) fills the bar; Space latches it held until pressed
  again or the screen closes.
- When the bar fills, the inputs are taken and the outputs given in one step, as now
  (`CraftRequest` at completion, so the rules core stays instant and atomic).
- Holding on starts the next craft at once; `x N` counts them, as now.
- Letting go pauses the bar; it keeps its progress while the same recipe stays selected
  and the screen stays open, and resets otherwise.
- A craft that becomes impossible (inputs gone, tempo out of range) stops and empties its
  bar, taking nothing.

### D4. Delegated crafting

On the station's screen, a crewed recipe shows **Dancers** instead of (or beside) the
hold button: the player sets how many to make and queues them. Each job reserves its crew
and inputs through the economy (`add-settlement-economy` section 7), advances on worksite quarter notes and
delivers once. The screen lists the station's queue with a bar per running job and how
many Dancers are missing for a waiting one; a small bar floats over the station in the
world while a job runs, so delegated work is visible from a distance.

### D5. One progress bar

`stubby_ui::progress_bar`: a filled track in the HUD style, stepped or smooth, with an
optional label (`x N`, `2 / 5`). The hand craft uses it stepped, delegated jobs smooth;
machines and the dismantle hold (`stubby_streams`) can use it later.

## Open

- Whether delegated crafts draw from settlement storage, the player's inventory or both.
- Whether a station has one queue or one per crew, and a queue's length.
- Whether a hand craft's steps can land on the bench's clock (the bench is an instrument).
