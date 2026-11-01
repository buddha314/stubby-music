## Method
Use GNU timeout with SIGKILL as an independent 20-second ceiling, outside the
renderer and Python control loop. Each test launches only its own editor process
group on a dedicated control port. Jackdaw also sets PDEATHSIG on its game child.
Afterward restore fixed native 60Hz. Higher-refresh tests also briefly power-cycle the display output,
leaving the desktop session intact. Save original run config contents and restore
them after each Play test. Never target an existing editor. No test can request
more than 20 seconds; shorter watchdog smoke tests need no display changes.

## Hypotheses
Refresh/scanout, compositor interaction with concurrent Vulkan contexts, or a
rendering problem remain possibilities. No GPU hang/reset is proven by existing
kernel logs. Run at most one graphical test at a time. Start at fixed 60Hz before
considering the original 165Hz mode. Do not interpret absent user feedback as
proof that physical display flicker did not occur. Screenshot samples and driver
logs can identify software faults but cannot measure physical panel flicker.

## Read-only driver investigation

Framework identifies PSR/Panel Replay as a known internal-panel flicker source
on Ryzen 7040 laptops. The current boot explicitly sets
`amdgpu.dcfeaturemask=0x8`; the kernel enum identifies 0x8 as DC_PSR_MASK.
The current dcdebugmask is 0. Both parameters are read-only (0444), so the
vendor's suggested disable-PSR/Replay workaround is a boot-time change requiring
a reboot. No boot configuration was modified during this investigation.

Primary sources:
- https://knowledgebase.frame.work/psr-display-flicker-workaround-for-linux-SJTKb7VMl
- https://dri.freedesktop.org/docs/drm/gpu/amdgpu/driver-core.html

This narrows the driver hypothesis but does not establish causality. The user
confirmed fixed-60Hz Play was flicker-free. Physical results of 165Hz comparisons
are not confirmed. Keep task 5.5 open for stable interactive validation.
