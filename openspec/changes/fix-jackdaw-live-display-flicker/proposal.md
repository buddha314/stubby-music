## Why
Embedded Jackdaw Play was followed by severe whole-display flicker that recurred after Stop. The display stabilized after closing render apps, switching from fixed 165Hz to fixed 60Hz, and resetting the output. The cause is unconfirmed.

## What Changes
- Add an independent maximum-20-second graphical test watchdog.
- Recover fixed 60Hz and reset the output after each test automatically.
- Compare editor-only and Play, refresh rate and GPU adapter in bounded tests.

## Capabilities
### New Capabilities
- `timed-render-tests`: bounded isolated graphical diagnostics.

## Impact
Diagnostic tooling and investigation notes. No permanent driver or compositor changes.
