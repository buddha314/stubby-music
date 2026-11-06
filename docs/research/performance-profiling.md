# Performance profiling (2026-11-01)

First profiling pass of the desert opening, to find out why embedded Play ran at
~10 fps and how much headroom the game itself has. Raw numbers are kept here for
later analysis; reproduce with the commands at the end.

## Setup

- Laptop: AMD Ryzen 9 7940HS (16 threads), discrete **AMD Radeon RX 7700S** (RADV
  NAVI33, Mesa 26.2.5) and integrated Radeon 780M (RADV PHOENIX); Fedora 44,
  kernel 7.2.11; internal panel 2560x1600.
- Game: Bevy 0.20.0, Rust 1.99.0, dev profile (our crates `opt-level = 1`,
  dependencies `opt-level = 3`), the authored level `assets/scene.bsn`,
  **~1,471 entities**, driven by the dev-only prefab review (walk, collect, open
  and patch the burbler rack). The game picks the discrete GPU by default.
- Tools (all built into Bevy, no custom instrumentation):
  `bevy/trace_chrome` (per-system CPU spans, summarised by
  `tools/profiling/trace_summary.py`), `RenderDiagnosticsPlugin` (GPU and encode
  time per render pass), `FrameTimeDiagnosticsPlugin`, `EntityCountDiagnosticsPlugin`,
  `LogDiagnosticsPlugin`. Frame-rate samples also come from the review's `fps`
  events. Graphical runs used the 20-second watchdog.

## Results

| Run | Frame size | fps | Frame time |
| --- | --- | --- | --- |
| Embedded Play, Jackdaw before fork fix | 1472x950 | ~9-13 | |
| Embedded Play, fork `f2a1cf20` (incremental PIE state streaming) | 1472x950 | ~35-46 | |
| Standalone, vsync on | 2133x1200 | 60 (vsync cap) | |
| Standalone, vsync off | 2133x1200 | ~63-70 | |
| Standalone, vsync off, tracing on | 2133x1200 | ~57 | 17.7 ms |
| Same, integrated Radeon 780M (`WGPU_ADAPTER_NAME=780M`) | 2133x1200 | ~36 | 27.7 ms |
| **Standalone, vsync off, tracing on, dependencies without debug assertions** | 2133x1200 | **~166** | **6.0 ms** |
| **Embedded Play, fork fix + dependencies without debug assertions** | 1472x950 | **~50-60** (60 Hz runner cap) | |

Removing the embedded-Play frame readback (a three-second A/B in one session) changed
nothing, before or after the fork fix: frame capture is not a cost.

### Dev build, tracing on (discrete GPU), ms per frame over 10 s steady state

| Where | ms/frame |
| --- | --- |
| Whole frame (`update`) | 17.5 |
| Render thread `Render` schedule (the bottleneck) | 16.8 |
| ... `submit_pending_command_buffers` | 9.8 |
| ... `queue_submit` (19 command buffers) | 7.2 |
| ... `finish_command_buffers` | 2.5 |
| Main thread `main app` (real work) | 4.9 |
| ... `PostUpdate` | 1.9 |
| ... `FixedMain` (incl. avian3d physics 0.6) | 1.2 |
| ... `Update` | 0.9 |
| Main thread `RenderExtractApp` (mostly waiting for the render thread) | 12.6 |
| `prepare_generated_environment_map_bind_groups` (pool) | 1.5 |

GPU time per frame (RenderDiagnostics, all passes): **6.2 ms** - main opaque 3.15,
shadows 0.87, light-probe irradiance 0.80, light-probe radiance 0.51, bloom 0.43,
tonemapping 0.10, the rest < 0.1 each. CPU encode time 0.2 ms. The GPU was not
saturated: the render thread was waiting in `queue_submit`.

### Dependencies without debug assertions, tracing on, ms per frame

| Where | ms/frame |
| --- | --- |
| Whole frame | 6.0 |
| Render thread `Render` schedule | 5.5 |
| ... `submit_pending_command_buffers` / `queue_submit` | 1.5 / 1.2 |
| Main thread `main app` | 3.7 |
| ... `PostUpdate` / `Update` / `FixedPostUpdate` | 1.7 / 0.7 / 0.4 |
| ... avian3d physics | 0.24 |
| `prepare_generated_environment_map_bind_groups` | 0.2 |

## Findings

