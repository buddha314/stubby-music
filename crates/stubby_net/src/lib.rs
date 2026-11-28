//! Multiplayer sessions (add-multiplayer-and-menus 2.1–2.2), ported from the us-outpost-31
//! template's `net.rs`: one player **hosts** (lightyear's server and that player's client in
//! one process) and others **join** over UDP with netcode. The server simulates at a fixed
//! tick; a client predicts what it controls and interpolates the rest.
//!
//! This crate knows nothing about any system: it runs the session, registers the physics
//! replication every peer shares, and reports how a session ended. The game's glue
//! (`stubby_game::multiplayer`) registers what is replicated and spawns players.
//!
//! ```text
//! stubby --host [PORT]           # host on UDP (default 5888) and play
//! stubby --join ADDR[:PORT]      # join a host, e.g. 127.0.0.1 or 192.168.1.20:5888
//! ```

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use avian3d::prelude::*;
use bevy::{app::PluginGroupBuilder, prelude::*};
use lightyear::{
    avian3d::plugin::{AvianReplicationMode, LightyearAvianPlugin},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use stubby_core::multiplayer::{DEFAULT_PORT, PROTOCOL_ID, TICK_HZ};

pub use lightyear;

/// netcode's connect-token key. A fixed key is fine for UDP development and LAN sessions;
/// Steam sessions (phase 5) authenticate with Steam instead.
const DEV_PRIVATE_KEY: [u8; 32] = [0; 32];

/// How this process takes part.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NetMode {
    /// Single player: no networking at all.
    #[default]
    Offline,
    /// The main menu chooses (Play, Host or Join): both roles are loaded so either can
    /// start in this process.
    Menu,
    /// Run the server and play on it.
    Host { port: u16 },
    /// Connect to a host.
    Join { server: SocketAddr },
}

impl NetMode {
    /// `--host [PORT]` or `--join ADDR[:PORT]`; otherwise `fallback` (the binaries pass
    /// `Menu` when the main menu shows, `Offline` for reviews and tests).
    pub fn from_args(
        args: impl IntoIterator<Item = String>,
        fallback: NetMode,
    ) -> Result<Self, String> {
        let mut args = args.into_iter().peekable();
        let mut mode = fallback;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => {
                    let port = match args.peek() {
                        Some(next) if !next.starts_with("--") => {
                            let port = args.next().unwrap_or_default();
                            port.parse()
                                .map_err(|_| format!("--host: `{port}` is not a port"))?
                        }
                        _ => DEFAULT_PORT,
                    };
                    mode = NetMode::Host { port };
                }
                "--join" => {
                    let addr = args
                        .next()
                        .ok_or("--join needs an address, e.g. 127.0.0.1")?;
                    mode = NetMode::Join {
                        server: parse_server(&addr)?,
                    };
                }
                _ => {}
            }
        }
        Ok(mode)
    }

    pub fn is_networked(self) -> bool {
        self != NetMode::Offline
    }
}

/// A host's address as a player types it: `IP` (the default port) or `IP:PORT`.
pub fn parse_server(addr: &str) -> Result<SocketAddr, String> {
    if let Ok(full) = addr.parse::<SocketAddr>() {
        return Ok(full);
    }
    addr.parse::<IpAddr>()
        .map(|ip| SocketAddr::new(ip, DEFAULT_PORT))
        .map_err(|_| format!("`{addr}` is not an IP address or IP:port"))
}

/// avian, set up for `mode`. Networked play hands Position/Transform sync to lightyear and
/// turns off sleeping, which breaks rollback. Add it before the game's plugins.
pub fn physics_plugins(mode: NetMode) -> PluginGroupBuilder {
    let physics = PhysicsPlugins::default().build();
    if !mode.is_networked() {
        return physics;
    }
    physics
        .disable::<PhysicsTransformPlugin>()
        .disable::<PhysicsInterpolationPlugin>()
        .disable::<IslandPlugin>()
        .disable::<IslandSleepingPlugin>()
}

