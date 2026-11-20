# Previewing long sessions in simulation games

A simulation preview can answer three design questions. What happens if the
player leaves a system running? What happens after a particular series of
player decisions? What should a mature world feel like to visit? Each question
calls for a different starting state and a different kind of evidence.

## Fast-forward a known state

Factorio's map editor can pause, advance a tick or a chosen interval, change
simulation speed, and set the time of day. Its command line can run a scenario
or save headlessly for a bounded number of ticks, and its benchmark mode can
reload the same save for each run. Wube also describes profiling large saves
that stress different parts of the game. A practical workflow is to start two
runs from the same state, change one intervention, and compare at the same
simulation time. Keep a dense layout for performance review and a smaller
layout where a designer can trace individual changes. [Factorio map editor](https://wiki.factorio.com/Map_editor) · [Factorio command line](https://wiki.factorio.com/Command_line_parameters) · [Wube on profiling large saves](https://www.factorio.com/blog/post/fff-165)

A mature state can have a long adjustment period after a rules change.
Colossal Order's Economy 2.0 diary describes demand changing quickly after
unpausing an existing city, while businesses, employment, and housing adjust
over time. For design reviews, record the starting condition, the transient
response, and the later condition. A screenshot immediately after a change
cannot tell you whether the change settles or keeps spreading. [Colossal Order's Economy 2.0 diary](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-two)

## Replay the decisions that create a world

Wube describes deterministic save/load as an observable equivalence between
continuing a run and resuming it after a save. Tarn Adams gives a related
design example for Dwarf Fortress. Generate a seeded history to year 110, or
generate it to year 100 and play ten years yourself. Legends can help explain
where those histories diverged. Stubby Music can borrow that comparison at a
smaller scale by starting with one seed and replaying world-level actions at
named milestones. Reuse the same actions to compare balance changes. Keep a
second valid build order to see whether the outcome depends on the route the
player took. [Wube on deterministic save/load](https://www.factorio.com/blog/post/fff-270) · [Interview with Tarn Adams](https://www.gamedeveloper.com/programming/how-tarn-adams-upgraded-and-optimized-dwarf-fortress-for-its-official-steam-release)

## Stubby Music's current tools

The speed keys already reach 64×. The calendar uses elapsed master-clock time
when audio is available and falls back to Bevy virtual time. The clock and
oscillators keep their musical rates as the world speeds up. A full 3,600-second
world day takes about 56 seconds at 64× under steady updates. Frame caps can
make it take longer. The 20-second graphical watchdog is suitable for inspecting
a prepared moment. Long runs belong in headless tests. [Calendar code](../../crates/stubby_world/src/calendar.rs) · [Existing speed and dusk review](day-night-and-simulation-speed.md)

The dusk review already prepares an appropriate calendar time and records
screenshots and JSONL. The purifier review prepares a laid stream, a running
burbler and an unpatched purifier. These are two reusable patterns for future
previews. Growth and purifier clarity already have headless tests driven by
virtual time. A first long-run test can reuse those systems with explicit time
steps. Setting the calendar can select the sky at a checkpoint. Growth,
clarity, resource returns and scheduled actions need updates over the elapsed
interval. [Dusk driver](../../crates/stubby_game/src/review.rs) · [Purifier driver](../../crates/stubby_game/src/purifier_review.rs) · [Growth](../../crates/stubby_game/src/watering.rs) · [Clarity](../../crates/stubby_game/src/purifier.rs)

The proposed scenarios and the intended late-session experience are described
in [Long-session design previews](../design/long-session-previews.md).
