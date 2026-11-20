Owners follow AGENTS.md: Claude builds art and panels; the engine modules belong to the
audio work stream (`add-rack-and-spatial-audio`) unless the user assigns otherwise.

## 0. Decide (user)
- [x] 0.1 The first campfire recipe is the reed water purifier, patched as the user's VCV Rack sketch (user, 2026-11-18)
- [x] 0.2 VCO frequencies are quantized to notes (user, 2026-11-18)
- [x] 0.3 The clock comes from the stream or burbler (borrowed) (user, 2026-11-18)
- [x] 0.4 Recipe: reeds plus a high-pass filter and an oscilloscope, 96 to 144 BPM; the purifier comes with its modules in place (user, 2026-11-18). Open: reed count
- [x] 0.5 Triangle and saw: fixed VCOs until their sources are designed; cutoff quantized to C5 (user, 2026-11-18)
- [x] 0.7 Four levels of clear (muddy, silty, cloudy, clear), the first filter lifts one; the purifier is a check dam in the stream's course (user, 2026-11-19; design D4, D6)
- [x] 0.8 (done 2026-11-20, #257) Collecting a high-pass filter and an oscilloscope: `add-high-pass-sources` and `add-ripple-pool` (the recipe cannot be made before)
- [x] 0.9 Filters in sequence lift the water further, up to clear (user, 2026-11-19)
- [x] 0.6 The patch is saved as `assets/rack-recipies/reed-water-filter.vcv` (user, 2026-11-18)

## 1. Rack settings
- [x] 1.1 `quantize: "semitone"` on module knobs; `stubby_rack::Knob::snap` to the pitch grid; tests
- [x] 1.2 Note-name readout on quantized knobs (`bevy_stubby_rack`: VCV-style knob readout beside the pointer; whole-click turning; starting values snap, so a bug's pitch lands on its nearest note)
- [x] 1.3 Profile `targets` and a `targets_met` check; move the campfire's `light_pattern` onto it (`configure.targets`, `RackTargets`, a log line naming missed targets when they change)

## 2. Engine modules (audio stream)
- [x] 2.1 (claimed: Codex) Triangle and saw VCOs (fundsp), module data for `triangle_vco` and `saw_vco`
- [x] 2.2 (claimed: Codex) Four-channel mixer with levels; module data and panel
- [x] 2.3 (claimed: Codex) State-variable VCF (LPF and HPF outs, cutoff Hz, resonance); module data and panel
- [x] 2.4 (claimed: Codex) Scope: pass-through, recent-sample ring to the panel; module data and panel

## 3. Art (Claude)
- [x] 3.1 (claimed: claude; done 2026-11-19) Mixer, VCF and scope panels in the module kit (`assets/modules/`, `tools/blender/create_purifier_modules.py`): panels and `asset.json` only; each `module.json` comes with its engine module (2.2-2.4). Ports: mixer `in_1..in_4`, `mix`; VCF `in`, `lpf`, `hpf`; scope `in`, `out`. Knobs: `level_<n>`; `cutoff` (Hz, log), `res`; `time`, `scale`. The scope's `Screen` node is where the game draws the trace
- [ ] 3.2 Purifier model: a check dam of reeds and sticks across a stream segment's bed, water pooled behind and spilling over (D6); profile with `configure` and `targets`, prefab, LODs, icon, hologram
  - [x] 3.2a (claimed: claude; done 2026-11-19) The dam: `tools/blender/create_stream_features.py`, `assets/machines/reed_water_purifier/`: sticks, stakes and reed bundles packed with mud across the bed and up both banks, `Pond` behind (upstream), `Spill` through a notch and `Foam`; `Interact` (configure) on the right bank, `Outlet` (clarity_lift 1); LOD0/LOD1 and `.meta`; prefab; icon. The hologram is the game's tint over the model (`build.rs`), so no separate asset
  - [x] 3.2b (claimed: Codex) Purifier rack profile and C2/C4/C5 targets in `machines.json`

## 4. Game
- [x] 4.1 (claimed: claude; done 2026-11-20) Recipe row with its BPM range; purifier item; placement (Open Question 5). `craft_reed_water_purifier` (20 reeds, a high-pass filter, an oscilloscope; 96-144 BPM) makes a `reed_water_purifier_kit` (IDs are unique across rules: `reed_water_purifier` is the machine); the `reed_water_purifier` buildable costs one kit. Reed count 20 is the draft
- [x] 4.2 (claimed: Codex) Binder and borrowed clock (reuse the campfire's lending)
- [x] 4.1a (claimed: claude; done 2026-11-20) Placement: the dam snaps across an aimed laid stream segment, square to its flow; red elsewhere (D6). Generic: buildables now say `model`, `placement` (`ground`, `beside_stream`, `across_segment`, `laid`) and an optional `built_hint`, and one placement mode in `build.rs` serves them all (the campfire moved onto it). A dam records its segment (`OnSegment`, keyed by `LaidStream::segment_key`), follows it when dismantling elsewhere respawns the stream, and goes with it, its build cost returned, when its own segment is dismantled
- [x] 4.3 (claimed: Codex; review fixes by claude 2026-11-20: a dam keeps the segment it stands on instead of re-searching every frame, and lifts ease out when the water stops instead of vanishing) Cleaning rule: four clarity levels; the stream below a working dam rises one level above what reaches it, so dams in sequence reach clear; easing back when one stops (D4)
- [x] 4.3a Data: `mines.json` leafed at silty, `min_clarity` 0.33 (user, 2026-11-19) (D4)
- [x] 4.4 (claimed: claude; done 2026-11-20) Crafting screen at the camp (`add-campfire-craft-bench` 3.3, 2b.3)
- [x] 4.5 (claimed: claude; done 2026-11-20) Bounded watchdog review: craft, place, patch, tune, see the water clear. `tools/jackdaw_timed_test.py --standalone --purifier-e2e` plays the loop itself (`--purifier-e2e=<dir>`): tempo 120 BPM, craft the kit at that tempo, build it through the build menu's placement across the laid stream, patch and tune, then check the water below clears (silty, 0.30, in 0.9 s) while upstream stays muddy, and that the stream's ripple pools keep its beat; `events.jsonl` ends `complete` with `success: true`; screenshots `1-built`, `2-clearing`, `3-pool`. The crafting screen itself and real input remain the live review (4.5b)
- [x] 4.5a (claimed: Codex) Seed-42 review setup with a patched burbler, laid stream, unpatched purifier, and six available cables. Launch from the feature worktree with `python3 tools/jackdaw_timed_test.py --standalone --purifier-review --name purifier`.
- [x] 4.6 (claimed: claude; done 2026-11-20) Review leftovers from #258: the lift time is a rule (`machines.json` `clarity_lift_s`, validated 0 to 600 s; 1 s for the purifier); a render test proves the high-pass (C2 36 dB weaker than through the LPF, C6 about 15 times stronger); the review driver gives up after 15 s with a reason. The 0.75 m search for a dam's segment stays a constant: it only attaches dams placed by a review or a level, since a built dam knows its segment. The clarity palette was brightened (silty `#8a7049`) after the e2e review measured the first lift at only +11 to +17 levels; now +21 to +31
- [ ] 4.5b Live review of E targeting, cable guidance, tuning, scope controls and the upstream/downstream water boundary.
