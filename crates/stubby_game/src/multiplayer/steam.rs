//! Multiplayer over Steam (add-multiplayer-and-menus 5.1-5.2), from the template
//! us-outpost-31's `steam.rs`: the host creates a Steam lobby and plays on a lightyear
//! server listening on Steam P2P; friends join from an invite (or the overlay's "Join
//! game") and connect to the lobby's owner through Valve's relays. No ports to open, no
//! addresses exposed. `stubby_steam` is the Bevy-free Steam side; this module starts
//! lightyear on what it reports.
//!
//! ```text
//! stubby_jackdaw --steam-host          # create a lobby; the invite dialog opens
//! stubby_jackdaw --steam               # play; accept an invite to join
//! stubby_jackdaw +connect_lobby <id>   # Steam launches this from an invite
//! ```
//!
//! A plain launch (the main menu) starts Steam too, so its "Host on Steam" button works
//! and an accepted invite joins from the menu. A joining player checks the lobby's build
//! and protocol first and refuses a different build, saying so. Without a running Steam
//! the game says why: `--steam-host` then hosts over UDP instead, and the menu shows the
//! reason in place of its Steam button. `--no-steam` does the same on purpose, for checking
//! the menus on a machine whose Steam is logged in.
use bevy::prelude::*;
use stubby_core::multiplayer::{DEFAULT_PORT, PROTOCOL_ID, SteamLobbySettings};
use stubby_net::{
    NetMode, SessionEnded,
    lightyear::prelude::{
        client::{ConnectTarget, SteamClientIo},
        server::{ListenTarget, SteamEndpoint},
        *,
    },
};
use stubby_steam::{Build, Event, Steam, steamworks::LobbyId};

/// Leave Steam alone this launch (the menu says why its Steam button does nothing).
pub const NO_STEAM: &str = "--no-steam";

/// The Steam P2P "port" the game's server listens on.
const VIRTUAL_PORT: i32 = 0;

/// What this build must match to play with a host.
pub fn this_build() -> Build {
    Build {
        id: env!("STUBBY_BUILD").to_string(),
        protocol: PROTOCOL_ID,
    }
}

/// Steam start-up, lobbies and invites, driving lightyear's Steam transport. In
/// [`NetMode::Steam`] and [`NetMode::Menu`].
pub struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        let mode = app
            .world()
            .get_resource::<NetMode>()
            .copied()
            .unwrap_or_default();
        let (host, lobby) = match mode {
            NetMode::Steam { host, lobby } => (host, lobby),
            NetMode::Menu => (false, None),
            _ => return,
        };
        let lobby_settings = SteamLobbySettings::default();
        // `--no-steam` leaves Steam alone: a check of the menus on a machine whose Steam is
        // logged in must not show its player in a game (app 480, Spacewar).
        let started = if std::env::args().any(|arg| arg == NO_STEAM) {
            Err("started with --no-steam".to_string())
        } else {
            Steam::start(lobby_settings.app_id).map_err(|error| error.to_string())
        };
        match started {
            Ok(steam) => {
                info!(
                    "Steam ready as {:?} (app {})",
                    steam.me(),
                    lobby_settings.app_id
                );
                app.insert_resource(SteamworksSockets::Client(steam.client().clone()))
                    .insert_resource(SteamSession {
                        steam,
                        lobby: None,
                        role: Role::Idle,
                    })
                    .add_systems(Startup, move |session: ResMut<SteamSession>| {
                        let session = session.into_inner();
                        if host {
                            session.host_lobby(&SteamLobbySettings::default());
                        } else if let Some(lobby) = lobby {
                            session.steam.join(LobbyId::from_raw(lobby));
                            session.role = Role::Joining;
                        } else if mode != NetMode::Menu {
                            info!(
                                "Waiting for a Steam invite (or start with --steam-host to host)"
                            );
                        }
                    })
                    .add_systems(PreUpdate, run_steam)
                    .add_systems(
                        Update,
                        leave_ended_session.run_if(resource_added::<SessionEnded>),
                    );
            }
            Err(error) if mode == NetMode::Menu => {
                info!("Steam is not available: {error}");
                app.insert_resource(SteamUnavailable(error.clone()));
            }
            Err(error) => {
                error!("{error}");
                warn!("Hosting over UDP on port {DEFAULT_PORT} instead (others join with --join)");
                app.insert_resource(SteamUnavailable(error.clone()))
                    .add_systems(Startup, |mut commands: Commands| {
                        stubby_net::host(&mut commands, DEFAULT_PORT)
                    });
            }
        }
    }
}

