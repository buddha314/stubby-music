## Implementation

- [x] 1.1 Reconcile compass framing and define the shared opening heading/spawn (claimed: Codex).
- [x] 1.2 Rotate the production placements and ridge rings as one composition; preserve oak and scene identities (claimed: Codex).
- [x] 1.3 Update focused grounding/look and opening-frame checks (claimed: Codex).
- [x] 1.4 Validate OpenSpec, build and capture a bounded standalone opening review (claimed: Codex).

## Evidence

- Player controls: 5 focused tests passed, including 75-degree spawn and the 15-degree astronomical-sunrise offset.
- Production scene: Jackdaw parser/prefab resolution passed; oak and burbler retain their original camera-relative XZ coordinates.
- Authored opening time: the solar-direction check confirms the sun is 10–25 degrees right and above the maximum ridge elevation.
- Rigid rotation audit: 20 root placements rotated around (0, 0, 6), preserving all pairwise distances, SceneNodeIds and nested local transforms.
- Asset layout/version checks, changed Rust formatting and both affected OpenSpec validations passed.
- `cargo build -p stubby_jackdaw --locked` passed.
- `python3 tools/jackdaw_timed_test.py --standalone --name=east-opening`: final 20.030-second run, expected watchdog exit -9, display restored, no asset errors or panics. Screenshot `/tmp/stubby-opening-east-opening.png` shows the low sun at right and the dormant oak ahead-left.
- The first exact-06:00 image showed the sun hidden by the ridge; the authored 06:30 start makes it visible without changing night lighting. The missing generated floor preview was generated before the final run. The asset checker now consistently exempts this generated floor both when absent and present.
