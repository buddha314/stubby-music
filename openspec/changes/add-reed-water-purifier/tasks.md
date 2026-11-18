Owners follow AGENTS.md: Claude builds art and panels; the engine modules belong to the
audio work stream (`add-rack-and-spatial-audio`) unless the user assigns otherwise.

## 0. Decide (user)
- [x] 0.1 The first campfire recipe is the reed water purifier, patched as the user's VCV Rack sketch (user, 2026-11-18)
- [x] 0.2 VCO frequencies are quantized to notes (user, 2026-11-18)
- [x] 0.3 The clock comes from the stream or burbler (borrowed) (user, 2026-11-18)
- [x] 0.4 Recipe: reeds plus a high-pass filter and an oscilloscope, 96 to 144 BPM; the purifier comes with its modules in place (user, 2026-11-18). Open: reed count
- [x] 0.5 Triangle and saw: fixed VCOs until their sources are designed; cutoff quantized to C5 (user, 2026-11-18)
- [ ] 0.7 How clean (Open Question 4) and placement (5)
- [ ] 0.8 Collecting a high-pass filter and an oscilloscope (their own changes; the recipe cannot be made before)
- [x] 0.6 The patch is saved as `assets/rack-recipies/reed-water-filter.vcv` (user, 2026-11-18)

## 1. Rack settings
- [ ] 1.1 `quantize: "semitone"` on module knobs; `stubby_rack::Knob::snap` to the pitch grid; tests
- [ ] 1.2 Note-name display on quantized knobs (`bevy_stubby_rack`)
- [ ] 1.3 Profile `targets` and a `targets_met` check; move the campfire's `light_pattern` onto it

## 2. Engine modules (audio stream)
- [ ] 2.1 Triangle and saw VCOs (fundsp), module data for `triangle_vco` and `saw_vco`
- [ ] 2.2 Four-channel mixer with levels; module data and panel
- [ ] 2.3 State-variable VCF (LPF and HPF outs, cutoff Hz, resonance); module data and panel
- [ ] 2.4 Scope: pass-through, recent-sample ring to the panel; module data and panel

## 3. Art (Claude)
- [ ] 3.1 Mixer, VCF and scope panels in the module kit (`assets/modules/`)
- [ ] 3.2 Purifier model (a reed filter bed in a stream), profile with `configure` and `targets`, prefab, LODs, icon, hologram

## 4. Game
- [ ] 4.1 Recipe row with its BPM range; purifier item; placement (Open Question 5)
- [ ] 4.2 Binder and borrowed clock (reuse the campfire's lending)
- [ ] 4.3 Cleaning rule: clarity downstream while targets are met
- [ ] 4.4 Crafting screen at the camp (`add-campfire-craft-bench` 3.3, 2b.3)
- [ ] 4.5 Bounded watchdog review: craft, place, patch, tune, see the water clear
