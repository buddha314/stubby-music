# Research: synthesis libraries and a thread-safe audio system for Bevy

*2026-10-31. Crate versions are from crates.io on this date. API notes on Firewheel
are from prior knowledge, not yet confirmed against its 0.14 docs. The spike in
`add-audio-engine` task group 2 confirms or refutes them.*

## The question

We need a modular synth (oscillators, clocks, sequencers, filters, VCAs) whose
**patch graph changes at runtime** as the player builds, running glitch-free beside a
Bevy game. Where does the DSP live, which library does the DSP, and how do the two
sides talk?

## Why not inside Bevy's schedule

Bevy systems run once per frame (~16 ms at 60 fps, and jittery). Audio needs a buffer
every ~2–10 ms, on time, every time. If Bevy hitches (asset load, shader compile, GC
of large entity sets), audio rendered in a system will drop out. So the DSP runs on
the **audio device's callback thread**, which `cpal` provides, and Bevy only sends
messages. This also keeps the engine testable and reusable with no window or GPU.

`bevy_audio` (built in, rodio-based) plays assets. It has no runtime patch graph and
no sample-accurate scheduling, so it is unsuitable here.

## Candidates

| Crate | Ver | What it is | Runtime graph edits | Fit |
| ----- | --- | ---------- | ------------------- | --- |
| **fundsp** | 0.23.0 | DSP library: large set of oscillators, filters, envelopes, effects; compile-time graph combinators **plus** a dynamic `Net` | Yes — `Net` has a frontend/backend split: edit the frontend, `commit()`, and the realtime-safe `NetBackend` picks up the change | **Strong.** Best DSP coverage; the `Net` split is the exact pattern we need. Not game-aware: we write scheduling and Bevy glue ourselves. |
| **firewheel** | 0.14.0 | Audio graph engine built for games (cpal backend, node graph, parameter patching, event scheduling) | Yes, by design | **Strong contender.** Game-oriented and built with Bevy integration in mind. Smaller built-in DSP set; fundsp nodes may be wrappable. API still pre-1.0. |
| **bevy_seedling** | 0.8.0 | Bevy ECS integration of Firewheel — nodes as entities | Yes | Attractive if we pick Firewheel, but it makes audio nodes ECS entities, which pulls the engine *toward* Bevy. Evaluate against our "no bevy dep in the engine" rule. |
| **knyst** | 0.5.1 | Runtime audio graph, sample-accurate scheduling, can host fundsp units | Yes | Good design, but its own docs warn the API "can vary wildly between releases". Too much churn for a foundation. |
| **kira** / bevy_kira_audio | 0.12.6 / 0.27.1 | Game audio playback: tweening, tracks, clocks | Limited (effects/tracks, not a patch graph) | Great for SFX and music *playback*, not synthesis. Kira's clock concept is worth borrowing. |
| **cpal** | 0.18.2 | Cross-platform device I/O | n/a | Required underneath whichever we choose. On Fedora it uses ALSA, which PipeWire serves. |
| **dasp** | 0.11.0 | Sample/frame primitives | n/a | Optional utility. |

Realtime plumbing:

| Crate | Ver | Use |
| ----- | --- | --- |
| `rtrb` | 0.4.1 | Wait-free SPSC ring: commands in, events out, garbage out |
| `triple_buffer` | 9.0.0 | Latest-value snapshots (meters, clock phase) without locks |
| `basedrop` | 0.1.3 | Deferred deallocation of shared data off the audio thread |
| `assert_no_alloc` | 1.1.2 | Aborts if the audio thread allocates (debug builds) |

## Recommendation

1. **Own the boundary.** Define our own command/event protocol in
   `crates/stubby_audio` (no `bevy` dependency). Bevy holds producer/consumer handles
   as a `Resource`. Whatever library sits behind the boundary can then be swapped
   without touching game code.
2. **Default to fundsp behind the boundary**, using `Net` for the live patch graph.
   It has the deepest DSP library, and modular synthesis lives or dies on module
   variety.
3. **Spike Firewheel before committing.** Build the same sine + clock patch on both
   and compare: glitch-free graph edits, scheduling precision, DSP coverage, and how
   well it holds the engine apart from Bevy. Firewheel wins if its scheduling and
   game integration save more work than fundsp's DSP breadth does. Record the result
   in `add-audio-engine/design.md` → *Open Questions*.
4. **Thread, not process.** A separate audio *process* (a synth server, as
   SuperCollider does) gives crash isolation but adds IPC, lifecycle management, and
   shared-memory complexity. A thread behind a message boundary keeps that option
   open: the protocol is already serialisable-shaped. Revisit only if audio crashes
   start taking down the game.

## Shape of the system

```
 Bevy main world                      audio callback thread (cpal)
 ──────────────                       ───────────────────────────
 systems ──Command──▶ [rtrb ring] ──▶ drain commands (bounded per block)
                                      apply graph edits / params
                                      render block (fundsp Net / firewheel)
 clock-light system ◀─Event─[rtrb]──  emit ClockTick{sample_time}
 meters/visuals    ◀─[triple_buffer]─ publish phase, levels, sample_pos
 drop thread       ◀─Garbage─[rtrb]── hand back replaced nodes
```

- **Node construction happens off-thread.** Bevy (or a helper control thread)
  allocates a module's DSP node, boxes it, and sends the box. The audio thread only
  moves pointers.
- **Removed nodes come back out** on a garbage ring and are dropped off the audio
  thread. `fundsp::Net`'s backend already does this for its own edits; with a custom
  layer we do it explicitly.
- **Time sync.** The audio thread publishes `(sample_position, wall_instant)` each
  block. Bevy converts event sample times to predicted display times, so the clock
  light flashes on the beat rather than up to one frame late.
- **Bounded work.** The callback drains at most N commands per block, so a flood of
  edits cannot cause an underrun. The rest wait for the next block.
- **Failure mode.** If the ring is full, Bevy-side `send` returns an error the game
  surfaces. It never blocks the main thread.

## Sources

- fundsp — https://docs.rs/fundsp (0.23.0)
- knyst — https://docs.rs/crate/knyst/latest (0.5.1; API-stability warning in README)
- firewheel, bevy_seedling, kira, cpal, rtrb, triple_buffer, basedrop,
  assert_no_alloc — crates.io listings, versions as of 2026-10-31
