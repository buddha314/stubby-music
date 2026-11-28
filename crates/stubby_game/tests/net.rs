//! Multiplayer in one process (add-multiplayer-and-menus 2.2–2.4), as the us-outpost-31
//! template tests it: a server app and client apps joined by in-process crossbeam links
//! (netcode on top, as over UDP), each with stubby's movement and the game's multiplayer
//! glue. Frames advance in real time, one tick apart, because netcode keeps real-time
//! timeouts; each test takes a few seconds.
//!
//! What they check: a joining client gets its own body, predicted and bound to its keys;
//! its inputs move the server's copy and prediction agrees, also over a laggy, lossy link;
//! a host plays on its own server and each sees the other's body.

use core::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship, prelude::*, state::app::StatesPlugin};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use stubby_core::{GroundHeight, LookCapture, ScreenMode};
use stubby_game::{
    controls::Player,
    movement::{MoveBody, MoveControls, MovementPlugin, NetworkedMovement, Sprint},
    multiplayer::{
        ClientPlayersPlugin, ProtocolPlugin, ServerPlayersPlugin, WorldHandshakePlugin,
        WorldMismatch,
    },
    net::{
        self, NetMode, NetPlayer,
        lightyear::{crossbeam::CrossbeamIo, link::RecvLinkConditioner, prelude::*},
    },
    world_generation::WorldConfig,
};

const SERVER_ADDR: SocketAddr = SocketAddr::new(core::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5888);
const TICK: Duration = Duration::from_micros(15_625); // 64 Hz

/// The shipped world with `seed`.
fn world(seed: u64) -> WorldConfig {
    let mut config = WorldConfig::shipped();
    config.seed = seed.to_string();
    config
}

/// What every peer has: headless Bevy, networked physics, a camera to look through, and
/// stubby's movement with the game's protocol. The world is seed 42 unless a test changes it.
fn peer(mode: NetMode) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        StatesPlugin,
        AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::input::InputPlugin,
        bevy::scene::ScenePlugin,
        net::physics_plugins(mode),
    ))
    .init_resource::<GroundHeight>()
    .insert_resource(LookCapture(true))
    .init_resource::<ScreenMode>()
    .insert_resource(NetworkedMovement)
    .insert_resource(world(42));
    let ground = GroundHeight::default();
    app.world_mut().spawn((
        Player::on(&ground),
        stubby_game::controls::spawn_transform(&ground),
    ));
    app
}

struct Session {
    server: App,
    clients: Vec<App>,
    server_entity: Entity,
}

impl Session {
    /// A server app with the game's networking, before it listens.
    fn server_app(host: bool) -> App {
        let mut server = peer(NetMode::Host { port: 0 });
        server.add_plugins(net::ServerSessionPlugin);
        if host {
            server.add_plugins(net::ClientSessionPlugin);
        }
        server.add_plugins((
            net::PhysicsReplicationPlugin,
            net::SharedEntitiesPlugin,
            ProtocolPlugin,
            WorldHandshakePlugin,
            ServerPlayersPlugin,
            MovementPlugin,
        ));
        server
    }

    /// A client app with the game's networking, before it connects.
    fn client_app(seed: u64) -> App {
        let mut client = peer(NetMode::Join {
            server: SERVER_ADDR,
        });
        client.insert_resource(world(seed));
        client.add_plugins((
            net::ClientSessionPlugin,
            net::PhysicsReplicationPlugin,
            net::SharedEntitiesPlugin,
            ProtocolPlugin,
            WorldHandshakePlugin,
            ClientPlayersPlugin,
            MovementPlugin,
        ));
        client.finish();
        client.cleanup();
        client
    }

    /// A server, plus (with `host`) a player of its own.
    fn new(host: bool) -> Self {
        let mut server = Self::server_app(host);
        let server_entity = server.world_mut().spawn(net::server_endpoint(false)).id();
        server.finish();
        server.cleanup();
        server.world_mut().trigger(server::Start {
            entity: server_entity,
        });
        if host {
            let local = server
                .world_mut()
                .spawn((
                    client::Client,
                    LinkOf {
                        endpoint: server_entity,
                    },
                    Link::default(),
                    Linked,
                ))
                .id();
            server
                .world_mut()
                .trigger(client::Connect { entity: local });
        }
        Self {
            server,
            clients: Vec::new(),
            server_entity,
        }
    }

    /// A client joining over an in-process link, optionally laggy and lossy.
    fn join(&mut self, conditions: Option<LinkConditionerConfig>) -> usize {
        self.join_with_seed(conditions, 42)
    }

