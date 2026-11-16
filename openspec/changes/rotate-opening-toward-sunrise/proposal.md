## Why

The Godot opening rotates the player and scene composition toward east. Bevy still opens facing north, with the sunrise off-screen. The user requests the same composition with the rising sun slightly to the right.

## What Changes

- Start at 06:30 so the rising sun clears the mountain skyline, following the repository guidance to move the opening time later when dawn reads too dark.
- Set the opening bearing to 75 degrees (east is 90), placing astronomical sunrise 15 degrees right of centre (about 20 degrees at 06:30).
- Rotate all authored opening placements and their orientations together about the existing spawn; retain the placed dormant oak, spatial relationships and identities.
- Rotate the decorative mountain rings with the composition, keeping the physical sky and compass directions fixed.
- Move the newly merged snail-shell model/prefab into its asset folder while updating its scene references, retaining the existing asset-layout CI contract.
- Reconcile the in-flight compass requirement's exact-east wording with the requested offset.

## Capabilities

### New Capabilities
- `opening-composition`: sunrise-relative framing of the existing opening.

### Modified Capabilities
None in accepted specs; updates the overlapping in-flight `sync-targeting-and-compass` requirement.

## Impact

Opening scene transforms, shared opening-frame constants, player spawn, mountain-ring transform and focused checks. Models, audio timing, the physical day/night cycle and the separate review scenes remain unchanged. Godot's `scenes/main/main.tscn` and `add-compass-bar/design.md` are references only.
