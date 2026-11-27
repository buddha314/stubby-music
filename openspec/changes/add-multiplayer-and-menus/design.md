## Context

The us-outpost-31 template (VerdantAI) has, on our engine pin (Bevy main `526f9e40`,
avian forks, Jackdaw `39af09e3`), working and tested:

- `crates/game/src/net.rs`: lightyear 0.31 host-server and clients over UDP/netcode,
  prediction of the own player, interpolation of others, a `Replication` bridge from
  Jackdaw's `SceneNodeId`, rollback-safe effects, disconnect reporting. 12 tests over
  in-process links, including a lossy one.
- `crates/game/src/steam.rs` + `crates/steam`: Steam lobbies, invites, rich presence,
  build/protocol gating, P2P transport.
- `crates/game/src/menu.rs`, `menu/options.rs`, `settings.rs`: main menu, connecting,
  loading, lobby, session-ended, Settings and Controls screens; `bevy_settings`.
- Lessons paid for: a lightyear rollback bug with `sync_to_transform`
  (`transform_follows_position`), menus drawing on a deactivated camera
  (`IsDefaultUiCamera`), mouse-look jumping when a menu releases the cursor.

stubby is single-player: its own first-person controller (`stubby_player::controls`,
Satisfactory-style, per-frame), a seeded deterministic world (`stubby_world`), systems
in layers (`split-game-into-systems`), a realtime audio engine outside Bevy whose clock
is the musical source of truth, and screens on `stubby_core::ScreenMode`.

## Decisions

**D1 Port, don't reinvent.** Networking, Steam, menus and settings follow the
template's code and tests, renamed into stubby's layers. The lightyear forks are already
in the shared `bevy_main_forks.json`; adding them here is `adding-a-dependency` (re-pin
Bevy, undo lock drift, licences in `deny.toml`).

**D2 Layers.** `stubby_net` is a service (lightyear session, protocol registration,
the replication bridge, session state); systems register what they replicate through it
and never import lightyear directly. `stubby_steam` is Bevy-free (as `outpost_steam`).
The menus live in `stubby_game` (assembly and glue), drawn with `stubby_ui`.

**D3 Same world from one seed.** The world is generated from `WorldConfig` (seed and
options) on every peer; the host sends its config when a client connects, and the client
generates before it is shown the world (the loading screen covers it). Authored and
seeded entities pair across peers by a stable key (Jackdaw's `SceneNodeId`; for seeded
placements, the placement index), as the template's `NetKey` does. A determinism test
generates the world twice in one process and compares the placements.

**D4 The host decides.** Collecting, building, crafting, machine configuration and
inventory changes are requests from clients (lightyear messages) that the host checks
against the same rules (`stubby_recipes`, `buildables`) and applies; results replicate.
Clients show their own request at once only where a wrong guess is harmless (the hand
animation, a build hologram), never for inventory.

**D5 Players.** Movement inputs become BEI actions read in the fixed step, so lightyear
can send them and replay them on rollback; the controller's feel (speeds, slide, step)
is kept, moved from per-frame to the fixed step. Whether to keep stubby's controller in
the fixed step or move to `bevy_ahoy` (as the template did) is decided by a spike
(task 2.3): keep the feel, prefer the existing crate if it can match it.

**D6 Time.** Simulation time and the calendar are the host's and replicate; clients do
not advance them. The simulation-speed keys work only on the host.

**D7 Music together.** Each peer runs its own `stubby_audio` engine (audio never
crosses the network). Rack configurations are data and replicate. Clocks follow the
host: the host sends its clock's beat phase against lightyear's synchronized tick, and
clients steer their engine's tempo to it, so everyone hears the same bar. How tightly
is an open question (below).

**D8 Screens in stubby's style.** The template's screen logic (states, loading, settings
persistence, lobby) on `ScreenMode`, drawn like the existing pause menu (amber on dark,
left column). The pause menu becomes the lobby in a session (players, Invite friends on
Steam, Settings, Leave). Leaving restarts into the main menu (a fresh process drops
every replicated entity, as in the template).

**D9 Testing.** The template's `Session` harness (host and clients in one process over
crossbeam, optionally lossy) for every replicated feature; two processes on one machine
for the menus; the two-machine Steam test by hand.

## Open questions (for the user)

1. **Inventory:** per player (Satisfactory) or one shared settlement store? Default:
   per player, with the settlement's stock shared.
2. **Players per session:** template default is 8.
3. **Who may configure a machine's rack** (anyone, or its builder)? Default: anyone.
4. **Music sync tolerance:** is "same bar, tens of milliseconds apart" enough, or must
   peers be sample-aligned (much harder over the internet)?
5. **Joining mid-session:** the world state since the start must reach a late joiner;
   is a short wait on the loading screen acceptable? Default: yes.
6. **Saves:** a hosted world is saved by the host only (with `add-saved-settings-and-saves`
   in the template, not yet built).

## Risks

- The per-frame controller moving to the fixed step may change its feel; the spike
  (2.3) checks it with the user before the rest builds on it.
- Every system that changes the world must route through the host (D4); a missed one
  desyncs silently. Each replicated feature gets a two-peer test.
- Audio clock following is new ground; it is last, after the shared world works.
