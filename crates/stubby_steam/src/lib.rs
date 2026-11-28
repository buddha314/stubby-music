//! Steam for the game, without Bevy: start-up with clear errors, the callback pump,
//! lobbies, the build check, invites, rich presence and the `+connect_lobby` launch
//! argument. The Bevy side (`stubby_game::multiplayer::steam`) runs lightyear over Steam P2P
//! on the lobby this sets up; the template us-outpost-31's `docs/MULTIPLAYER.md` describes the session flow.
//!
//! Steam calls back through [`Steam::run_callbacks`]; what happened comes out of
//! [`Steam::events`].

use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender, channel},
};

pub use steamworks;
use steamworks::{
    CallbackHandle, ChatRoomEnterResponse, Client, GameLobbyJoinRequested, LobbyEnter, LobbyId,
    LobbyType, SteamAPIInitError, SteamId,
};

/// Lobby data keys the host publishes and a joining player checks.
pub const BUILD_KEY: &str = "build";
pub const PROTOCOL_KEY: &str = "protocol";

/// What must match between host and client for them to play together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    /// The game's build (version and commit, say). A different build may simulate
    /// differently, which prediction cannot survive.
    pub id: String,
    /// The network protocol id (`stubby_core::multiplayer::PROTOCOL_ID`).
    pub protocol: u64,
}

/// Why a lobby cannot be joined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The host runs another build.
    OtherBuild { host: String, ours: String },
    /// The host speaks another protocol.
    OtherProtocol { host: u64, ours: u64 },
    /// The lobby has no build data: not a lobby of this game, or not ready yet.
    NotThisGame,
    /// Steam refused entry.
    Entry(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::OtherBuild { host, ours } => write!(
                f,
                "the host is running build {host} and you are running {ours}; update so both match"
            ),
            Refusal::OtherProtocol { host, ours } => write!(
                f,
                "the host's network protocol is {host:#x} and yours is {ours:#x}; update so both match"
            ),
            Refusal::NotThisGame => {
                write!(f, "that lobby is not a game of this build (no build data)")
            }
            Refusal::Entry(why) => write!(f, "Steam would not let you into the lobby: {why}"),
        }
    }
}

/// Check a host's published build against ours.
pub fn check_build(
    host_build: Option<&str>,
    host_protocol: Option<&str>,
    ours: &Build,
) -> Result<(), Refusal> {
    let (Some(host_build), Some(host_protocol)) = (host_build, host_protocol) else {
        return Err(Refusal::NotThisGame);
    };
    let host_protocol = u64::from_str_radix(host_protocol.trim_start_matches("0x"), 16)
        .map_err(|_| Refusal::NotThisGame)?;
    if host_protocol != ours.protocol {
        return Err(Refusal::OtherProtocol {
            host: host_protocol,
            ours: ours.protocol,
        });
    }
    if host_build != ours.id {
        return Err(Refusal::OtherBuild {
            host: host_build.to_string(),
            ours: ours.id.clone(),
        });
    }
    Ok(())
}

/// The lobby Steam asked us to join at launch: `+connect_lobby <id>`, which Steam
/// adds when a player starts the game from an invite or "Join game".
pub fn connect_lobby_arg(args: impl IntoIterator<Item = String>) -> Result<Option<u64>, String> {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "+connect_lobby" {
            let id = args.next().ok_or("+connect_lobby needs a lobby id")?;
            return id
                .parse()
                .map(Some)
                .map_err(|_| format!("+connect_lobby: `{id}` is not a lobby id"));
        }
    }
    Ok(None)
}

/// Why Steam could not start, in words a player can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartError(pub String);

impl std::fmt::Display for StartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StartError {}

fn start_error(error: SteamAPIInitError) -> StartError {
    StartError(match error {
        SteamAPIInitError::NoSteamClient(detail) => format!(
            "Steam is not running, or you are not logged in. Start Steam, then the game again ({detail})."
        ),
        SteamAPIInitError::VersionMismatch(detail) => {
            format!("The Steam client is older than this game needs; update Steam ({detail}).")
        }
        SteamAPIInitError::FailedGeneric(detail) => format!("Steam could not start: {detail}"),
    })
}

