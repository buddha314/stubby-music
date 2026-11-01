## 1. Shared browser state and controls
- [ ] 1.1 Add Gallery / List mode to existing Project browser state, defaulting to Gallery and retaining the choice during the session.
- [ ] 1.2 Add themed toolbar controls with clear selected state and existing keyboard/tooltip conventions; retain folder, filters, selection and ordering when switching.

## 2. Filename hover
- [ ] 2.1 Extend the shared filename tooltip to the whole tile/row hover surface, including thumbnail and label, without nested duplicate popovers.
- [ ] 2.2 Keep complete filenames/extensions/Unicode and existing tooltip timing, theme, wrapping, dismissal and window-bound placement.

## 3. Detailed list
- [ ] 3.1 Add a list renderer using existing entries, ThumbnailSlot/cache and file-operation handlers; columns are Thumbnail, Name and Last modified.
- [ ] 3.2 Show full names without ellipsis, wrapping as necessary, and cached filesystem modification date/time with a clear timezone convention and unavailable-value fallback.
- [ ] 3.3 Preserve folder navigation, selection, open, rename, context menus, drag/drop and keyboard behavior across views; avoid per-frame filesystem metadata reads.

## 4. Focused validation and publication
- [ ] 4.1 Run one focused regression for mode/state retention and metadata fallback; briefly review hover coverage, long names and both views with the independent graphical watchdog.
- [ ] 4.2 Commit/push the general enhancement to our shared Jackdaw fork; promote while idle and update the game's editor pin after validation. No upstream code push or PR.
