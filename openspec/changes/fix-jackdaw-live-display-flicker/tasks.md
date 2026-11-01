## Investigation
- [x] Add and smoke-test an independent watchdog capped at 20 seconds.
- [x] Run bounded editor-only baseline at fixed 60Hz.
- [x] Run bounded embedded Play at fixed 60Hz.
- [x] Compare adapter or refresh mode where needed; record evidence.
- [x] User reports applying grubby PSR/Replay workaround before reboot.
- [x] After reboot, verify dcdebugmask is 1040 and the intended AMD boot options are loaded.
- [x] Rebuild from the persistent worktree; repeat bounded 165Hz Play with automatic 60Hz recovery.
- [x] Validate the corrective change in a 20-second 165Hz Play test; user confirmed no flicker.

Evidence limit: PSR remains a supported diagnosis, not a proven sole cause. This validation covers the bounded test, not indefinite stability.