/// Something Steam told us.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Our lobby exists ([`Steam::host`]).
    LobbyCreated(LobbyId),
    /// Creating the lobby failed.
    LobbyFailed(String),
    /// We are in a lobby we asked to join (also fires for our own, as host).
    LobbyEntered(LobbyId),
    /// Steam refused entry to a lobby.
    LobbyRefused(LobbyId, String),
    /// The player accepted an invite or chose "Join game" while the game runs.
    JoinRequested { lobby: LobbyId, friend: SteamId },
}

/// A running Steam connection.
pub struct Steam {
    client: Client,
    events: Mutex<Receiver<Event>>,
    sender: Sender<Event>,
    _callbacks: Vec<CallbackHandle>,
}

impl Steam {
    /// Start Steam for `app_id` (480, Spacewar, for development) and reach Valve's
    /// relay network, which P2P sessions go through.
    pub fn start(app_id: u32) -> Result<Self, StartError> {
        let client = Client::init_app(app_id).map_err(start_error)?;
        client.networking_utils().init_relay_network_access();
        let (sender, events) = channel();
        let joins = sender.clone();
        let entries = sender.clone();
        let callbacks = vec![
            client.register_callback(move |join: GameLobbyJoinRequested| {
                let _ = joins.send(Event::JoinRequested {
                    lobby: join.lobby_steam_id,
                    friend: join.friend_steam_id,
                });
            }),
            client.register_callback(move |entered: LobbyEnter| {
                let event = match entered.chat_room_enter_response {
                    ChatRoomEnterResponse::Success => Event::LobbyEntered(entered.lobby),
                    other => Event::LobbyRefused(entered.lobby, refusal_reason(other)),
                };
                let _ = entries.send(event);
            }),
        ];
        Ok(Self {
            client,
            events: Mutex::new(events),
            sender,
            _callbacks: callbacks,
        })
    }

    /// The steamworks client, for lightyear's Steam transport.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// This player's Steam id.
    pub fn me(&self) -> SteamId {
        self.client.user().steam_id()
    }

    /// Deliver Steam's callbacks; call once a frame.
    pub fn run_callbacks(&self) {
        self.client.run_callbacks();
    }

