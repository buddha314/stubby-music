//! Building in a shared world (task 3.3, design D4): the host decides. A joined client's
//! build becomes a request; the host checks the builder is in reach of the place, takes
//! the cost from that player's pack, and builds it. What is built carries its `Built`
//! record, which replicates to everyone (late joiners too); each peer shows it from the
//! record (model, placed asset) and runs it locally, as it runs everything it hears.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_net::{
    HostChannel, JoinedHost, NetPlayer,
    lightyear::prelude::{server::Started, *},
};

use super::inventory::{HostSays, Pack, pack_of};
use crate::build::{
    BuildRequest, Built, BuiltCount, built_goal, built_looks, pay_for, segment_on, spawn_built,
};
use crate::recipes_plugin::{GoalRequest, PlayerInventory, RecipeCatalog};

/// How far from a player's body the host accepts a build: the farthest aim (10 m), a
/// stream-side snap (up to 7 m more), and a moment's lag.
pub const BUILD_REACH: f32 = 18.0;

/// A client asks the host to build.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Build {
    pub buildable: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub segment: Option<u64>,
}

/// The host built this for the client that asked: its goal step and hint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuiltFor {
    pub buildable: String,
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<Build>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<BuiltFor>()
            .add_direction(NetworkDirection::ServerToClient);
        app.component::<Built>().replicate();
        // Building's own, so a session works without the build menu (tests, a server).
        app.add_message::<BuildRequest>()
            .init_resource::<BuiltCount>();
        app.add_systems(
            Update,
            (
                (serve_builds, replicate_built).chain(),
                (ask_to_build, hear_builds, show_built).run_if(resource_exists::<JoinedHost>),
            ),
        );
    }
}

// --- On the host --- //

/// A client's build: allowed in reach of its body, across a segment the host has (for
/// what sits across a stream), and paid from its pack.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn serve_builds(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<Build>,
        &mut MessageSender<BuiltFor>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    bodies: Query<(&ControlledBy, &avian3d::prelude::Position), With<NetPlayer>>,
    streams: Query<(Entity, &stubby_streams::LaidStream)>,
    catalog: Option<Res<RecipeCatalog>>,
    server: Option<Res<AssetServer>>,
    mut count: ResMut<BuiltCount>,
    mut commands: Commands,
) {
    let (Some(catalog), Some(server)) = (catalog, server) else {
        return;
    };
    for (link, mut receiver, mut built_for, mut says) in &mut links {
        for build in receiver.receive() {
            let Some(at) = bodies
                .iter()
                .find(|(owner, _)| owner.owner == link)
                .map(|(_, position)| position.0)
            else {
                continue;
            };
            let place = Transform::from_translation(build.translation.into())
                .with_rotation(Quat::from_array(build.rotation));
            if place.translation.distance(at) > BUILD_REACH {
                says.send::<HostChannel>(HostSays::Line("Too far away to build there.".into()));
                continue;
            }
            let on = build
                .segment
                .and_then(|key| segment_on(key, place.translation, &streams));
            if build.segment.is_some() && on.is_none() {
                says.send::<HostChannel>(HostSays::Line("That stream is not there.".into()));
                continue;
            }
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            match pay_for(&catalog, &build.buildable, &mut pack.items) {
                Ok(definition) => {
                    spawn_built(&mut commands, &server, &mut count, &definition, place, on);
                    built_for.send::<HostChannel>(BuiltFor {
                        buildable: definition.id,
                    });
                }
                Err(said) => says.send::<HostChannel>(HostSays::Line(said)),
            }
        }
    }
}

/// Everything built on a running server, by anyone, is everyone's.
fn replicate_built(
    servers: Query<(), With<Started>>,
    built: Query<Entity, (With<Built>, Without<Replicate>)>,
    mut commands: Commands,
) {
    if servers.is_empty() {
        return;
    }
    for entity in &built {
        commands
            .entity(entity)
            .insert(Replicate::to_clients(NetworkTarget::All));
    }
}

// --- On a joined client --- //

fn ask_to_build(
    mut requests: MessageReader<BuildRequest>,
    mut senders: Query<&mut MessageSender<Build>, (With<client::Client>, Without<LinkOf>)>,
) {
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        sender.send::<HostChannel>(Build {
            buildable: request.buildable.clone(),
            translation: request.place.translation.into(),
            rotation: request.place.rotation.to_array(),
            segment: request.segment,
        });
    }
}

/// The host built what this player asked for: its goal step, and its hint.
fn hear_builds(
    mut receivers: Query<&mut MessageReceiver<BuiltFor>, (With<client::Client>, Without<LinkOf>)>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    mut goals: MessageWriter<GoalRequest>,
) {
    for mut receiver in &mut receivers {
        for BuiltFor { buildable } in receiver.receive() {
            goals.write(built_goal(&buildable));
            if let Some(hint) = catalog
                .as_ref()
                .and_then(|c| c.0.buildable(&buildable))
                .and_then(|b| b.built_hint.clone())
            {
                player.feedback = hint;
            }
        }
    }
}

/// Something the host built arrived: show it here, from its record.
#[allow(clippy::type_complexity)]
fn show_built(
    arrived: Query<(Entity, &Built), (With<client::Remote>, Without<stubby_core::PlacedAsset>)>,
    streams: Query<(Entity, &stubby_streams::LaidStream)>,
    catalog: Option<Res<RecipeCatalog>>,
    server: Option<Res<AssetServer>>,
    mut commands: Commands,
) {
    let (Some(catalog), Some(server)) = (catalog, server) else {
        return;
    };
    for (entity, built) in &arrived {
        let Some(definition) = catalog.0.buildable(&built.buildable) else {
            warn!(
                "The host built {}, which these rules do not have",
                built.buildable
            );
            continue;
        };
        let mut shown = commands.entity(entity);
        shown.insert(built_looks(&server, definition, built));
        if let Some(on) = built
            .segment
            .and_then(|key| segment_on(key, built.place().translation, &streams))
        {
            shown.insert(on);
        }
    }
}
