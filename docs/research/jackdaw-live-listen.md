# Live listen: audio-engine task 5.5

## Implementation

Use `--module-demo` in the game, or the Jackdaw configuration **Live listen:
clock + square**. This patches the actual placed 120 BPM clock and 110Hz square
VCO through an attack/decay envelope and VCA. One clock ID drives the lamp and
the sound, so equal tempo settings on two different clocks cannot disguise drift.
No independent F8 demo should be running during this check.

No new DSP: the patch uses existing Clock, SquareVco, Envelope and Vca modules.
The two auxiliary nodes are cleaned up if either source module is removed.
Normal launch remains unpatched and silent at spawn.

## Verification (2026-10-31)

- Worktree `/tmp/stubby-music-live-listen`, branch `add-jackdaw-live-listen`.
- Dedicated game and adapter target directories under
  `/home/buddha/.cache/stubby-music-live-listen/`; no shared target with another
  agent or the live editor checkout.
- Game tests: 31 pass, including sample-level audio onset/quiet intervals and
  audition activation/cleanup. Workspace Clippy passes with warnings denied.
- Standalone screenshot: `/tmp/stubby-live-listen-game.png`.
- Standalone sink capture: `/tmp/stubby-live-listen-game.wav`, 8 seconds, peak
  0.1178, 15 detected bursts with every complete interval 0.500 seconds.
- User confirmed hearing the standalone square-wave sound.
- Jackdaw adapter built successfully against the shared source checkout.
- Jackdaw Play connected in the isolated editor (remote port 15713). Output capture
  `/tmp/stubby-live-listen-jackdaw.wav`: 8 seconds, peak 0.1178, 16 bursts with
  every complete interval 0.500 seconds.
- Editor screenshot: `jackdaw/.jackdaw/live-listen.png` (ignored generated evidence).
- Stop returned the editor to authoring; `/tmp/stubby-live-listen-stopped.wav`
  contains 2 seconds of digital silence (peak 0.0).
- Replay connected again and `/tmp/stubby-live-listen-replay.wav` contains
  8 seconds, peak 0.1178, 16 bursts with every complete interval 0.500 seconds.
- Workspace and Jackdaw adapter Clippy pass with warnings denied; both changed
  OpenSpec proposals validate strictly; formatting passes.
- Verification temporarily made the live-listen run config first so the editor's
  Play operator could select it. Original run order was restored afterward;
  the normal Desert opening remains the default.

## Display regression after listening check

The user reported wild whole-display flashing during Jackdaw Play, then recurrence
after Stop. All Jackdaw and game processes were confirmed exited. This means the
sound/capture checks do not establish stable interactive operation; task 5.5 and
module task 2.5 are reopened. Do not relaunch the graphical test automatically.

No GPU timeout/fault/reset error was observed in the inspected kernel logs. AMD
GPU runtime-resume messages and a Blender context cleanup warning were present;
these alone do not identify a cause. Mutter experimental features were empty.
The internal display was running fixed 2560x1600 at 165Hz. A temporary fixed 60Hz
mode was applied, with the original display state saved in
`/tmp/stubby-flicker-display-before.json`. The test editor was terminated and no
Jackdaw/game/Blender processes remained at verification. The original editor
session had already exited separately. Changes remain uncommitted.

This listening check does not close the separate on-screen peak-within-10ms
measurement (module-asset-binding task 2.4). Frame screenshot review alone cannot
establish that timing tolerance.

Recovery follow-up: fixed 60Hz was confirmed active, then display output was
briefly powered off/on via Mutter PowerSaveMode (finally restored to On).
This did not restart the desktop or close unrelated applications. Visual bell,
ambient brightness adaptation, and night light were all disabled already.
No compositor warning was present in the inspected recent user logs. Physical
flicker resolution has not been confirmed; do not report a proven rendering fix
or mark the live UI test complete from these diagnostic readings alone.

## Bounded follow-up tests

