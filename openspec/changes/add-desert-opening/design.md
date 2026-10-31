## Context

The opening has to establish tone (silence, storm, isolation) and teach the core idea
(sound is something you find and build) with only two objects. It depends on
`add-audio-engine` for the modules and audio-clock sync.

## Goals / Non-Goals

**Goals:**
- Seeded, streamed mudflat that looks hand-made at ground level.
- A storm that hides and reveals, so the player navigates by silhouette and light.
- Mood progression infrastructure that later content can push on.
- A repeatable Blender → Bevy pipeline.

**Non-Goals:**
- Factory building, patch cables, inventory UI.
- Biomes beyond the mudflat; day/night cycle.
- Wind *audio*. The storm is silent by design until the player brings sound.

## Decisions

**Terrain: Voronoi cracks on a near-flat heightfield.** Worley/Voronoi cells give the
polygon crack pattern; cell edges become shallow trenches, and cell centres get a
slight concave curl. Large-scale undulation comes from low-amplitude fBm (`noise`
crate). Fine crack detail goes into a normal map or shader so mesh density stays
modest. *Alternative:* modelled tiles from Blender. Rejected because they repeat
visibly and do not stream.

**Seeding: `rand_chacha` per chunk, keyed by `(seed, cx, cz)`.** This is stable
across platforms and Rust versions, unlike `StdRng`. Generation runs on
`AsyncComputeTaskPool`.

**Sand: `bevy_hanabi` GPU particles in a volume that follows the camera, plus
volumetric fog.** Particles near the camera sell speed; fog sells distance. Both read
`StormIntensity`, a seeded gust curve. *Fallback* if `bevy_hanabi` lags Bevy 0.20:
CPU instanced billboards in a camera-local volume.

**Mood: one `Resource<Mood>`, one place it is read.** An atmosphere system maps
`Mood` to colour grading, fog, light colour/intensity, and storm scale through
authored curves, eased with critical damping. Discoveries add to a target, never set
the value directly.

**Clock light timing.** A point light plus emissive mesh. Brightness is an envelope
triggered at the display time predicted from the audio `ClockTick.sample_time` (see
`add-audio-engine` D6), not on event receipt.

**Square found by ear.** *(Changed 2026-10-31 from a sine found by sight.)* A pure
sine is the hardest sound to locate by ear, so the first discovery is a band-limited
square, kept under 1 kHz. It emits, spatialized, within a 35 m hearing radius that
spawn is always outside, so the game opens silent and the player follows the square
once they catch it. A square-wave ripple in the sand is the visual fallback. See
`docs/research/rack-and-spatial-audio.md`.

## Risks / Trade-offs

- [Volumetric fog cost on laptop GPUs] → scale quality by preset; test on `breq`.
- [bevy_hanabi version lag] → CPU fallback above; keep the sand behind one plugin.
- [Players get lost in zero visibility] → gust troughs are guaranteed every 20–40 s,
  and the square is placed within 120 m.
- [Mood curves need lots of tuning] → expose them as an asset file, hot-reloaded.

## Open Questions

- Player representation: first-person (Inside-like intimacy is lost) or third-person
  side-ish camera (closer to Inside)? This affects the camera, storm volume, and
  controls. Needs a decision before task 4.
- Whether `.blend` files go in Git LFS.