/// The running Steam connection and this player's part in a session.
#[derive(Resource)]
pub struct SteamSession {
    pub steam: Steam,
    /// The lobby this player is in, once in one.
    pub lobby: Option<LobbyId>,
    pub role: Role,
}

impl SteamSession {
    /// Create a lobby; once Steam has made it, the server starts and the invite dialog
    /// opens ([`Event::LobbyCreated`]).
    pub fn host_lobby(&mut self, settings: &SteamLobbySettings) {
        self.steam
            .host(u32::from(settings.max_members), settings.friends_only);
        self.role = Role::Hosting;
    }
}

/// Why Steam could not start, for the menu to show. Present instead of [`SteamSession`].
#[derive(Resource, Debug, Clone)]
pub struct SteamUnavailable(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Not in a session: an invite will join one.
    Idle,
    /// Creating, then running, a lobby and its server.
    Hosting,
    /// Entering a lobby, then playing on its owner's server.
    Joining,
}

/// Pump Steam and act on what it reports.
fn run_steam(mut session: ResMut<SteamSession>, mut commands: Commands) {
    let session = &mut *session;
    session.steam.run_callbacks();
    for event in session.steam.events() {
        match event {
            Event::LobbyCreated(lobby) => {
                session.lobby = Some(lobby);
                session.steam.publish(lobby, &this_build());
                let server = commands
                    .spawn((
                        Name::new("Steam server"),
                        SteamEndpoint {
                            target: ListenTarget::Peer {
                                virtual_port: VIRTUAL_PORT,
                            },
                            config: SessionConfig::default(),
                        },
                        Server,
                    ))
                    .id();
                commands.trigger(server::Start { entity: server });
                stubby_net::host_on(&mut commands, server);
                info!(
                    "Hosting Steam lobby {}; invite friends from the overlay",
                    lobby.raw()
                );
                session.steam.invite(lobby);
            }
            Event::LobbyFailed(why) => {
                error!("{why}");
                session.role = Role::Idle;
                commands.insert_resource(SessionEnded(why));
            }
            Event::LobbyEntered(lobby) => {
                if session.role == Role::Hosting {
                    continue; // our own lobby, entered on creation
                }
                match session.steam.check(lobby, &this_build()) {
                    Ok(owner) if owner == session.steam.me() => {}
                    Ok(owner) => {
                        session.lobby = Some(lobby);
                        session.role = Role::Joining;
                        let client = commands
                            .spawn((
                                Name::new("Steam client"),
                                client::Client,
                                ReplicationReceiver,
                                Link::default(),
                                SteamClientIo {
                                    target: ConnectTarget::Peer {
                                        steam_id: owner,
                                        virtual_port: VIRTUAL_PORT,
                                    },
                                    config: SessionConfig::default(),
                                },
                            ))
                            .id();
                        commands.trigger(client::Connect { entity: client });
                        info!("Joining {owner:?}'s game (lobby {})", lobby.raw());
                    }
                    Err(refusal) => {
                        let message = format!("Can't join that game: {refusal}");
                        error!("{message}");
                        session.steam.leave(lobby);
                        session.role = Role::Idle;
                        commands.insert_resource(SessionEnded(message));
                    }
                }
            }
            Event::LobbyRefused(lobby, why) => {
                let message = format!("Can't join that Steam lobby: {why}.");
                error!("{message} (lobby {})", lobby.raw());
                session.role = Role::Idle;
                commands.insert_resource(SessionEnded(message));
            }
            Event::JoinRequested { lobby, friend } => {
                if session.role != Role::Idle {
                    warn!("Invite from {friend:?} ignored: already in a game (quit it first)");
                    continue;
                }
                session.role = Role::Joining;
                session.steam.join(lobby);
            }
        }
    }
}

/// A joined session ended (`SessionEnded`): leave its lobby, so the next invite can be
/// accepted.
fn leave_ended_session(mut session: ResMut<SteamSession>) {
    if session.role != Role::Joining {
        return;
    }
    if let Some(lobby) = session.lobby.take() {
        session.steam.leave(lobby);
    }
    session.role = Role::Idle;
}

/// The lobby to rejoin with the host's world, when this player joined over Steam (it has
/// no address to rejoin by).
pub fn rejoin_lobby(session: Option<&SteamSession>) -> Option<u64> {
    session
        .filter(|s| s.role == Role::Joining)
        .and_then(|s| s.lobby)
        .map(|lobby| lobby.raw())
}
