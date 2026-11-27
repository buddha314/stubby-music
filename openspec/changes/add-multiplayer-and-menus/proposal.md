## Why

The user (2026-11-27): stubby should go into a menu and lobby at launch, and play
together, as games made from the us-outpost-31 template do. Today it starts straight in
the desert with no main menu, no loading screen and no settings, and is single-player.
The template has built, tested versions of all of it on the same engine pin: a main
menu, loading, settings saved per user, an in-session lobby, and host/join multiplayer
with prediction over UDP and Steam (lightyear, whose Bevy-main forks are already in our
shared `bevy_main_forks.json`). This change brings them to stubby, in stubby's own look.

## What Changes

- **Screens, single-player first:** a main menu at launch (Play, Host, Join, Host on
  Steam, Settings, Quit), a loading screen while the world generates and the scene
  spawns, a Settings screen (window, graphics, volumes; saved per user with
  `bevy_settings`), and the pause menu gaining Settings and Exit to main menu. All in
  stubby's style (`stubby_ui::style`), on the existing `ScreenMode`.
- **Networking core:** a host plays and others join (UDP for development and LAN).
  The host is authoritative; every peer generates the same world from the host's seed,
  so only what changes is sent. Players move with prediction.
- **Shared world state:** collecting, building (streams, machines), crafting, rack
  configurations, time of day and the calendar are the host's, replicated to everyone.
- **Music together:** every peer runs its own audio engine; machine racks (data) are
  replicated and the clocks follow the host's.
- **Steam:** lobbies and friend invites through Valve's relays, refusing other builds.
- **The lobby:** the pause menu during a session lists who is there, invites friends,
  and leaves; a session-ended screen says why a session stopped.

## Capabilities

### New Capabilities
- `game-menus`: the main menu, loading, settings and the in-session lobby.
- `multiplayer`: hosting, joining, what is shared, and how it stays consistent.

### Modified Capabilities
None (the pause menu's requirements move into `game-menus`).

## Impact

- New crates: `stubby_net` (service: lightyear session, protocol, replication glue) and
  `stubby_steam` (Bevy-free Steam lobby/invite layer, as the template's `outpost_steam`);
  `deny.toml` bans gain them.
- `stubby_player` (inputs as BEI actions in the fixed step), `stubby_game` (menus,
  glue), `stubby_world` (seeded generation shared by peers), `stubby_streams`,
  `bevy_stubby_rack`, `bevy_stubby_recipes` (host-validated actions), `stubby_audio` /
  `bevy_stubby_audio` (clock following).
- Dependencies: lightyear (fork, already pinned for the template), bevy_enhanced_input
  (fork), steamworks; Bevy features `bevy_settings`, `bevy_ui_widgets`.
- Owner: Claude, by the user's direct request (crates are otherwise Codex's).
