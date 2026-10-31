# Satisfactory control conventions

Researched 2026-10-31 from the [official Controls reference](https://satisfactory.wiki.gg/wiki/Controls),
[Settings reference](https://satisfactory.wiki.gg/wiki/Settings), and
[HUD reference](https://satisfactory.wiki.gg/wiki/HUD). The wiki's indexed control
and settings tables were available; direct page fetches returned HTTP 403.

Use familiar builder interactions when their corresponding game systems exist.
The table records keyboard/mouse defaults, not controller bindings.

| Action | Satisfactory default | stubby-music decision |
| --- | --- | --- |
| Move | WASD | Grounded movement relative to heading |
| Look | Mouse X/Y | Free look, no mouse button required |
| Sprint | Left Shift | Hold to sprint |
| Jump / crouch | Space / C | Grounded jump / hold to crouch |
| Use / collect | E | Implemented; hold to collect successive nearby bugs |
| Build menu | Q | Reserved; build system not implemented |
| Dismantle | F | Reserved; dismantling not implemented |
| Inventory | Tab | Reserved; inventory count is currently on HUD |
| Codex | O | Recipe book implemented on O |
| Flashlight | B | Reserved; no flashlight yet |
| Resource scanner | V | Reserved for discovery/scanning |
| Map / quick search | M / N | Reserved for corresponding future systems |
| Place / cancel building | Left / right mouse | Future build context |
| Rotate placement / snap | Mouse wheel / Ctrl | Future build context |
| Sample existing building | Middle mouse | Future build context |
| Bench crafting | Space | Future crafting-bench context |

Interaction distinctions matter: Satisfactory supports holding E for collecting
flora, and tapping or holding it to mine resource nodes. Our current bugs yield
one waveform pattern per instance, so E collects once and holding it can sweep
successive nearby bugs. Continuous mining of a multi-unit deposit is a separate
future mechanic; do not accidentally make one bug yield every frame.

Recipe-book knowledge survives consuming material. O is the codex-like entry
point; it should not compete with B's flashlight role. E must never double as
camera ascent. The player starts standing on the mudflat. Mouse look is captured during play;
Escape or focus loss releases the cursor, and left-click recaptures it. Looking
up/down never changes walking height. Terrain support bridges small cracks;
machine collision and sprint-slide momentum are future work.
