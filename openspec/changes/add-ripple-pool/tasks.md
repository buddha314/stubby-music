## 0. Decide (user)
- [x] 0.1 The oscilloscope comes from a new ripple pool (user, 2026-11-19)
- [ ] 0.2 Open Questions 1 (one per pool?) and 2 (does it water?)

## 1. Art (Claude)
- [ ] 1.1 Ripple pool model: a shallow hollow of clear still water with a rim; rings as a material Bevy can render; `Interact` socket; LODs, prefab, previews
- [ ] 1.2 Item icon `assets/ui/icons/oscilloscope.png`

## 2. Rules (data)
- [ ] 2.1 `oscilloscope` item; `ripple_pool` resource yielding one; `rules-fmt`

## 3. Game (Codex)
- [ ] 3.1 Binder and pickup through the existing E path; the pool keeps rippling after (D3)
- [ ] 3.2 Ripples driven by sound reaching the pool on the audio clock (D2; fallback: beat pulses)
- [ ] 3.3 Seeded placement in the wetness channel (`add-hybrid-world-placement` 2b.3)
- [ ] 3.4 Bounded watchdog review: a pool rippling to a burbler; collect its oscilloscope