/// The session for the mode in the [`NetMode`] resource (offline: nothing). Add it before
/// the game's glue registers its protocol.
pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        let mode = *app
            .world_mut()
            .get_resource_or_insert_with(NetMode::default);
        let server = match mode {
            NetMode::Offline => return,
            NetMode::Host { .. } | NetMode::Menu => true,
            NetMode::Join { .. } => false,
        };
        if server {
            app.add_plugins(ServerSessionPlugin);
        }
        app.add_plugins((
            ClientSessionPlugin,
            PhysicsReplicationPlugin,
            SharedEntitiesPlugin,
        ));
        match mode {
            NetMode::Host { port } => {
                app.add_systems(Startup, move |mut commands: Commands| {
                    host(&mut commands, port);
                });
            }
            NetMode::Join { server } => {
                app.add_systems(Startup, move |mut commands: Commands| {
                    join(&mut commands, server, random_client_id());
                });
            }
            _ => {}
        }
    }
}

/// The fixed tick every peer shares.
pub fn tick_duration() -> Duration {
    Duration::from_secs_f64(1.0 / TICK_HZ)
}

/// The server side of a session: lightyear's server, replicating to every link.
pub struct ServerSessionPlugin;

impl Plugin for ServerSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lightyear::prelude::server::ServerPlugins {
            tick_duration: tick_duration(),
        })
        .add_observer(accept_link);
    }
}

/// The client side of a session: lightyear's client, and reporting how a session ended.
pub struct ClientSessionPlugin;

impl Plugin for ClientSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(lightyear::prelude::client::ClientPlugins {
            tick_duration: tick_duration(),
        })
        .add_observer(remember_connecting)
        .add_observer(remember_connected)
        .add_observer(report_disconnect);
    }
}

/// What every peer registers alike for physics: lightyear's avian replication, and the
/// fix that keeps a rollback honest.
pub struct PhysicsReplicationPlugin;

impl Plugin for PhysicsReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LightyearAvianPlugin {
            // ahoy moves characters by writing Transform in the fixed step.
            replication_mode: AvianReplicationMode::Position {
                sync_to_transform: true,
            },
            ..default()
        });
        // lightyear restores Position on rollback, but with `sync_to_transform` each physics
        // tick starts by importing Transform, which a rollback leaves stale (found in the
        // template, 2026-11-27).
        app.add_systems(
            FixedPreUpdate,
            transform_follows_position.after(PredictionSystems::SnapToConfirmed),
        );
    }
}

/// At the start of every tick, put each body's Transform where its Position is. Normally
/// they already agree; after a rollback restores (or snaps) Position, this keeps the stale
/// Transform from overwriting it on the first replayed tick.
fn transform_follows_position(
    mut bodies: Query<(&Position, &Rotation, &mut Transform), (With<RigidBody>, Without<ChildOf>)>,
) {
    for (position, rotation, mut transform) in &mut bodies {
        if transform.translation != position.0 {
            transform.translation = position.0;
        }
        if transform.rotation != rotation.0 {
            transform.rotation = rotation.0;
        }
    }
}

// --- Shared entities: the Replication bridge (design D3) --- //

/// Marks an entity the host simulates and every peer shares. Every peer spawns its own
/// copy of the world (the same seed and scene); the host's copy is replicated, and each
/// client's copy hands its looks over to it (adoption) and goes. `interpolated` entities
/// are shown smoothly; the others are predicted, so a client sees its own pushes at once.
/// A reflected component: it can be attached in the editor.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct Shared {
    pub interpolated: bool,
}

/// Pairs a shared entity with every peer's own copy of it. Authored entities take their
/// scene id (`SceneNodeId`); generated ones a key from what generated them
/// ([`NetKey::of`]), the same on every peer.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetKey(pub u64);

