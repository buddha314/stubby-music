## Why

The *Clean the water* goal (2026-11-20) waits on `item_held` for "find a high-pass filter",
"find an oscilloscope" and "craft a purifier kit". The user found the filter step never
completed: the filter had already gone into a kit, and the kit into a dam, so the player no
longer *held* them. A step that asks the player to find or make something should be met by
having found or made it, which the recipe book already records and never forgets.

## What Changes

- A goal condition kind **`item_found { item }`**: met once the player's recipe book knows the
  item (collected or crafted); it latches, so using the item up later does not undo it.
- Crafting now teaches its outputs to the recipe book, as collecting already does.
- *Clean the water*'s find and craft steps use `item_found`.
- The craft notice names the recipe ("Crafted Reed water purifier."), not its ID.

## Impact

- `stubby_recipes` (the condition, schema, validation), `bevy_stubby_recipes` (crafting
  discovers its outputs), `assets/recipes/goals.json` and the formatter's schema.
