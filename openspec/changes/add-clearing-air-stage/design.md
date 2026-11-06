## Context

`lighting_keys.json` holds two moods at four times of day, and light quality blends them from 0 to 1. **The game does not read this file yet**: only `render_style_sheet.py` does. `TimeOfDay`, the light quality field and the blend are open tasks in `add-desert-opening` (3.4 to 3.7). So this stage can be authored and reviewed in the style sheet now, and appears in the game once those land.

## Goals / Non-Goals

**Goals:**

- Three readable stages: hazy brown, clear and burning, fully colourful.
- Blending stays continuous, with no hard cuts.
- The schema allows the stops to be tuned as data.

**Non-Goals:**

- Building the keys reader or the light quality field, which stay in `add-desert-opening`.
- Grading the world to match the photo exactly, or using anything from it but mood and colour.

## Decisions

**A third mood in the same file, with stops as data.** `light_quality` becomes `bleak` 0, `clearing` 0.5 and `bloomed` 1 as a starting point. The blend takes the two stops either side of the value. Alternatives: a separate haze axis is more expressive but adds a whole dimension to author, and tuning the bleak-to-bloomed curve alone cannot reach orange, since neither end has it.

**What changes at `clearing`, in order of importance.** Fog falls toward bloomed levels, because clearing the air is what this stage is. Skies saturate. Shadows go violet-black. Flora accents stay out or very low: colour from life is what `bloomed` adds. Each time of day needs its own version, and dusk is the one the reference defines. Dawn, day and night need authoring in the same spirit, without a reference yet.

**`bloomed` stays as it is.** Its pinks and violets remain the destination, so the sequence moves through orange into colour. Revisit this once the three rows can be seen together.

**The signal stays amber.** The style guide fixes amber as the module signal in every key. Against an orange sky it may stop reading, and the guide says to fix the key, not the module. This is the main thing to check when authoring.

## Risks / Trade-offs

- **Amber on orange:** see above. Check module legibility in every `clearing` tile.
- **Blends through three stops** may show a change of direction at the middle stop. Check a sweep, not just the stops.
- **Authored blind:** the style sheet is a stand-in. Retune once the game reads the keys (`add-art-direction` 1.4).
- **Four more keys to maintain:** 12 instead of 8. Accepted for the stage.
- **What drives the stage is undecided:** light quality comes mostly from plant density, so the air would clear because of life. The story might call for something more direct, such as the windstorm ending.