impl NetKey {
    /// A key from stable bytes (a placement's key, serialized): 64-bit FNV-1a, the same in
    /// every process, unlike Rust's per-process hashers.
    pub fn of(bytes: &[u8]) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        Self(hash)
    }
}

/// Shared entities: registered identically by every peer, in both roles.
pub struct SharedEntitiesPlugin;

impl Plugin for SharedEntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.component::<NetKey>().replicate();
        app.add_observer(key_scene_entities)
            .add_observer(joined_a_host)
            .add_systems(
                PreUpdate,
                (replicate_shared, hold_local_copies, adopt_local_copies),
            );
    }
}

/// Authored entities marked `Shared` take their scene id as their key.
fn key_scene_entities(
    added: On<Add<Shared>>,
    ids: Query<&jackdaw_scene_types::SceneNodeId, Without<NetKey>>,
    mut commands: Commands,
) {
    if let Ok(id) = ids.get(added.entity) {
        commands.entity(added.entity).insert(NetKey(id.0));
    }
}

/// Present once this process has joined someone else's server: from then on its own
/// copies of shared entities wait for the host's. A host never gets it, so its own
/// entities keep simulating (the template's bridge froze them on a host that also had the
/// client role; found 2026-11-27).
#[derive(Resource)]
pub struct JoinedHost;

fn joined_a_host(
    added: On<Add<Connected>>,
    remote_clients: Query<(), (With<lightyear::prelude::client::Client>, Without<LinkOf>)>,
    mut commands: Commands,
) {
    if remote_clients.contains(added.entity) {
        commands.insert_resource(JoinedHost);
    }
}

/// On a running server, shared entities are replicated to every client.
fn replicate_shared(
    servers: Query<(), With<lightyear::prelude::server::Started>>,
    shared: Query<(Entity, &Shared), (With<NetKey>, Without<Replicate>)>,
    mut commands: Commands,
) {
    if servers.is_empty() {
        return;
    }
    for (entity, shared) in &shared {
        let mut entity = commands.entity(entity);
        entity.insert(Replicate::to_clients(NetworkTarget::All));
        if shared.interpolated {
            entity.insert(InterpolationTarget::to_clients(NetworkTarget::All));
        } else {
            entity.insert(PredictionTarget::to_clients(NetworkTarget::All));
        }
    }
}

/// On a joined client, its own copy of a shared entity waits for the host's, without
/// simulating.
#[derive(Component)]
struct LocalCopy;

fn hold_local_copies(
    joined: Option<Res<JoinedHost>>,
    copies: Query<
        Entity,
        (
            With<Shared>,
            Without<LocalCopy>,
            Without<lightyear::prelude::client::Remote>,
        ),
    >,
    mut commands: Commands,
) {
    if joined.is_none() {
        return;
    }
    for copy in &copies {
        commands.entity(copy).insert((LocalCopy, RigidBodyDisabled));
    }
}

/// The host's copy of an entity arrived: move this peer's copy's looks, collider and
/// body onto it (keeping the host's state), and drop the copy. Components move by cloning:
/// one that is not `Clone` (or reflect-clonable) is silently left behind with the copy. An interpolated body
/// follows the host, so here it is kinematic.
fn adopt_local_copies(
    arrived: Query<
        (Entity, &NetKey, Has<Interpolated>),
        (With<lightyear::prelude::client::Remote>, Without<Adopted>),
    >,
    copies: Query<(Entity, &NetKey, Option<&Children>), With<LocalCopy>>,
    mut commands: Commands,
) {
    for (replicated, key, interpolated) in &arrived {
        let Some((copy, _, children)) = copies.iter().find(|(_, k, _)| *k == key) else {
            continue;
        };
        commands
            .entity(copy)
            .clone_with_opt_out(replicated, |builder| {
                // Deny exactly these, not the components requiring them too (the body
                // and collider require Position and Transform).
                builder.without_required_by_components(|builder| {
                    builder.deny::<(
                        // The host's state, and the bridge's own markers.
                        (Position, Rotation, LinearVelocity, AngularVelocity),
                        (
                            Transform,
                            GlobalTransform,
                            NetKey,
                            LocalCopy,
                            RigidBodyDisabled,
                        ),
                        // Relationships point at the copy; cloning them would tie the
                        // host's entity to it (despawning the copy would take it too).
                        (ChildOf, Children, ColliderOf, RigidBodyColliders),
                    )>();
                });
            });
        // The copy's children (an authored model's meshes, say) move over.
        for &child in children.into_iter().flatten() {
            commands.entity(child).insert(ChildOf(replicated));
        }
        commands.entity(copy).despawn();
        let mut replicated = commands.entity(replicated);
        replicated.insert(Adopted);
        if interpolated {
            replicated.insert(RigidBody::Kinematic);
        }
    }
}

