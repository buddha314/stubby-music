## 0. Decide (user)
- [x] 0.1 The oscilloscope comes from ripple pools in streams, Poisson along the stream, rippling to the beat, refreshing semi-daily (user, 2026-11-19)
- [x] 0.2 Mean spacing 12 m (user, 2026-11-19)
- [ ] 0.3 Open Question 2 (pools on wild water)

## 1. Art (Claude)
- [x] 1.1 (claimed: claude; done 2026-11-19) Ripple pool model, `tools/blender/create_stream_features.py`, `assets/streams/ripple_pool/`: still clear water just beyond the left bank (D2), in the stream segment's frame (flow +X, origin at the channel centre); `Interact` (resource `ripple_pool`); prefab, previews. No LODs: small and animated
- [x] 1.2 (claimed: claude; done 2026-11-19) `Ripple_Beat`, one beat at 24 fps: a ring leaves the centre on the beat while last beat's ring reaches the rim, with a drop at the centre; `Ripples` ships at scale 0, shown while the stream sounds, clip looped at BPM / 60
- [x] 1.3 (claimed: claude; done 2026-11-19) Item icon `assets/ui/icons/oscilloscope.png`

## 2. Rules (data)
- [ ] 2.1 `oscilloscope` item; `ripple_pool` resource yielding one, `replenishment: after 1800`; `rules-fmt`

## 3. Game (Codex)
- [ ] 3.1 `stubby_streams`: seeded Poisson pool positions per laid segment (D1); pools removed with their segment
- [ ] 3.2 Ripple clip on the stream's clock; still when silent (D2)
- [ ] 3.3 Binder and pickup through the existing E path; scanner shows uncollected pools (D3)
- [ ] 3.4 Bounded watchdog review: lay a stream, see pools ripple on the beat, collect an oscilloscope
