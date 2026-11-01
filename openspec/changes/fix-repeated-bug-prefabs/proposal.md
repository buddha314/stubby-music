## Why
Repeated lightning bug placements inherit the same resource ID and clock 1, so Play rejects them as duplicates. Designers must be able to place reusable prefabs without hand-renumbering template state.

## What Changes
Scope inherited bug template state with Jackdaw SceneNodeId and PrefabEntityId. Let each default bug allocate its own clock. Preserve explicit instance/clock overrides and existing duplicate validation. Do not alter the authored scene or add editor functionality.

## Impact
Game adapter binding only; existing prefabs remain loadable.
