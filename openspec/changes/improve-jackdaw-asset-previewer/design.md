## Context and existing conventions

Reviewed the shared Jackdaw dev/0.20 source:

- src/project_window.rs already uses Tooltip::title plus Hovered on truncated
  filename labels. Labels currently shorten names longer than ten characters.
  The asset tooltip uses entry.file_name, without directory components. Search
  results expose their folder separately via attach_folder_caption; scene tabs
  have a separate path-description convention, which does not apply to asset tiles.
- crates/jackdaw_feathers/src/tooltip.rs owns the shared hover behavior: 300 ms
  title delay, 1200 ms expanded stage, theme tokens and constrained placement.
  Reuse it; do not create another tooltip timer, popup or placement algorithm.
- ProjectWindowState owns the current folder, entries, search, kind filter and
  selection. Both presentations must use that one listing and its existing
  file-operation and drag/drop handlers.
- src/thumbnail.rs owns ThumbnailSlot, visibility-aware requests and cached
  images. Both presentations use that service and its existing fallback icons.
  The native-LOD thumbnail-layer fix is already on our fork's dev/0.20; retain it.

## Decisions

### Filename discovery

Attach a single shared filename tooltip to the interactive tile/row hover surface,
covering the thumbnail and label. Its title is the exact filesystem filename,
including extension and Unicode characters, without directory components, following
the existing asset-browser convention. The list Name column follows the same rule.
If a future target lacks a filename/path convention, default to filename only for
readability. Keep existing separate breadcrumbs/folder captions. Remove duplicate nested filename
tooltips where necessary. Match current tooltip timing, typography, dismissal and
window-bound placement; wrap long tooltip text rather than truncating its content.
Do not change operator tooltips elsewhere or introduce asset-specific hover UI.

### Presentation choice

Use existing themed toolbar toggle/segmented controls with Gallery and List labels,
selected-state styling, keyboard focus and standard tooltips. Gallery remains the
initial default. Store presentation mode in browser state, independent of entries.
Keep it while navigating folders and reopening the panel in the same session.
Persistence across editor restarts is deferred; do not introduce a preference store.

Switching presentation must retain the folder, search, kind filter, selected path
and rename/inspector subject. Keep the selected item visible where possible.
Do not re-import files or invalidate thumbnail images because the view changes.

### List layout and metadata

Use column headers Thumbnail, Name and Last modified. Render one entry per row:
a compact cached thumbnail (or the gallery's fallback icon), the complete filename
including extension, and an absolute date/time from filesystem modification time.
Let long names wrap and row height grow so the name is never replaced by ellipsis;
keep the date column readable. Reuse theme spacing and selection/hover treatments.
Directories retain their existing folder icons and navigation behavior.

Use an existing Jackdaw date/time formatter where one exists; otherwise use a
consistent local-time display such as YYYY-MM-DD HH:mm and make its timezone
clear through the column's standard tooltip. Do not add relative-age timers.
Read modified time with the existing directory scan/refresh path and cache it in
entry metadata. Do not stat every row every frame. If metadata is unavailable or
a file disappears during refresh, show an em dash and retain existing fail-soft
file handling; do not substitute creation/access time or the current time.

Selection, open, rename, context menu, drag/drop and keyboard navigation keep the
existing Project browser meanings. Adding sorting controls, extra metadata columns,
thumbnail-size controls, a new inspector, or a separate preview viewport is out of
scope. Preserve current ordering in both presentations.

## Validation and delivery

Keep checks minimal: one focused regression covering mode changes/state retention
and metadata fallback, plus a brief manual review of hover coverage, long Unicode
filenames and both views. Any graphical diagnostic uses the independent 20-second
watchdog on an unused port and verifies project/executable identity before commands.
No blanket workspace tests or new test infrastructure. Implement in a Jackdaw
worktree and publish only to buddha314. Promote shared binaries and update the
game's compatibility pin only after the editor is idle and validation passes.
