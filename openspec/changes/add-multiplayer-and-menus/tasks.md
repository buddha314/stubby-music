Requested by the user on 2026-11-27: stubby goes into a menu and lobby, and plays together,
as the us-outpost-31 template does. Owner: Claude, by the user's direct request.
Port from the template (VerdantAI/us-outpost-31) rather than writing anew (D1).

## 1. Screens (single-player first)
- [x] 1.1 Main menu at launch in stubby's style (Play, Settings, Quit now; Host, Join, Host on Steam once 2 and 5 exist), on `ScreenMode` (claimed: Claude). Host, Join and Host on Steam shown as coming; they light up with 2 and 5.
- [x] 1.2 Loading screen over world generation and scene spawn, with a timeout (claimed: Claude). (`menus::AwaitScene`/`WorldReady` from stubby_jackdaw's scene events; 20 s timeout)
- [x] 1.3 Settings screen (window mode and size, anti-aliasing, shadows, UI scale, vsync; master, effects, music volume) saved with `bevy_settings`; reachable from the main menu and the pause menu (claimed: Claude). Volumes need the engine's new bus stages (`Command::SetBusVolume`: master, world after the world meter, interface). Window size and anti-aliasing left out (stubby sets its own window).
- [x] 1.4 Pause menu: Settings and Exit to main menu (restart into the menu); no camera jump when a menu frees the mouse (claimed: Claude). (restarts with the same arguments; stubby's look already turns only while captured, so no jump)
- [x] 1.5 Tests: the menu flow headless; screenshots of each screen (claimed: Claude). (`menus` tests, `controls::escape_and_tab_leave_the_menus_to_themselves`, `stubby_audio/tests/volume.rs`; screenshots reviewed)

## 2. Networking core
- [x] 2.0 The template's multiplayer settings as data (`stubby_core::multiplayer`: Steam lobby, port, tick, protocol id), by the user's request (claimed: Claude). The editor's Host/Join/Steam host runs and Jackdaw's multiplayer module come with 2.2, when they do something
- [ ] 2.1 `stubby_net` service crate; lightyear and BEI forks added (`adding-a-dependency`: re-pin Bevy, lock drift, `deny.toml`); layering bans updated
- [ ] 2.2 Host and join over UDP (`--host`, `--join`, the menu's Host and Join); the template's `Session` test harness
- [x] 2.3 Spike: player movement in the fixed step from BEI actions, stubby's controller vs `bevy_ahoy`; the user judges the feel (D5) (claimed: Claude). **The user chose `bevy_ahoy`** (2026-11-27): `stubby_player::movement` replaces the per-frame controller, with tests for speeds (also at 4x simulation speed), one jump per press, crouch, solids vs markers, screens, and the ground collider
- [ ] 2.4 Players replicated: own predicted, others interpolated; Omar's shadow body for others
- [ ] 2.5 World config from the host; determinism test (two generations compare equal); seeded and authored entities paired by stable keys (D3)

## 3. Shared world state (host decides, D4)
- [ ] 3.1 Collecting and sources (lightning bugs, plants, mines)
- [ ] 3.2 Inventory (per D4 and open question 1)
- [ ] 3.3 Building: streams and placed machines
- [ ] 3.4 Crafting, benches and durations
- [ ] 3.5 Rack configurations (modules, cables, knobs) as replicated data
- [ ] 3.6 Simulation time, calendar and sky from the host (D6)
- [ ] 3.7 A two-peer test per feature, including a lossy link

## 4. Music together (D7)
- [ ] 4.1 Each peer's engine plays the replicated machines
- [ ] 4.2 Clock following against lightyear's synchronized tick; measure the drift (open question 4)

## 5. Steam
- [ ] 5.1 `stubby_steam` from the template's `outpost_steam`: lobby, invites, rich presence, build and protocol gating
- [ ] 5.2 Steam P2P transport; Host on Steam and invite joins from the menu
- [ ] 5.3 Two machines (breq and ctenophora, two Steam accounts) by hand

## 6. Lobby and hardening
- [ ] 6.1 Pause menu as lobby: players, Invite friends, Leave (D8)
- [ ] 6.2 Session-ended screen; disconnect and refusal messages
- [ ] 6.3 Late joiners receive the world state (open question 5)
- [ ] 6.4 AGENTS.md: the multiplayer rules every new system must follow (route changes through the host; a two-peer test)