#[derive(Component)]
struct Adopted;

/// Present while lightyear re-simulates past ticks after a misprediction. One-off effects
/// (sounds, particles, UI) must not fire then: the original tick already did. Take it as a
/// parameter and return early when it `is_some()`.
pub type Replaying<'w> = Option<Res<'w, Rollback>>;

/// Marks a player's body and says whose it is.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetPlayer(pub PeerId);

/// Host: listen on `port` and connect this process's own player to the server.
pub fn host(commands: &mut Commands, port: u16) {
    let server = listen(
        commands,
        SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port),
    );
    host_on(commands, server);
}

/// Single player on the networked stack: a host nobody else can reach (loopback, any free
/// port). The menu's Play in a networked build, without a restart.
pub fn play_solo(commands: &mut Commands) {
    let server = listen(commands, SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0));
    host_on(commands, server);
}

/// Connect this process's own player to `server`, a started server in this world: the
/// host's player is a client inside the server's own world.
pub fn host_on(commands: &mut Commands, server: Entity) {
    let local = commands
        .spawn((
            Name::new("Host client"),
            lightyear::prelude::client::Client,
            LinkOf { endpoint: server },
            Link::default(),
            Linked,
        ))
        .id();
    commands.trigger(lightyear::prelude::client::Connect { entity: local });
}

/// Start a server listening for clients over UDP at `addr`.
pub fn listen(commands: &mut Commands, addr: SocketAddr) -> Entity {
    let server = commands
        .spawn((
            server_endpoint(true),
            LocalAddr(addr),
            lightyear::prelude::server::ServerUdpIo,
        ))
        .id();
    commands.trigger(lightyear::prelude::server::Start { entity: server });
    server
}

/// The server half of a host, before its transport: netcode on stubby's protocol.
/// `check_addresses` is off only for in-process test links, which have no real address.
pub fn server_endpoint(check_addresses: bool) -> impl Bundle {
    (
        Name::new("Server"),
        lightyear::prelude::server::NetcodeServer::new(lightyear::prelude::server::NetcodeConfig {
            protocol_id: PROTOCOL_ID,
            private_key: DEV_PRIVATE_KEY,
            server_addr_check: check_addresses,
            ..default()
        }),
    )
}

/// Join: connect to the host at `server` over UDP.
pub fn join(commands: &mut Commands, server: SocketAddr, client_id: u64) {
    let client = commands
        .spawn((
            client_endpoint(server, client_id),
            LocalAddr(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)),
            PeerAddr(server),
            UdpIo::default(),
        ))
        .id();
    commands.trigger(lightyear::prelude::client::Connect { entity: client });
}

/// A joining client, before its transport.
pub fn client_endpoint(server: SocketAddr, client_id: u64) -> impl Bundle {
    let auth = Authentication::Manual {
        server_addr: server,
        client_id,
        private_key: DEV_PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };
    (
        Name::new("Client"),
        lightyear::prelude::client::Client,
        ReplicationReceiver,
        Link::default(),
        lightyear::prelude::client::NetcodeClient::new(
            auth,
            lightyear::prelude::client::NetcodeConfig::default(),
        )
        .expect("netcode client config"),
    )
}

