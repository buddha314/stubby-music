## Why

The reed water purifier needs a **high-pass filter** (`add-reed-water-purifier`), and the
world has no way to give one. AGENTS.md already says where they come from: openings in
rock formations and hollow tree trunks, which pass short waves and stop long ones, like
sound through a narrow duct. This change makes them collectable (user, 2026-11-19).

## What Changes

- **Two resource kinds yield a high-pass filter**, one each, collected once with E and
  never replenished (as the empty snail shell):
  - **a rock opening**: a gap through a rock formation (an arch, a hole, a slot between
    boulders). Each opening gives one.
  - **a hollow trunk**: some scrub oaks have a hollow in their trunk. Each hollow gives one.
    Not every oak is hollow; which ones are is seeded.
- **They are heard before they are seen**: the world's sound passing through an opening or
  hollow is high-passed (thin and bright), so the player can find one by ear, as the bug's
  square is found. Silent world, silent opening: it sounds only once something sounds near it.
- **Art**: rock formations built from the boulder kit, some with an opening; a hollow
  variant of the scrub oak trunk; each opening and hollow carries an `Interact` socket.
- **Rules data**: the `high_pass_filter` item, `rock_opening` and `hollow_trunk` resources,
  and its recipe-book entry.
- **Placement**: rock formations and hollow trunks come from the seeded opening
  (`add-hybrid-world-placement` 2b.4, 2b.5).

## Capabilities

### New Capabilities

- `high-pass-sources`: where high-pass filters come from and how they are collected.

## Impact

- Art (Claude): `tools/blender/create_boulders.py` or a new formation script,
  `assets/props/rocks/`, the scrub oak generator.
- Rules (data): `items.json`, `resources.json`.
- Game (Codex): binders for the formation and the hollow (sockets through
  `model_part_bindings`), the pickup through the existing E path, and the high-passed
  listening (the audio work stream, `add-rack-and-spatial-audio`; it may reuse the VCF
  from `add-reed-water-purifier` 2.3).