    fn join_with_seed(&mut self, conditions: Option<LinkConditionerConfig>, seed: u64) -> usize {
        let id = self.clients.len();
        let mut client = Self::client_app(seed);
        let (client_io, server_io) = CrossbeamIo::new_pair();
        let link = || match &conditions {
            Some(c) => Link::default().with_conditioner(RecvLinkConditioner::new(c.clone())),
            None => Link::default(),
        };
        let entity = client
            .world_mut()
            .spawn((
                net::client_endpoint(SERVER_ADDR, 100 + id as u64),
                client_io,
            ))
            .insert(link())
            .id();
        self.server.world_mut().spawn((
            LinkOf {
                endpoint: self.server_entity,
            },
            link(),
            PeerAddr(SocketAddr::new(
                Ipv4Addr::LOCALHOST.into(),
                6000 + id as u16,
            )),
            Linked,
            server_io,
        ));
        client.world_mut().trigger(client::Connect { entity });
        self.clients.push(client);
        id
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let next = Instant::now() + TICK;
            for client in &mut self.clients {
                client.update();
            }
            self.server.update();
            std::thread::sleep(next.saturating_duration_since(Instant::now()));
        }
    }

    /// Run until `done`, or fail after `seconds`.
    fn until(&mut self, seconds: f32, what: &str, mut done: impl FnMut(&mut Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs_f32(seconds);
        while !done(self) {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            self.frames(1);
        }
    }

    fn seconds(&mut self, s: f32) {
        self.frames((s / TICK.as_secs_f32()).round() as usize);
    }
}

/// The body `app` controls, once it has one.
fn local_body(app: &mut App) -> Option<Entity> {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, (With<MoveBody>, With<CharacterController>)>()
        .iter(world)
        .next()
}

fn position(app: &App, entity: Entity) -> Vec3 {
    app.world().get::<Position>(entity).unwrap().0
}

fn server_body(server: &mut App, peer: PeerId) -> Entity {
    let world = server.world_mut();
    world
        .query::<(Entity, &NetPlayer)>()
        .iter(world)
        .find(|(_, p)| p.0 == peer)
        .map(|(e, _)| e)
        .expect("the server has the player")
}

fn peer_of(app: &App, body: Entity) -> PeerId {
    app.world().get::<NetPlayer>(body).unwrap().0
}

/// Hold `A` on `body`'s action (as if a key were held), or release it.
fn hold<A: InputAction>(app: &mut App, body: Entity, value: Option<impl Into<ActionValue>>) {
    let world = app.world_mut();
    let action = world
        .query::<(Entity, &ActionOf<MoveControls>, Has<Action<A>>)>()
        .iter(world)
        .find(|(_, of, is_a)| *is_a && of.get() == body)
        .map(|(e, _, _)| e)
        .expect("the body has the action");
    let mock = match value {
        Some(v) => ActionMock::new(TriggerState::Fired, v, MockSpan::Manual),
        None => ActionMock::new(
            TriggerState::None,
            ActionValue::zero(A::Output::DIM),
            MockSpan::Manual,
        ),
    };
    world.entity_mut(action).insert(mock);
}

fn joined(session: &mut Session, client: usize) -> Entity {
    session.until(5.0, "the client's predicted body", |s| {
        local_body(&mut s.clients[client]).is_some()
    });
    local_body(&mut session.clients[client]).unwrap()
}

#[test]
fn a_joining_client_gets_its_own_predicted_bound_body() {
    let mut session = Session::new(false);
    session.join(None);
    let body = joined(&mut session, 0);
    session.seconds(0.3);
    let client = &mut session.clients[0];
    assert!(client.world().get::<Predicted>(body).is_some(), "predicted");
    let world = client.world_mut();
    let bound = world
        .query_filtered::<&ActionOf<MoveControls>, With<Bindings>>()
        .iter(world)
        .filter(|of| of.get() == body)
        .count();
    assert_eq!(
        bound, 4,
        "Movement, Jump, Crouch and Sprint are bound (Look is fed from the camera)"
    );
    let peer = peer_of(client, body);
    let on_server = server_body(&mut session.server, peer);
    assert!(
        session.server.world().get::<MoveBody>(on_server).is_none(),
        "the server does not steer a remote player's body"
    );
}

