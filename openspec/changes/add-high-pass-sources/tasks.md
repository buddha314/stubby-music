## 0. Decide (user)
- [x] 0.1 One high-pass filter per rock opening and per hollow trunk; not every trunk is hollow (user, 2026-11-19)
- [x] 0.2 Recur semi-daily (`after` 1800 s); hollows collectable while dormant; collecting silences the source until it returns (user, 2026-11-19)

## 1. Art (Claude)
- [ ] 1.1 Rock formations from the boulder kit, several variants, some with an opening and `Opening` sockets; acoustic rock extras; LODs, colliders, prefabs; previews
- [ ] 1.2 Hollow scrub oak trunk variant with a `Hollow` socket; seeded hollow rate in `assets/flora/scrub_oak/asset.json`
- [ ] 1.3 Item icon `assets/ui/icons/high_pass_filter.png`

## 2. Rules (data)
- [ ] 2.1 `high_pass_filter` item; `rock_opening` and `hollow_trunk` resources yielding one, `replenishment: after 1800`; `rules-fmt`

## 3. Game (Codex)
- [ ] 3.1 Binders: formation and hollow oak sockets become pickups with stable instance IDs (`model_part_bindings`)
- [ ] 3.2 Collect with E through the existing pickup path; the recorder records it
- [ ] 3.3 (audio stream) Sound near an opening or hollow is heard high-passed (D3)
- [ ] 3.4 Seeded placement: formations and hollow oaks (`add-hybrid-world-placement` 2b.4, 2b.5)
- [ ] 3.5 Bounded watchdog review: find an opening, collect its filter