User authorized further graphical investigation with a 20-second timer.
`tools/jackdaw_timed_test.py` uses GNU timeout SIGKILL outside the render process,
then restores fixed 60Hz and resets the output automatically. It does not target
existing editors. Short smoke test killed a 60-second sleep at the configured
half-second limit; requests above 20 seconds are rejected.

- `editor-60`: editor only, fixed 60Hz, external 20-second kill observed. Controller
  elapsed 20.092 seconds (polling after the external kill); editor reported stopped
  throughout, no game launched. Report `/tmp/jackdaw-flicker-editor-60.json` and log
  `/tmp/jackdaw-flicker-editor-60.log`. No render fault appeared in the log.

Absence of a renderer error or user reply is not physical flicker confirmation.
The 165Hz mode is not a proven cause yet.

- `play-60`: Play connected and ran within the 20-second test; watchdog killed it
  and restored fixed 60Hz. User reported **no flicker**. The one brief black screen
  at test closure was the diagnostic's deliberate display power-cycle, not a
  reproduced rendering fault. Default 60Hz tests now skip that power-cycle;
  higher-refresh tests retain it for recovery. Report
  `/tmp/jackdaw-flicker-play-60.json`, log `/tmp/jackdaw-flicker-play-60.log`.

- `editor-165` and `play-165`: both completed under the 20-second watchdog and
  recovered fixed 60Hz. Play connected normally. No render panic or GPU fault
  was present in the application logs. Reports are under
  `/tmp/jackdaw-flicker-{editor,play}-165.json`. Physical flicker observations for
  these two tests are not yet confirmed; do not treat silent feedback as a pass.

Hardware identification: Framework Laptop 16 (AMD Ryzen 7040 Series), integrated
780M and discrete RX 7700S. Current amdgpu dcdebugmask is 0. Boot options include
`amdgpu.sg_display=0` and `amdgpu.dcfeaturemask=0x8`. No boot parameters changed.
Framework publishes a directly relevant PSR/internal-display-flicker workaround:
https://knowledgebase.frame.work/psr-display-flicker-workaround-for-linux-SJTKb7VMl
This is a possible display-driver cause, not a confirmed diagnosis. Keep fixed
60Hz as the user-confirmed stable workaround while investigating.

The kernel's DC_FEATURE_MASK enum identifies **0x8 as DC_PSR_MASK**, so the
existing boot argument explicitly enables PSR. See
https://dri.freedesktop.org/docs/drm/gpu/amdgpu/driver-core.html
Framework's matching-hardware guidance recommends disabling PSR and Panel Replay
with `amdgpu.dcdebugmask=0x410` as a boot workaround. Current dcdebugmask is 0;
both module parameters are read-only (0444). This makes PSR a plausible cause,
not a demonstrated root cause. The vendor workaround needs reboot validation;
no grub, kernel arguments or driver files were changed. Root-only debugfs panel
state could not be read without a password, so that inspection was not performed.

## Reboot handoff

Before reboot the worktree is moved from `/tmp/stubby-music-live-listen` to
`/home/buddha/src/stubby-music-live-listen`. Timed-test JSON/log files, audio
captures, original display state and standalone screenshot are preserved in
`/home/buddha/.cache/stubby-music-live-listen/evidence/` (local, not versioned).
Historical `/tmp` paths above describe the actual test runs; use the evidence
folder after reboot. Build outputs remain in dedicated on-disk cache directories.
Update before reboot: the **user reports grubby was applied** with removal of
`amdgpu.dcfeaturemask=0x8` and addition of `amdgpu.dcdebugmask=0x410`. The running
kernel has not been reboot-verified yet. On resume check
`cat /sys/module/amdgpu/parameters/dcdebugmask` → expected `1040`, and inspect the
AMD options in `/proc/cmdline`. Do not reapply boot changes automatically.

Rebuild the game/adapter before running from the moved worktree. Continue only
with 20-second watchdog tests, first 60Hz then a bounded 165Hz comparison that
restores 60Hz. Keep the flicker regression open until stable Play is validated.
Earlier statements that no boot settings changed describe the automated
investigation, before the user's manual grubby application.
