## Context

Module outputs reach the speakers mono in both channels. Lightning bugs fade with distance by setting a VCA's gain from the game, with no direction. `add-rack-and-spatial-audio` plans HRTF voices, ranking and occlusion, but depends on its rack engine, which is not built.

## Goals / Non-Goals

**Goals:**

- A burbler's voice is locatable by ear, by direction and level, on speakers or headphones.
- A long stream carries the sound along its whole length.
- Keep the rule that only `stubby_audio` names Firewheel types.

**Non-Goals:**

- HRTF, voice budgets and occlusion, which stay in `add-rack-and-spatial-audio`.
- Sound travelling down the stream with a delay: every point sounds at once.
- Moving lightning bug clicks to this path.

## Decisions

**Place the output, not a new patch module.** "A rack plays only what it shows" covers modules and cables. World position is not a module, so the rack panel and its patch are untouched, and `SetEmitter` adds the spatial node behind the normalled output.

**Firewheel's `SpatialBasicNode`.** It is already in the stack, with smoothing built in. It treats y and z only as distance, so it cannot tell front from back. A camera turn still moves the sound between the ears. HRTF is the planned fix.

**Nearest point of machine or stream.** A stream is the played sequence laid out in space, so standing at its far end should sound like standing at the water. One voice at the nearest point keeps this simple and needs no extra audio voices.

**The radius is the game's volume, so the node's own distance loss is off.** Two curves stacked would make the radius hard to tune. The curve is the bug click's, `(1 - d / r)^2`, and the node keeps its distance muffling.

**`hearing_radius_metres` lives in the machine's `configure` block.** New machines add data, not systems. 30 m is a starting value for tuning by ear.

**A `SetEmitter` for an output no longer cabled is ignored.** A rack rebuilds its voice whenever a cable changes, so a late update is normal and not an error.

## Risks / Trade-offs

- **No front or back:** a sound directly ahead and directly behind are the same. Accepted until HRTF.
- **The audible point can jump** when the nearest point moves around a stream's bend. Node smoothing (about 60 ms) covers this.
- **A rack rebuild recreates the spatial node:** the voice's existing fade covers it, and it only happens on patching.
- **Rack screen:** the listener stays where the player stands while the rack is open, so patching still sounds like standing beside the machine.
