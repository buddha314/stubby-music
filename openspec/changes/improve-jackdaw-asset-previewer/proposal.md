## Why

The Project asset browser truncates gallery filenames, making similarly named
prefabs difficult to identify. A label-only tooltip already exists, but hovering
the thumbnail or the rest of the tile does not consistently reveal the filename.
The gallery also provides no detailed listing for comparing file names and dates.

## What Changes

- Reveal the complete file name, including its extension, on hover using Jackdaw's
  existing tooltip component, timing, styling and placement conventions.
- Add an explicit Gallery / List choice to the existing Project browser toolbar.
- Keep the current gallery and add list rows with a thumbnail, the full filename,
  and the file's last-modified date/time.
- Share browser entries, navigation, filters, selection, operations and cached
  thumbnails across both views. Retain the chosen mode during the editor session.

## Capabilities

### New Capabilities
- `jackdaw-asset-browser`: consistent filename discovery and gallery/list browsing.

## Impact

This proposal is tracked in the game's existing OpenSpec repository, alongside
other Jackdaw work. Implementation belongs in the shared buddha314 Jackdaw fork,
principally src/project_window.rs and the existing thumbnail/settings/UI helpers.
It is a general editor enhancement: no game-specific editor behavior, alternate
asset loader, replacement thumbnail renderer or LOD implementation.

Specification only in this change request; implementation tasks remain unclaimed.
Upstream code pushes and pull requests remain forbidden.
