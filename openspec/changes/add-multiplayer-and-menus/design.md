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
placements, FNV-1a of the serialized `PlacementKey`, not its index, which shifts when a
rule changes), as the template's `NetKey` does. A determinism test generates the world
twice in one process and compares the placements and keys. **Built (2.5):** the host's
config replicates as JSON (`HostWorld`); a client whose world differs restarts with the
host's (`--world-config`) and rejoins, since a fresh process is the one sure way to
generate another world. Entities to share are marked `stubby_net::Shared`.

**D4 The host decides.** Collecting, building, crafting, machine configuration and
inventory changes are requests from clients (lightyear messages) that the host checks
against the same rules (`stubby_recipes`, `buildables`) and applies; results replicate.
Clients show their own request at once only where a wrong guess is harmless (the hand
animation, a build hologram), never for inventory. **Built (3.1–3.3):** requests travel on `stubby_net::HostChannel`;
systems that would apply one locally stand aside under `stubby_core::DecidedElsewhere`.
What is built or laid carries a small serializable record (`Built`, `StreamRecord`)
that replicates, and every peer shows it from the record and simulates it locally, as
every peer generates the world (D3). Each player's inventory is a `Pack` on the host,
replicated to its owner alone; storage is built barrels (`Store`), which anyone may use.

**D5 Players.** Movement inputs become BEI actions read in the fixed step, so lightyear
can send them and replay them on rollback. **Decided (user, 2026-11-27, after the spike
in 2.3): `bevy_ahoy`**, as the template uses it, tuned to stubby's speeds and jump
(`stubby_player::movement`). The ground is a heightfield sampled from `GroundHeight` over
the whole world; the simulation-speed keys scale ahoy's tuning so the player moves in real
time. The per-frame controller is gone.

**D6 Time.** The calendar is the host's and replicates; clients keep theirs close to
it. **Revised 2026-11-28:** in company time runs at 1x and the speed keys are off; they
work in a world of one. lightyear sets Bevy's virtual-time speed on every networked peer
for its timeline sync, so a sped-up host would drag its clients' timelines along; the
world's speed is the game's own `SimSpeed`, which the calendar reads. Speeding a shared
world's calendar alone (physics at 1x) is possible later if wanted.

**D7 Music together.** Each peer runs its own `stubby_audio` engine (audio never
crosses the network). Rack configurations are data and replicate. Clocks follow the
host: the host sends its clock's beat phase against lightyear's synchronized tick, and
clients shift their engine's master clock to it, so everyone hears the same bar.
**Built (4.2):** a master offset added to every master-locked clock's sample counter
(not a tempo change: clocks derive their beat from the counter), corrected when the
smoothed error passes 15 ms; about 5 ms apart in steady play.

**D8 Screens in stubby's style.** The template's screen logic (states, loading, settings
persistence, lobby) on `ScreenMode`, drawn like the existing pause menu (amber on dark,
left column). The pause menu becomes the lobby in a session (players, Invite friends on
Steam, Settings, Leave). Leaving restarts into the main menu (a fresh process drops
every replicated entity, as in the template).

**D9 Testing.** The template's `Session` harness (host and clients in one process over
crossbeam, optionally lossy) for every replicated feature; two processes on one machine
for the menus; the two-machine Steam test by hand.

## Open questions (for the user)

All but saves are settled (2026-11-27).

1. ~~Inventory~~ **Settled (user, 2026-11-27): per player, with shared storage.** Each
   player carries their own inventory; storage in the world is shared by everyone. Both
   are host state (D4): a client requests a move between them and the host applies it.
2. ~~Players per session~~ **Settled (user, 2026-11-27): the template's multiplayer
   settings**, in `stubby_core::multiplayer`: 8 players, friends-only Steam lobbies,
   Steam app 480 until stubby has its own id, UDP port 5888, a 64 Hz tick, protocol id
   `stubby01`.
3. ~~Rack permissions~~ **Settled (user, 2026-11-27): anyone** may configure any
   machine's rack.
4. ~~Music sync tolerance~~ **Settled (user, 2026-11-27): music is tied to locations,
   so it is universally synced.** What plays where is world state (the machines and
   their racks), the same for everyone; every peer follows the host's musical clock
   (D7), so two players at one place hear the same music in time. Phase 4 measures how
   close the clocks get and reports it.
5. ~~Joining mid-session~~ **Settled (user, 2026-11-27): yes**, a late joiner waits
   briefly on the loading screen while the world state since the start arrives.
6. **Saves:** a hosted world is saved by the host only (with `add-saved-settings-and-saves`
   in the template, not yet built).

## Risks

- The per-frame controller moving to the fixed step may change its feel; the spike
  (2.3) checks it with the user before the rest builds on it.
- Every system that changes the world must route through the host (D4); a missed one
  desyncs silently. Each replicated feature gets a two-peer test.
- Audio clock following is new ground; it is last, after the shared world works.
