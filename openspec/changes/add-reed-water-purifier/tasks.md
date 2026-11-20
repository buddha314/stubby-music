Owners follow AGENTS.md: Claude builds art and panels; the engine modules belong to the
audio work stream (`add-rack-and-spatial-audio`) unless the user assigns otherwise.

## 0. Decide (user)
- [x] 0.1 The first campfire recipe is the reed water purifier, patched as the user's VCV Rack sketch (user, 2026-11-18)
- [x] 0.2 VCO frequencies are quantized to notes (user, 2026-11-18)
- [x] 0.3 The clock comes from the stream or burbler (borrowed) (user, 2026-11-18)
- [x] 0.4 Recipe: reeds plus a high-pass filter and an oscilloscope, 96 to 144 BPM; the purifier comes with its modules in place (user, 2026-11-18). Open: reed count
- [x] 0.5 Triangle and saw: fixed VCOs until their sources are designed; cutoff quantized to C5 (user, 2026-11-18)
- [x] 0.7 Four levels of clear (muddy, silty, cloudy, clear), the first filter lifts one; the purifier is a check dam in the stream's course (user, 2026-11-19; design D4, D6)
- [ ] 0.8 Collecting a high-pass filter and an oscilloscope: `add-high-pass-sources` and `add-ripple-pool` (the recipe cannot be made before)
- [x] 0.9 Filters in sequence lift the water further, up to clear (user, 2026-11-19)
- [x] 0.6 The patch is saved as `assets/rack-recipies/reed-water-filter.vcv` (user, 2026-11-18)

## 1. Rack settings
- [x] 1.1 `quantize: "semitone"` on module knobs; `stubby_rack::Knob::snap` to the pitch grid; tests
- [x] 1.2 Note-name readout on quantized knobs (`bevy_stubby_rack`: VCV-style knob readout beside the pointer; whole-click turning; starting values snap, so a bug's pitch lands on its nearest note)
- [x] 1.3 Profile `targets` and a `targets_met` check; move the campfire's `light_pattern` onto it (`configure.targets`, `RackTargets`, a log line naming missed targets when they change)

## 2. Engine modules (audio stream)
- [ ] 2.1 Triangle and saw VCOs (fundsp), module data for `triangle_vco` and `saw_vco`
- [ ] 2.2 Four-channel mixer with levels; module data and panel
- [ ] 2.3 State-variable VCF (LPF and HPF outs, cutoff Hz, resonance); module data and panel
- [ ] 2.4 Scope: pass-through, recent-sample ring to the panel; module data and panel

## 3. Art (Claude)
- [x] 3.1 (claimed: claude; done 2026-11-19) Mixer, VCF and scope panels in the module kit (`assets/modules/`, `tools/blender/create_purifier_modules.py`): panels and `asset.json` only; each `module.json` comes with its engine module (2.2-2.4). Ports: mixer `in_1..in_4`, `mix`; VCF `in`, `lpf`, `hpf`; scope `in`, `out`. Knobs: `level_<n>`; `cutoff` (Hz, log), `res`; `time`, `scale`. The scope's `Screen` node is where the game draws the trace
- [ ] 3.2 Purifier model: a check dam of reeds and sticks across a stream segment's bed, water pooled behind and spilling over (D6); profile with `configure` and `targets`, prefab, LODs, icon, hologram
  - [x] 3.2a (claimed: claude; done 2026-11-19) The dam: `tools/blender/create_stream_features.py`, `assets/machines/reed_water_purifier/`: sticks, stakes and reed bundles packed with mud across the bed and up both banks, `Pond` behind (upstream), `Spill` through a notch and `Foam`; `Interact` (configure) on the right bank, `Outlet` (clarity_lift 1); LOD0/LOD1 and `.meta`; prefab; icon. The hologram is the game's tint over the model (`build.rs`), so no separate asset
  - [ ] 3.2b The profile's `configure` block and `targets`, once the mixer, VCF and scope panels exist (3.1)

## 4. Game
- [ ] 4.1 Recipe row with its BPM range; purifier item; placement (Open Question 5)
- [ ] 4.2 Binder and borrowed clock (reuse the campfire's lending)
- [ ] 4.1a Placement: the dam snaps across an aimed laid stream segment, square to its flow; red elsewhere (D6)
- [ ] 4.3 Cleaning rule: four clarity levels; the stream below a working dam rises one level above what reaches it, so dams in sequence reach clear; easing back when one stops (D4)
- [x] 4.3a Data: `mines.json` leafed at silty, `min_clarity` 0.33 (user, 2026-11-19) (D4)
- [ ] 4.4 Crafting screen at the camp (`add-campfire-craft-bench` 3.3, 2b.3)
- [ ] 4.5 Bounded watchdog review: craft, place, patch, tune, see the water clear