    /// What happened since the last call.
    pub fn events(&self) -> Vec<Event> {
        match self.events.lock() {
            Ok(events) => events.try_iter().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Create a lobby for up to `max_members` players: friends-only, or public.
    /// Answers with [`Event::LobbyCreated`] or [`Event::LobbyFailed`].
    pub fn host(&self, max_members: u32, friends_only: bool) {
        let sender = self.sender.clone();
        let kind = if friends_only {
            LobbyType::FriendsOnly
        } else {
            LobbyType::Public
        };
        self.client
            .matchmaking()
            .create_lobby(kind, max_members, move |result| {
                let _ = sender.send(match result {
                    Ok(lobby) => Event::LobbyCreated(lobby),
                    Err(error) => {
                        Event::LobbyFailed(format!("Steam could not create a lobby: {error}"))
                    }
                });
            });
    }

    /// Publish our build on our lobby, and let friends join it from Steam (rich
    /// presence makes "Join game" start the game with `+connect_lobby`).
    pub fn publish(&self, lobby: LobbyId, build: &Build) {
        let matchmaking = self.client.matchmaking();
        matchmaking.set_lobby_data(lobby, BUILD_KEY, &build.id);
        matchmaking.set_lobby_data(lobby, PROTOCOL_KEY, &format!("{:#x}", build.protocol));
        self.client
            .friends()
            .set_rich_presence("connect", Some(&format!("+connect_lobby {}", lobby.raw())));
    }

    /// Open Steam's overlay to invite friends to `lobby`.
    pub fn invite(&self, lobby: LobbyId) {
        self.client.friends().activate_invite_dialog(lobby);
    }

    /// Ask to join `lobby`; answers with [`Event::LobbyEntered`] or
    /// [`Event::LobbyRefused`].
    pub fn join(&self, lobby: LobbyId) {
        self.client.matchmaking().join_lobby(lobby, |_| {});
    }

    /// Leave `lobby`, and stop offering it to friends.
    pub fn leave(&self, lobby: LobbyId) {
        self.client.matchmaking().leave_lobby(lobby);
        self.client.friends().clear_rich_presence();
    }

    /// Who is in `lobby`, with their Steam names, owner first.
    pub fn members(&self, lobby: LobbyId) -> Vec<(SteamId, String)> {
        let matchmaking = self.client.matchmaking();
        let owner = matchmaking.lobby_owner(lobby);
        let mut members: Vec<_> = matchmaking
            .lobby_members(lobby)
            .into_iter()
            .map(|id| (id, self.client.friends().get_friend(id).name()))
            .collect();
        members.sort_by_key(|(id, _)| *id != owner);
        members
    }

    /// Whether we may play in `lobby`, and with whom: its owner hosts the session.
    pub fn check(&self, lobby: LobbyId, build: &Build) -> Result<SteamId, Refusal> {
        let matchmaking = self.client.matchmaking();
        check_build(
            matchmaking.lobby_data(lobby, BUILD_KEY).as_deref(),
            matchmaking.lobby_data(lobby, PROTOCOL_KEY).as_deref(),
            build,
        )?;
        Ok(matchmaking.lobby_owner(lobby))
    }
}

fn refusal_reason(response: ChatRoomEnterResponse) -> String {
    match response {
        ChatRoomEnterResponse::DoesntExist => "the lobby no longer exists".into(),
        ChatRoomEnterResponse::NotAllowed => "it is not open to you".into(),
        ChatRoomEnterResponse::Full => "the lobby is full".into(),
        ChatRoomEnterResponse::Banned => "you are banned from it".into(),
        ChatRoomEnterResponse::Limited => "your account is limited".into(),
        ChatRoomEnterResponse::RatelimitExceeded => "too many attempts; wait a moment".into(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ours() -> Build {
        Build {
            id: "0.1.0-abc123".into(),
            protocol: 0x6f75,
        }
    }

    #[test]
    fn a_matching_host_is_accepted() {
        assert_eq!(
            check_build(Some("0.1.0-abc123"), Some("0x6f75"), &ours()),
            Ok(())
        );
    }

    #[test]
    fn a_different_build_or_protocol_is_refused_clearly() {
        let build = check_build(Some("0.0.9-old"), Some("0x6f75"), &ours()).unwrap_err();
        assert!(build.to_string().contains("0.0.9-old") && build.to_string().contains("update"));
        let protocol = check_build(Some("0.1.0-abc123"), Some("0x1"), &ours()).unwrap_err();
        assert_eq!(
            protocol,
            Refusal::OtherProtocol {
                host: 1,
                ours: 0x6f75
            }
        );
        assert_eq!(check_build(None, None, &ours()), Err(Refusal::NotThisGame));
        assert_eq!(
            check_build(Some("x"), Some("zz"), &ours()),
            Err(Refusal::NotThisGame)
        );
    }

    #[test]
    fn the_launch_lobby_is_read_from_steams_argument() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            connect_lobby_arg(args(&["game", "+connect_lobby", "109775241"])),
            Ok(Some(109775241))
        );
        assert_eq!(connect_lobby_arg(args(&["game", "--steam"])), Ok(None));
        assert!(connect_lobby_arg(args(&["game", "+connect_lobby"])).is_err());
        assert!(connect_lobby_arg(args(&["game", "+connect_lobby", "abc"])).is_err());
    }

    #[test]
    fn starting_without_steam_fails_with_advice_not_a_panic() {
        // CI has no Steam client; a developer machine may. Either way: no panic,
        // and a failure says what to do.
        if let Err(error) = Steam::start(480) {
            assert!(!error.0.is_empty());
        }
    }
}
