## Context

Synchronize night sky motion and environment lighting timing. Source: `archived add-sky-motion-and-mountains, add-compass-bar, add-time-speed-controls; autoload/calendar.gd`. Source task completion is evidence of reported status, not a fresh runtime certification. The audit records conflicts and deferred work. The user's direction (2026-11-16): the Godot sky's colour scheme and motion are good, but need not be replicated exactly.

## Parity with Sky3D (1.1)

The Godot game draws its sky with the Sky3D add-on: a physically based atmosphere (Rayleigh and Mie), a sun and a moon disc, a rotating star map and drifting cirrus and cumulus, driven each frame by `Calendar.solar_hour()`. Its moon is moved by the game's own `moon_path.gd`: a fixed point among the stars, turning 15 degrees of hour angle per solar hour. Colours of note: warm horizon light (0.98, 0.64, 0.46), blue-grey night tint (0.17, 0.20, 0.25), orange sun disc, a large moon (size 0.07) and blue moonlight (0.57, 0.78, 0.96).

Before this change the Bevy game moved only the sunlight: the sky was one generated gradient cubemap, baked once, whose brightness was scaled with daylight. No sunrise colour, no moon, no stars.

Still different from Sky3D after this change: no clouds, no star twinkle or Milky Way texture, no moon phases or texture, and no echoed ridges.

## Decisions

Retain the existing engine boundaries and native asset layout. Directly copying Godot implementation was rejected because it changes engine hosting, import semantics and often signal conventions.

**D1. The sky is Bevy's own atmosphere**, not a third-party crate. `bevy_sky_gradient` 0.4 (Bevy 0.19, stars and auroras) runs its own day timer and spawns its own sunlight, conflicting with the calendar, and would need a Bevy-main fork; `bevy_atmosphere` stops at Bevy 0.16; no maintained star crate exists. Bevy's `Atmosphere` (in the pinned Bevy main) scatters every directional light, draws a disc per light (`SunDisk`), and supplies environment lighting (`AtmosphereEnvironmentMapLight`).

**D2. Desert air.** Earth's medium plus a ~3 km layer of dust that scatters neutrally and absorbs blue (`sky::desert_air`): dawn and dusk glow warm, the day sky hazes brown-grey instead of clean blue, as the bleak opening asks.

**D3. One solar mapping** (`calendar::hour_angle`); `horizontal(declination, hour_angle)` places the sun (declination 0), the moon (declination 8 degrees, trailing the sun by 195 degrees of hour angle) and every star; `sky_rotation` turns the star sphere.

**D4. The moon is a second directional light**: 100 lux at full height, no shadows, the Godot moonlight colour, its disc pale by day (`SunDisk` intensity 0.004) and bright by night (0.03).

**D5. Stars are our own seeded mesh**: 1,800 points on a 900 m sphere (inside the far plane, behind the mountains), additive and unlit, faded in by sun altitude.

**D6. Night exposure adapts two stops** (ev100 10.5 by day, 8.5 by night): deep navy sky, readable moonlit ground. The art keys' bleak night goes much further (ev100 -2 with 0.6 lux moon); revisit when the keys drive lighting.

**D7. The camera clears to black.** The atmosphere blends over the cleared background by its transmittance, so Bevy's default grey showed through thin night air.

**D8. Rebake on change.** The environment map is filtered, the generator dropped (the existing bake-once mechanism), and the generator reinserted when the sun or moon has moved a degree: every ~15 s at 1x, a few times a second at 64x.

**D9. Editor preview lights are hidden in Play.** The level's "Desert sun" is `HiddenInGame`, but Jackdaw's embedded Play leaves such entities drawn, so it was a second, fixed 9,500 lux sun (it also doubled daylight in every earlier Play session). The adapter (`stubby_jackdaw`) keeps `HiddenInGame` lights hidden.

## Measurements (2026-11-16, RX 7700S, embedded Play, `--sky` review)

| Pass | ms/frame |
| --- | --- |
| `atmosphere_luts` | ~0.56 (every frame) |
| `render_sky` | ~0.10 |
| `atmosphere_environment` (cubemap draw) | ~0.01 |
| light-probe irradiance + radiance (filtering) | ~1.2, only on a rebake |

So the moving sky costs ~0.67 ms/frame against the static sky's 0, plus a filter on each degree of motion. If that matters on the integrated GPU, the LUT sizes and sample counts in `AtmosphereSettings` are the lever.

## Risks and validation

`tools/jackdaw_timed_test.py --sky` jumps the calendar through dawn (east, and west for the setting moon), noon, dusk, midnight and late night, saving screenshots, light states and per-pass GPU time under the 20-second watchdog. Focused tests cover the solar mapping, moon and star motion, and the lights.
