//! The game's multiplayer glue (add-multiplayer-and-menus 2.2–2.4), ported from the
//! us-outpost-31 template: what stubby's systems replicate, and players' bodies.
//!
//! - Every peer registers the movement inputs (`stubby_player::movement`'s actions on
//!   `MoveControls`) and the player marker, and rolls ahoy's own state back with
//!   `Position` on a misprediction.
//! - The server gives every client that connects a body at the opening, in a ring when
//!   several join. The host's own body is controlled right there; a remote one is
//!   predicted by its owner and interpolated by everyone else.
//! - A client completes its own predicted body (controller, collider) and the camera
//!   follows it; other players' bodies are drawn.
//!
//! The session itself is `stubby_net`. Offline (`NetMode::Offline`) none of this runs.

use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_ahoy::{CharacterControllerState, CharacterLook, input::AccumulatedInput, prelude::*};
use bevy_enhanced_input::prelude::{Action, ActionOf};
use stubby_net::{
    NetMode, NetPlayer,
    lightyear::prelude::{
        input::{
            InputConfig, InputRegistryExt,
            bei::{InputMarker, InputPlugin as BeiInputPlugin},
        },
        *,
    },
};
use stubby_player::{
    controls::{Player, floor},
    movement::{self, Look, MoveBody, MoveControls, NetworkedMovement, Sprint},
};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        let mode = app
            .world()
            .get_resource::<NetMode>()
            .copied()
            .unwrap_or_default();
        if !mode.is_networked() {
            return;
        }
        info!("Multiplayer: {mode:?}");
        app.insert_resource(NetworkedMovement).add_plugins((
            stubby_net::SessionPlugin,
            ProtocolPlugin,
            ClientPlayersPlugin,
        ));
        if matches!(mode, NetMode::Host { .. } | NetMode::Menu) {
            app.add_plugins(ServerPlayersPlugin);
        }
    }
}

/// The server's players: a body for every client that connects.
pub struct ServerPlayersPlugin;

impl Plugin for ServerPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_connected_player);
    }
}

/// A client's players: its own body predicted and followed, the others drawn.
pub struct ClientPlayersPlugin;

impl Plugin for ClientPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(predict_own_player)
            .add_systems(PreUpdate, send_look)
            .add_systems(Update, show_other_players);
    }
}

/// What every peer registers identically: the inputs, the replicated marker, and the
/// character state rolled back on a mismatch.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BeiInputPlugin::<MoveControls>::new(InputConfig {
            // Other players are interpolated, not predicted: they need no inputs.
            rebroadcast_inputs: false,
            ..default()
        }));
        app.register_input_action::<Movement>()
            .register_input_action::<Jump>()
            .register_input_action::<Crouch>()
            .register_input_action::<Sprint>()
            .register_input_action::<Look>();
        app.component::<NetPlayer>().replicate();
        // ahoy's own state is not sent, but must rewind with Position on rollback.
        app.local_rollback::<CharacterControllerState>();
        app.local_rollback::<AccumulatedInput>();
        app.local_rollback::<CharacterLook>();
    }
}

/// Players beyond the first stand in a ring this far from the opening spot.
const RING_RADIUS_M: f32 = 1.5;

/// Where the `index`th player stands: the opening spot, then a ring around it.
pub fn spawn_spot(index: usize) -> Vec2 {
    let opening = stubby_core::OPENING_SPAWN_XZ;
    if index == 0 {
        return opening;
    }
    let angle = index as f32 * 1.3;
    opening + Vec2::new(angle.cos(), angle.sin()) * RING_RADIUS_M
}

/// A client finished connecting: give it a body. The host's own is controlled here (its
/// camera follows it); a remote one is predicted by its client and interpolated by the
/// others.
fn spawn_connected_player(
    connected: On<Add<Connected>>,
    links: Query<(&RemoteId, Has<client::Client>), With<server::ClientOf>>,
    players: Query<(), With<NetPlayer>>,
    cameras: Query<Entity, With<Player>>,
    ground: Res<stubby_core::GroundHeight>,
    mut commands: Commands,
) {
    let link = connected.entity;
    let Ok((remote, is_host)) = links.get(link) else {
        return;
    };
    let peer = remote.0;
    let spot = spawn_spot(players.iter().count());
    let feet = floor(&ground, spot.x, spot.y);
    let mut body = commands.spawn((
        Name::new(format!("Player {peer:?}")),
        NetPlayer(peer),
        movement::body(spot, feet),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::Single(peer)),
        InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(peer)),
        ControlledBy {
            owner: link,
            lifetime: Lifetime::SessionBased,
        },
    ));
    if is_host && let Some(camera) = cameras.iter().next() {
        body.insert(MoveBody { camera });
    }
    let body = body.id();
    movement::spawn_actions(&mut commands, body);
    info!("player {peer:?} joined as {body:?}");
}

/// This client's own body arrived: simulate it here too (predicted), and follow it.
fn predict_own_player(
    added: On<Add<(NetPlayer, Predicted)>>,
    bodies: Query<(), (With<NetPlayer>, With<Predicted>, Without<MoveBody>)>,
    cameras: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    if !bodies.contains(added.entity) {
        return;
    }
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    commands
        .entity(added.entity)
        .insert((movement::body_parts(), MoveBody { camera }));
}

/// lightyear sends an action's state only once it carries `InputMarker`, which it adds
/// when keys are bound. `Look` has no keys (it is fed from the camera), so the body this
/// machine controls gets the marker here; without it the host never learns where a remote
/// player faces (found 2026-11-27: the client ran toward its camera, the host along -Z).
#[allow(clippy::type_complexity)]
fn send_look(
    looks: Query<
        (Entity, &ActionOf<MoveControls>),
        (With<Action<Look>>, Without<InputMarker<MoveControls>>),
    >,
    local: Query<(), With<MoveBody>>,
    mut commands: Commands,
) {
    for (look, of) in &looks {
        if local.contains(of.get()) {
            commands
                .entity(look)
                .insert(InputMarker::<MoveControls>::default());
        }
    }
}

/// Other players are drawn as a body their size, on hosts and clients alike. (Omar's
/// model replaces it when it exists; today only his gloves and shadow do.)
pub fn show_other_players(
    players: Query<Entity, (With<NetPlayer>, Without<MoveBody>, Without<Mesh3d>)>,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut commands: Commands,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    let (radius, height) = movement::BODY_SIZE;
    for player in &players {
        commands.entity(player).insert((
            Mesh3d(meshes.add(Capsule3d::new(radius, height - 2. * radius))),
            MeshMaterial3d(materials.add(Color::srgb(0.55, 0.4, 0.28))),
        ));
    }
}
