## 1. Shared browser state and controls
- [ ] (claimed: codex) 1.1 Add Gallery / List mode to existing Project browser state, defaulting to Gallery and retaining the choice during the session.
- [ ] (claimed: codex) 1.2 Add themed toolbar controls with clear selected state and existing keyboard/tooltip conventions; retain folder, filters, selection and ordering when switching.

## 2. Filename hover
- [ ] (claimed: codex) 2.1 Extend the shared filename tooltip to the whole tile/row hover surface, including thumbnail and label, without nested duplicate popovers.
- [ ] (claimed: codex) 2.2 Follow the asset-browser filename/path convention (currently filename and extension only; default to no path if none exists), preserve Unicode, and reuse existing tooltip timing, theme, wrapping, dismissal and window-bound placement.

## 3. Detailed list
- [ ] (claimed: codex) 3.1 Add a list renderer using existing entries, ThumbnailSlot/cache and file-operation handlers; columns are Thumbnail, Name and Last modified.
- [ ] (claimed: codex) 3.2 Show full names without ellipsis, wrapping as necessary, and cached filesystem modification date/time with a clear timezone convention and unavailable-value fallback.
- [ ] (claimed: codex) 3.3 Preserve folder navigation, selection, open, rename, context menus, drag/drop and keyboard behavior across views; avoid per-frame filesystem metadata reads.

## 4. Focused validation and publication
- [ ] (claimed: codex) 4.1 Run one focused regression for mode/state retention and metadata fallback; briefly review hover coverage, long names and both views with the independent graphical watchdog.
- [ ] (claimed: codex) 4.2 Commit/push the general enhancement to our shared Jackdaw fork; promote while idle and update the game's editor pin after validation. No upstream code push or PR.
