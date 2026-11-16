## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 Amend existing nearest-target requirement explicitly; retain avian3d query and hold-E behavior, never substitute handwritten reach checks. (`stubby_player::interaction::pick`: most central within 20°, else nearest; same avian query.)
- [x] 2.1 Spike a Bevy-compatible maintained outline solution on thin reeds, shells and scale-animated bugs; record compatibility and measured cost before adopting it. (design.md, "Outline spike")
- [x] 3.1 Add single-target outline presentation and compass with data-driven style and screen suppression. (`stubby_player::outline`, `assets/ui/collectible_outline.json`; `stubby_player::compass`; build mode via `stubby_core::BuildMode`.)
- [ ] 4.1 Test targeting tie/fallback cases, hidden/collected targets, LOD changes and compass bearings; bounded graphical review only. (Unit tests for ranking, outline suppression and bearings; prefab review captures the outlined target. Open: a bug's dark silhouette between flashes; the opening facing east.)