/// A client id for this run. netcode only needs it to be unique in the session.
pub fn random_client_id() -> u64 {
    use std::hash::{BuildHasher, RandomState};
    RandomState::new().hash_one(std::process::id())
}

/// Every link to a client replicates the world to it.
fn accept_link(link: On<Add<LinkOf>>, mut commands: Commands) {
    commands.entity(link.entity).insert(ReplicationSender);
}

/// Why this client's session ended, for the game to show. Inserted when the host leaves or
/// the connection is lost, or when connecting fails.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct SessionEnded(pub String);

/// Whether a client got as far as connecting, so its `Disconnected` (which every client
/// starts with) can be told apart from losing a session.
#[derive(Component, Default)]
struct SessionHistory {
    connected: bool,
}

fn remember_connecting(
    added: On<Add<lightyear::prelude::client::Connecting>>,
    clients: Query<(), (With<lightyear::prelude::client::Client>, Without<LinkOf>)>,
    mut commands: Commands,
) {
    if clients.contains(added.entity) {
        commands
            .entity(added.entity)
            .insert(SessionHistory::default());
    }
}

fn remember_connected(added: On<Add<Connected>>, mut clients: Query<&mut SessionHistory>) {
    if let Ok(mut history) = clients.get_mut(added.entity) {
        history.connected = true;
    }
}

/// Say why a session ended, or never started, in words a player can act on.
fn report_disconnect(
    added: On<Add<lightyear::prelude::client::Disconnected>>,
    clients: Query<(&lightyear::prelude::client::Disconnected, &SessionHistory)>,
    mut commands: Commands,
) {
    let Ok((disconnected, history)) = clients.get(added.entity) else {
        return; // a fresh client, not yet connecting
    };
    let why = format!("{:?}", disconnected.reason);
    let message = if history.connected {
        format!("The session is over: the host left or the connection was lost ({why}).")
    } else {
        format!(
            "Could not join ({why}). Check that the host is running, reachable, and on the same build."
        )
    };
    error!("{message}");
    commands.insert_resource(SessionEnded(message));
    commands.entity(added.entity).remove::<SessionHistory>();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(args: &[&str], fallback: NetMode) -> Result<NetMode, String> {
        NetMode::from_args(args.iter().map(|a| a.to_string()), fallback)
    }

    #[test]
    fn keys_are_the_same_in_every_process() {
        // FNV-1a's published test vectors: stable, not seeded per process.
        assert_eq!(NetKey::of(b"").0, 0xcbf2_9ce4_8422_2325);
        assert_eq!(NetKey::of(b"a").0, 0xaf63_dc4c_8601_ec8c);
        assert_ne!(NetKey::of(b"cell 1"), NetKey::of(b"cell 2"));
    }

    #[test]
    fn launch_arguments_choose_the_session() {
        assert_eq!(mode(&["stubby"], NetMode::Menu), Ok(NetMode::Menu));
        assert_eq!(mode(&["stubby"], NetMode::Offline), Ok(NetMode::Offline));
        assert_eq!(
            mode(&["stubby", "--host"], NetMode::Menu),
            Ok(NetMode::Host { port: DEFAULT_PORT })
        );
        assert_eq!(
            mode(&["stubby", "--host", "7000", "--seed=4"], NetMode::Menu),
            Ok(NetMode::Host { port: 7000 })
        );
        assert_eq!(
            mode(&["stubby", "--join", "127.0.0.1"], NetMode::Menu),
            Ok(NetMode::Join {
                server: "127.0.0.1:5888".parse().unwrap()
            })
        );
        assert!(mode(&["stubby", "--join", "nowhere"], NetMode::Menu).is_err());
        assert!(mode(&["stubby", "--host", "x"], NetMode::Menu).is_err());
    }
}
