## 1. Shared browser state and controls
- [x] (claimed: codex) 1.1 Add Gallery / List mode to existing Project browser state, defaulting to Gallery and retaining the choice during the session.
- [x] (claimed: codex) 1.2 Add themed toolbar controls with clear selected state and existing keyboard/tooltip conventions; retain folder, filters, selection and ordering when switching.

## 2. Filename hover
- [x] (claimed: codex) 2.1 Extend the shared filename tooltip to the whole tile/row hover surface, including thumbnail and label, without nested duplicate popovers.
- [x] (claimed: codex) 2.2 Follow the asset-browser filename/path convention (currently filename and extension only; default to no path if none exists), preserve Unicode, and reuse existing tooltip timing, theme, wrapping, dismissal and window-bound placement.

## 3. Detailed list
- [x] (claimed: codex) 3.1 Add a list renderer using existing entries, ThumbnailSlot/cache and file-operation handlers; columns are Thumbnail, Name and Last modified.
- [x] (claimed: codex) 3.2 Show full names without ellipsis, wrapping as necessary, and cached filesystem modification date/time with a clear timezone convention and unavailable-value fallback.
- [x] (claimed: codex) 3.3 Preserve folder navigation, selection, open, rename, context menus, drag/drop and keyboard behavior across views; avoid per-frame filesystem metadata reads.

## 4. Focused validation and publication
- [x] (claimed: codex) 4.1 Run one focused regression for mode/state retention and metadata fallback; briefly review hover coverage, long names and both views with the independent graphical watchdog.
- [ ] (claimed: codex) 4.2 Commit/push the general enhancement to our shared Jackdaw fork; promote while idle and update the game's editor pin after validation. No upstream code push or PR.

## Workstream checkpoint

Implementation: https://github.com/buddha314/jackdaw/pull/2 (draft), branch
feat/asset-browser-views, commit 031027b2. One focused regression, editor build
and bounded 20-second UI reviews passed for thumbnail hover, both views, UTC
timestamps, long-name wrapping and selection retention. Full implementation
evidence is in the Jackdaw PR and docs/research/asset-browser-views.md there.
Shared installation/pin promotion (4.2) remains pending; the live editor was not
replaced. Existing unrelated long-name inspector/status-label limitations are
recorded in the implementation notes.
