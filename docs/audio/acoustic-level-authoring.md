# Acoustic level authoring and review

Read this before adding or changing a level, prefab, or model that could block or
shape a sound path. The editor-authored component is
`stubby_core::AcousticOccluder`; its data is copied to AudioNimbus's scene by
`stubby_game::glue::publish_acoustic_scene`.

For engine work before production caves or buildings exist, see
[`pre-level-validation.md`](pre-level-validation.md). Synthetic fixtures can validate
the bridge and simulation, but do not replace authored-level listening reviews.

## What the component controls

Add `AcousticOccluder` to the entity that owns the relevant `Mesh3d` primitive.
Adding it to a prefab root does not mark its child meshes. Jackdaw exposes these
fields in the inspector; `Default` is the generic material.

Each marked mesh has one acoustic material shared by all its triangles:

| Field | Range | Meaning |
| --- | --- | --- |
| `absorption[3]` | 0–1 per band | Fraction of sound energy absorbed. Higher values make reflected sound weaker. |
| `scattering` | 0–1 | How diffuse the reflected sound is: 0 is mirror-like; 1 scatters in all directions. |
| `transmission[3]` | 0–1 per band | How much sound passes through the surface. 0 blocks transmission; 1 passes it all. |

For this Steam Audio API, the three bands are centered at 400 Hz, 2.5 kHz, and
15 kHz. The component values describe acoustics, not visual PBR roughness, base
color, opacity, or metalness. A value of 0.8 absorption means substantial energy
loss; it does not mean “80% quieter” at the listener.

Steam Audio material values are a good FOSS starting point. The checked-in constants
`AcousticOccluder::GENERIC` and `::ROCK` match the SDK's generic and rock presets.
Other useful reference presets are:

| Surface | Absorption [low, mid, high] | Scattering | Transmission [low, mid, high] |
| --- | --- | --- | --- |
| Concrete | [0.05, 0.07, 0.08] | 0.05 | [0.015, 0.002, 0.001] |
| Brick | [0.03, 0.04, 0.07] | 0.05 | [0.015, 0.015, 0.015] |
| Rock | [0.13, 0.20, 0.24] | 0.05 | [0.015, 0.002, 0.001] |
| Wood | [0.11, 0.07, 0.06] | 0.05 | [0.070, 0.014, 0.005] |
| Glass | [0.06, 0.03, 0.02] | 0.05 | [0.060, 0.044, 0.011] |
| Metal | [0.20, 0.07, 0.06] | 0.05 | [0.200, 0.025, 0.010] |
| Carpet | [0.24, 0.69, 0.73] | 0.05 | [0.020, 0.005, 0.003] |
| Gravel | [0.60, 0.70, 0.80] | 0.05 | [0.031, 0.012, 0.008] |

These are starting estimates, not measurements of the game's exact art materials.
Use the nearest physical surface class and adjust only when a listening review
shows a clear reason. The [Steam Audio material guide](https://valvesoftware.github.io/steam-audio/doc/unreal/material.html)
defines the properties; the [engine integration guide](https://github.com/ValveSoftware/steam-audio/blob/master/core/doc/integration.rst)
recommends explicit geometry and material authoring controls.

### Marking a model by glTF extras

A model can opt in from its source instead of the editor: give a mesh node the glTF
extras `"acoustic": "rock"` in its Blender generator, and the game marks the meshes under
that node with `AcousticOccluder::ROCK` when they spawn (`add-echo-grotto`; the boulders
of `tools/blender/create_boulders.py` do this). Only the near level carries the extras.

## What to mark

Mark geometry that changes an important path between sound sources and the player:

- Room, cave, tunnel, and building floors, walls, ceilings, and large openings.
- Large terrain or rock faces that separate play spaces or strongly screen a source.
- Large props or dense groups of props only when they materially change audibility.

Leave small clutter, isolated leaves, grass, decorative trim, and tiny props unmarked
unless a level review demonstrates a meaningful acoustic effect. More triangles are
not automatically better: every tagged mesh enters the ray-traced acoustic scene.

The marker must be on a static `Mesh3d` triangle-list entity. The current bridge
assigns one material to every triangle in that mesh. If one primitive combines
surfaces with different acoustic behavior, split it into material-specific mesh
primitives in the source/export pipeline. Do not duplicate a visible mesh just to
tag it acoustically.

Keep room boundaries coherent: avoid accidental holes, gaps, overlapping duplicate
faces, and inconsistent triangle winding. A cave needs a floor, ceiling, walls, and
openings represented by its acoustic mesh; a visual shell alone does not affect
sound. Moving or animated geometry is not supported as a low-cost static scene at
present. Do not mark animated foliage, creatures, or moving machinery.

## Level audit checklist

For each changed sound-critical area, review the saved level/prefab in Jackdaw and
record the following in the PR description:

1. **Sources and listener:** list the representative source and player/listener
   positions, including a point inside and outside each enclosed space.
2. **Geometry coverage:** identify which `Mesh3d` children carry
   `AcousticOccluder`; confirm that floors, walls, ceilings, and openings match the
   level's intended sound boundaries.
3. **Material rationale:** state the chosen surface preset or values and why they
   fit the surface. Check transmission as well as absorption for walls that should
   let some sound through.
4. **Path checks:** compare a clear outdoor path, a source behind the boundary, a
   nearby doorway/opening, and (for a cave) a source/listener pair inside the same
   enclosure. Listen for direct level/frequency changes and verify the sound
   recovers when the path clears.
5. **Scope:** note any significant geometry intentionally left untagged and why.

Use the same sound source, gain, listener position, and distance for material A/B
listening. `--audio-demo` includes a deliberately sustained 110 Hz square wave, so
its low hum is expected; use its steady source for broad occlusion comparisons and
listen for the clock blips or a dedicated test patch for transient detail.

## Current implementation limits

The bridge snapshots marked meshes into a world-space AudioNimbus scene off the
audio callback. The generated mudflat and mountain ridges are tagged in code. The
Jackdaw component is reflectable for authored mesh children. Authored walls/caves
still need tags and a successful in-game occlusion/transmission listening review.
The AudioNimbus reflection API has an offline synthetic-room test, but there is no
live reflection/reverb bus in the Firewheel graph yet. Animated acoustic geometry
and per-triangle material indices are also not implemented; do not describe them as
supported in a level review.

## Opening boulder grotto review (2026-11-17)

`tools/jackdaw_timed_test.py --standalone --grotto-audio-review --name <name>` runs a
20-second bounded standalone pass. It plays the square/clock demo at the grotto
burbler, moves the listener to an interior/mouth point and two outside points, and
records listener/source positions, world-bus peak levels, and screenshots under
`/tmp/grotto-audio-review/`. The outside comparison points are both 17 m from the
source: one ray passes through boulder 0; the other is on the spawn-facing side.

The scene snapshot contained 1,302,514 vertices, 434,172 triangles, and 104 material
records. The test did **not** pass the occlusion check: the steady world-bus peak was
−33.71 dBFS at both matched-distance outside points. At the blocked point, a direct
simulation diagnostic also returned `occlusion=1` and unity transmission for the
square demo emitter. The interior/mouth point measured −23.64 dBFS at about 5.5 m,
so that level difference is explained by distance and does not establish occlusion.
A visible boulder lies across the blocked camera ray, so investigate why this
source's ray does not register the tagged rock before treating the grotto as
acoustically validated.

Live echo/reverb could not be evaluated: the Firewheel graph does not yet consume
AudioNimbus reflection outputs. Keep `add-echo-grotto` task 4.1 open until the direct
occlusion path is corrected and the live reflection/reverb bus is integrated and
heard in the grotto.