#[test]
fn client_input_moves_the_server_body_and_prediction_agrees() {
    let mut session = Session::new(false);
    session.join(None);
    let body = joined(&mut session, 0);
    session.seconds(0.5); // stand still while the timelines sync
    let peer = peer_of(&session.clients[0], body);
    let on_server = server_body(&mut session.server, peer);
    let start = position(&session.server, on_server);

    // The camera faces 75 degrees from -Z at spawn: the server agrees only if the client's
    // Look reaches it (it once did not; see multiplayer::send_look).
    hold::<Movement>(&mut session.clients[0], body, Some(Vec2::Y));
    hold::<Sprint>(&mut session.clients[0], body, Some(true));
    session.seconds(1.0);
    hold::<Movement>(&mut session.clients[0], body, None::<Vec2>);
    hold::<Sprint>(&mut session.clients[0], body, None::<bool>);
    session.seconds(1.5); // stop, and let the server's last state arrive

    let server_at = position(&session.server, on_server);
    let client_at = position(&session.clients[0], body);
    let moved = server_at - start;
    assert!(
        Vec2::new(moved.x, moved.z).length() > 5.0,
        "the server ran it (sprint arrives too): {moved:?}"
    );
    assert!(
        server_at.distance(client_at) < 0.1,
        "prediction agrees: server {server_at:?}, client {client_at:?}"
    );
}

#[test]
fn prediction_converges_over_a_laggy_lossy_link() {
    let mut session = Session::new(false);
    session.join(Some(LinkConditionerConfig {
        incoming_latency: Duration::from_millis(50),
        incoming_jitter: Duration::from_millis(10),
        good_loss: 0.03,
        ..default()
    }));
    let body = joined(&mut session, 0);
    session.seconds(0.8);
    let peer = peer_of(&session.clients[0], body);
    let on_server = server_body(&mut session.server, peer);
    let start = position(&session.server, on_server);

    hold::<Movement>(&mut session.clients[0], body, Some(Vec2::new(0.6, 0.8)));
    session.seconds(0.6);
    hold::<Jump>(&mut session.clients[0], body, Some(true));
    session.seconds(0.1);
    hold::<Jump>(&mut session.clients[0], body, None::<bool>);
    session.seconds(0.6);
    hold::<Movement>(&mut session.clients[0], body, None::<Vec2>);
    session.seconds(2.0);

    let server_at = position(&session.server, on_server);
    let client_at = position(&session.clients[0], body);
    assert!(
        server_at.distance(start) > 3.0,
        "it moved: {start:?} -> {server_at:?}"
    );
    assert!(
        server_at.distance(client_at) < 0.1,
        "converged: server {server_at:?}, client {client_at:?}"
    );
}

#[test]
fn a_host_plays_on_its_own_server_while_a_client_joins() {
    let mut session = Session::new(true);
    session.until(3.0, "the host's own body", |s| {
        local_body(&mut s.server).is_some()
    });
    let host_body = local_body(&mut session.server).unwrap();
    session.join(None);
    let client_body = joined(&mut session, 0);
    session.seconds(0.5);

    let start = position(&session.server, host_body);
    hold::<Movement>(&mut session.server, host_body, Some(Vec2::Y));
    session.seconds(0.8);
    hold::<Movement>(&mut session.server, host_body, None::<Vec2>);
    session.seconds(1.0);
    assert!(
        position(&session.server, host_body).distance(start) > 2.0,
        "the host walks"
    );

    // The client sees the host's body (interpolated), and the host the client's.
    let host_peer = peer_of(&session.server, host_body);
    let world = session.clients[0].world_mut();
    let seen = world
        .query::<(Entity, &NetPlayer)>()
        .iter(world)
        .find(|(_, p)| p.0 == host_peer)
        .map(|(e, _)| e)
        .expect("the client sees the host's body");
    assert!(
        session.clients[0]
            .world()
            .get::<Interpolated>(seen)
            .is_some()
    );
    let client_peer = peer_of(&session.clients[0], client_body);
    server_body(&mut session.server, client_peer);

    // The camera follows each player's own body.
    let camera_of = |app: &mut App| {
        let world = app.world_mut();
        world
            .query_filtered::<&Transform, With<Player>>()
            .iter(world)
            .next()
            .unwrap()
            .translation
    };
    let host_camera = camera_of(&mut session.server);
    let host_at = position(&session.server, host_body);
    assert!(
        Vec2::new(host_camera.x - host_at.x, host_camera.z - host_at.z).length() < 0.05,
        "the host's camera is at its body"
    );
}

