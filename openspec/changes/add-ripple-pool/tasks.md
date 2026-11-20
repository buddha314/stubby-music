## 0. Decide (user)
- [x] 0.1 The oscilloscope comes from ripple pools in streams, Poisson along the stream, rippling to the beat, refreshing semi-daily (user, 2026-11-19)
- [ ] 0.2 Open Questions 1 (spacing) and 2 (wild water)

## 1. Art (Claude)
- [ ] 1.1 Ripple pool model: an eddy widening a stream segment's bed, still water, `Interact` socket; LODs, prefab, previews
- [ ] 1.2 One-beat ring ripple clip (baked glTF animation, as the burbler's bubbles)
- [ ] 1.3 Item icon `assets/ui/icons/oscilloscope.png`

## 2. Rules (data)
- [ ] 2.1 `oscilloscope` item; `ripple_pool` resource yielding one, `replenishment: after 1800`; `rules-fmt`

## 3. Game (Codex)
- [ ] 3.1 `stubby_streams`: seeded Poisson pool positions per laid segment (D1); pools removed with their segment
- [ ] 3.2 Ripple clip on the stream's clock; still when silent (D2)
- [ ] 3.3 Binder and pickup through the existing E path; scanner shows uncollected pools (D3)
- [ ] 3.4 Bounded watchdog review: lay a stream, see pools ripple on the beat, collect an oscilloscope