1. **Embedded Play's ~10 fps was the editor link**, not the game: the game-side PIE
   state stream reflect-serialized every entity's every component each frame.
   Fixed in the fork (buddha314/jackdaw#4, pinned at `f2a1cf20`).
2. **Debug assertions in dependencies cost ~3x frame time** in dev builds (17.7 ms
   to 6.0 ms; `queue_submit` 7.2 ms to 1.2 ms: wgpu and Bevy debug-only checks).
   Players' release builds would not pay it, but every dev build and editor Play
   did. Turning them off for dependencies only (`[profile.dev.package."*"]
   debug-assertions = false, overflow-checks = false`) keeps them in our own crates,
   where `stubby_audio`'s `assert_no_alloc` relies on them.
3. **The game is CPU-light**: ~3.7 ms main-thread work with nothing over 1 ms in our
   own systems; physics 0.24 ms.
4. **The GPU load is modest on the discrete GPU** (6.2 ms) but **the integrated 780M
   runs at ~36 fps**: on weaker GPUs the main opaque pass (terrain, fog, materials)
   and shadows will dominate.
5. **The generated environment map is rebuilt every frame** (~1.3 ms GPU for
   irradiance and radiance, plus bind-group preparation on the CPU). Unless the sky
   changes every frame it could regenerate only when the light changes (day/night),
   or less often.
6. **Embedded Play is capped at 60 fps** by the editor's windowless schedule runner
   (`ScheduleRunnerPlugin::run_loop(1/60)` in `jackdaw_runtime/src/pie_windowless.rs`).

## Integrated GPU (Radeon 780M), 2026-11-02

Standalone `--prefab-review`, 20 s, on battery, after #86 (no dependency assertions)
and #89 (sky baked once). Frame time was too noisy on battery to compare (18-56 ms for
identical runs), so compare GPU pass times, which repeated within ~0.1 ms for shadows
and ~1 ms for the opaque pass.

| Configuration | Opaque | Shadows | GPU total |
| ------------- | -----: | ------: | --------: |
| Ground as one mesh (before) | 7.0-8.6 | 6.1-6.2 | ~18 |
| **Ground in 12x12-plate chunks** (shipped) | 5.8-6.7 | 3.2 | ~13 |
| **Chunks + 2 cascades to 120 m** (shipped) | 6.3 | 1.6 | ~11.5 |
| Chunks, ground casts no shadow | 8.0 | 1.1 | - |

Other passes are fixed costs: bloom 0.7, upscaling 0.4, tonemapping 0.3 ms.

7. **The ground was one ~430k-triangle mesh** (120x120 plates, ~30 triangles each), so
   nothing culled it: it was drawn whole into the main view, behind the camera
   included, and into all four shadow cascades. Chunked into 100 meshes it is culled
   per view and per cascade: shadows halve, opaque drops ~1 ms. Plates are generated
   per cell, so the chunked ground is identical (screenshot diff: 0.025% of pixels,
   the flashing bug and the sand).
8. **Two shadow cascades** (to 120 m instead of four to 150 m) halve shadows again and
   looked the same in a daytime shot. Shipped; it trades resolution in long
   dawn/dusk shadows, so raise the count if those look too soft.
9. **The rest of the opaque pass** is the ground's vertex load plus fill at 2133x1200.
   Next levers if needed: a coarser far LOD for distant chunks (plates beyond ~40 m
   as flat polygons, cracks dropped), or a lower render scale on integrated GPUs.

## Candidates for later

- Apply finding 2 (a two-line `Cargo.toml` profile change).
- ~~Environment-map regeneration on change only (finding 5).~~ Done in #89: baked once.
- A Tracy session (`bevy/trace_tracy`) for interactive drill-down if a frame spike
  needs explaining; the Chrome trace of 20 s is ~2 GB.

## Reproduce

Temporary edits, not committed: in `crates/stubby_jackdaw/src/main.rs` add
`FrameTimeDiagnosticsPlugin`, `RenderDiagnosticsPlugin`,
`EntityCountDiagnosticsPlugin` and `LogDiagnosticsPlugin`, and set the primary
window's `present_mode` to `AutoNoVsync`. Then:

```bash
cargo +1.99.0 build --bin stubby_jackdaw --manifest-path crates/stubby_jackdaw/Cargo.toml --features bevy/trace_chrome
python3 tools/jackdaw_timed_test.py --seconds 20 --name profile --command \
  bash -c "cd jackdaw && ../target/debug/stubby_jackdaw --prefab-review=/tmp/stubby-standalone"
# diagnostics: /tmp/jackdaw-flicker-profile.log ; trace: trace-*.json
python3 tools/profiling/trace_summary.py trace-*.json 8 18   # 8-18 s window
```

Add `WGPU_ADAPTER_NAME=780M` before the binary for the integrated GPU, and
`CARGO_PROFILE_DEV_DEBUG_ASSERTIONS=false CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false`
before `cargo build` for the no-assertions build. Delete the trace afterwards.