#[test]
fn a_client_learns_the_hosts_world_and_notices_when_it_differs() {
    let mut session = Session::new(false);
    session.join(None); // seed 42, as the host
    session.join_with_seed(None, 7);
    joined(&mut session, 0);
    joined(&mut session, 1);
    session.until(3.0, "the host's world at the second client", |s| {
        s.clients[1].world().contains_resource::<WorldMismatch>()
    });
    let mismatch = session.clients[1].world().resource::<WorldMismatch>();
    assert_eq!(mismatch.host.seed, "42", "the host's world, to rejoin with");
    assert!(
        !session.clients[0]
            .world()
            .contains_resource::<WorldMismatch>(),
        "the same world needs nothing"
    );
}

/// Something only a peer's own copy has, to see that adoption moved it over. Adoption
/// clones components, so it must be `Clone`: one that is not is silently left behind.
#[derive(Component, Clone)]
struct Looks;

fn spawn_shared_cube(app: &mut App, interpolated: bool) {
    app.world_mut().spawn((
        net::Shared { interpolated },
        net::NetKey(1),
        Looks,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Transform::from_xyz(3.0, 4.0, 0.0),
    ));
}

#[test]
fn a_shared_entity_is_simulated_by_the_host_and_adopted_by_clients() {
    for interpolated in [false, true] {
        let mut session = Session::new(false);
        spawn_shared_cube(&mut session.server, interpolated);
        let client = session.join(None);
        spawn_shared_cube(&mut session.clients[client], interpolated);
        joined(&mut session, client);
        session.seconds(2.0); // it falls and lands

        let world = session.clients[client].world_mut();
        let cubes: Vec<(Vec3, bool, bool, Option<RigidBody>)> = world
            .query::<(
                &net::NetKey,
                &Position,
                Has<Looks>,
                Has<Collider>,
                Option<&RigidBody>,
            )>()
            .iter(world)
            .map(|(_, p, looks, collider, body)| (p.0, looks, collider, body.copied()))
            .collect();
        assert_eq!(cubes.len(), 1, "one cube on the client, not two: {cubes:?}");
        let (at, looks, collider, body) = cubes[0];
        assert!(looks && collider, "the local copy's components moved over");
        let expected = if interpolated {
            RigidBody::Kinematic
        } else {
            RigidBody::Dynamic
        };
        assert_eq!(body, Some(expected));

        let world = session.server.world_mut();
        let on_server = world
            .query_filtered::<&Position, With<net::NetKey>>()
            .single(world)
            .unwrap()
            .0;
        assert!(on_server.y < 1.5, "it fell on the host: {on_server:?}");
        assert!(
            at.distance(on_server) < 0.1,
            "interpolated={interpolated}: client {at:?}, host {on_server:?}"
        );
    }
}

/// A host that also has the client role (the main menu's launch) keeps simulating its own
/// shared entities: they are the original, not a copy waiting for a host.
#[test]
fn a_host_keeps_simulating_its_own_shared_entities() {
    let mut session = Session::new(true);
    spawn_shared_cube(&mut session.server, false);
    session.until(3.0, "the host's own body", |s| {
        local_body(&mut s.server).is_some()
    });
    session.join(None);
    joined(&mut session, 0);
    session.seconds(2.0);
    let world = session.server.world_mut();
    let (at, disabled) = world
        .query_filtered::<(&Position, Has<RigidBodyDisabled>), With<net::NetKey>>()
        .single(world)
        .unwrap();
    assert!(!disabled, "the host's own cube is not frozen");
    assert!(at.0.y < 1.5, "it fell on the host: {:?}", at.0);
}

/// Every peer generates the world itself (design D3), so generation must be exact: the
/// same configuration gives the same placements and the same keys, and another seed does
/// not.
#[test]
fn the_same_world_generates_the_same_placements_and_keys() {
    let layout = |seed: u64| {
        world(seed)
            .resolve_layout(&[])
            .expect("the opening resolves")
    };
    let keys = |seed: u64| -> Vec<net::NetKey> {
        layout(seed)
            .placements
            .iter()
            .map(|p| net::NetKey::of(serde_json::to_string(&p.key).unwrap().as_bytes()))
            .collect()
    };
    let (a, b) = (layout(42), layout(42));
    assert!(!a.placements.is_empty(), "the opening places things");
    assert_eq!(a, b, "two generations from one configuration agree");
    assert_eq!(keys(42), keys(42), "and so do their keys");
    let distinct: std::collections::HashSet<_> = keys(42).into_iter().collect();
    assert_eq!(
        distinct.len(),
        a.placements.len(),
        "every placement has its own key"
    );
    assert_ne!(
        layout(42).placements,
        layout(7).placements,
        "another seed, another world"
    );
}
