//! Streams in a shared world (task 3.3, design D4): the host decides. A joined client's
//! lay or dismantle becomes a request; the host checks the player is in reach, the port is
//! free and the plan sound, takes the cost from (or refunds it to) that player's pack, and
//! lays or splits the stream. Every laid stream carries a `StreamRecord` that replicates to
//! everyone (late joiners too); each peer bends its meshes from the record and runs its
//! water and music locally. A dismantle respawns streams on the host, and the despawns and
//! spawns replicate.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_core::{AssetState, PlacedAsset};
use stubby_net::{
    HostChannel, JoinedHost, NetPlayer,
    lightyear::prelude::{server::Started, *},
};
use stubby_streams::{
    Dismantle, LaidStream, LayRequest, Path, PortOwner, SegmentCost, StreamRecord, Templates,
    UpstreamRecord, dismantle_segment, lay_cost, lay_stream, plan_is_sound, port_is_free,
    segment_cost, upstream_record,
};

use super::inventory::{HostSays, Pack, pack_of};

/// How far from a player's body the host accepts a lay's port or a dismantled segment:
/// the stream tools' reach (a port within 3 m of the aim, F within 8 m), and room.
pub const STREAM_REACH: f32 = 12.0;

/// A client asks the host to lay a stream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lay {
    pub points: [[f32; 3]; 4],
    pub segments: u32,
    pub upstream: UpstreamRecord,
    pub joins: bool,
}

/// A client asks the host to dismantle the segment with this key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cut(pub u64);

pub struct StreamsPlugin;

impl Plugin for StreamsPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<Lay>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<Cut>()
            .add_direction(NetworkDirection::ClientToServer);
        app.component::<StreamRecord>().replicate();
        // Streams' own, so a session works without the build tools (tests, a server).
        app.add_message::<LayRequest>()
            .add_message::<Dismantle>()
            .init_resource::<Templates>()
            .init_resource::<SegmentCost>();
        app.add_systems(
            Update,
            (
                (serve_lays, serve_cuts, keep_records)
                    .chain()
                    .run_if(not(resource_exists::<JoinedHost>)),
                (
                    ask_to_lay,
                    ask_to_cut,
                    show_streams,
                    follow_records,
                    connect_burblers,
                )
                    .chain()
                    .run_if(resource_exists::<JoinedHost>),
            ),
        );
    }
}

/// Where `upstream` is here.
fn find_upstream(
    upstream: &UpstreamRecord,
    placed: &Query<(Entity, &PlacedAsset)>,
    keys: &Query<(Entity, &LaidStream)>,
) -> Option<PortOwner> {
    match upstream {
        UpstreamRecord::Burbler(instance) => placed
            .iter()
            .find(|(_, p)| p.instance == *instance)
            .map(|(e, _)| PortOwner::Burbler(e)),
        UpstreamRecord::Stream(key) => keys
            .iter()
            .find(|(_, s)| s.key() == *key)
            .map(|(e, _)| PortOwner::Stream(e)),
    }
}

/// The body of whoever is on `link`.
fn body_of(
    link: Entity,
    bodies: &Query<(&ControlledBy, &avian3d::prelude::Position), With<NetPlayer>>,
) -> Option<Vec3> {
    bodies
        .iter()
        .find(|(owner, _)| owner.owner == link)
        .map(|(_, position)| position.0)
}

// --- On the host --- //

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn serve_lays(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<Lay>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    bodies: Query<(&ControlledBy, &avian3d::prelude::Position), With<NetPlayer>>,
    placed: Query<(Entity, &PlacedAsset)>,
    (templates, mut meshes, cost): (
        Option<Res<Templates>>,
        ResMut<Assets<Mesh>>,
        Option<Res<SegmentCost>>,
    ),
    mut burblers: Query<&mut AssetState>,
    mut streams: Query<(Entity, &mut LaidStream)>,
    mut commands: Commands,
) {
    let (Some(templates), Some(cost)) = (templates, cost) else {
        return;
    };
    for (link, mut receiver, mut says) in &mut links {
        for lay in receiver.receive() {
            let Some(at) = body_of(link, &bodies) else {
                continue;
            };
            let path = Path::from_points(lay.points.map(Vec3::from_array));
            let keys: Vec<(Entity, u64)> = streams.iter().map(|(e, s)| (e, s.key())).collect();
            let upstream = match &lay.upstream {
                UpstreamRecord::Burbler(instance) => placed
                    .iter()
                    .find(|(_, p)| p.instance == *instance)
                    .map(|(e, _)| PortOwner::Burbler(e)),
                UpstreamRecord::Stream(key) => keys
                    .iter()
                    .find(|(_, k)| k == key)
                    .map(|(e, _)| PortOwner::Stream(*e)),
            };
            let Some(upstream) = upstream.filter(|u| port_is_free(*u, &burblers, &streams)) else {
                says.send::<HostChannel>(HostSays::Line("That port is taken.".into()));
                continue;
            };
            if path.points[0].distance(at) > STREAM_REACH || !plan_is_sound(&path, lay.segments) {
                says.send::<HostChannel>(HostSays::Line("That stream cannot be laid.".into()));
                continue;
            }
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            let price: Vec<stubby_recipes::Stack> = lay_cost(&cost, lay.segments)
                .into_iter()
                .map(|(item, quantity)| stubby_recipes::Stack { item, quantity })
                .collect();
            if let Err(error) = pack.items.take_stacks(&price) {
                says.send::<HostChannel>(HostSays::Line(error.to_string()));
                continue;
            }
            lay_stream(
                &mut commands,
                &templates,
                &mut meshes,
                &LayRequest {
                    path,
                    segments: lay.segments,
                    upstream,
                    joins: lay.joins,
                },
                &mut burblers,
                &mut streams,
            );
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn serve_cuts(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<Cut>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    bodies: Query<(&ControlledBy, &avian3d::prelude::Position), With<NetPlayer>>,
    (templates, mut meshes, cost): (
        Option<Res<Templates>>,
        ResMut<Assets<Mesh>>,
        Option<Res<SegmentCost>>,
    ),
    mut burblers: Query<&mut AssetState>,
    mut streams: Query<(Entity, &mut LaidStream)>,
    mut commands: Commands,
) {
    let (Some(templates), Some(cost)) = (templates, cost) else {
        return;
    };
    let mut gone = Vec::new();
    for (link, mut receiver, mut says) in &mut links {
        for Cut(key) in receiver.receive() {
            let Some(at) = body_of(link, &bodies) else {
                continue;
            };
            let Some((stream, k, point)) = streams.iter().find_map(|(e, s)| {
                (s.first..=s.last)
                    .find(|k| s.segment_key(*k) == key)
                    .map(|k| (e, k, s.path.at((k as f32 + 0.5) * s.span).0))
            }) else {
                continue;
            };
            if gone.contains(&stream) || point.distance(at) > STREAM_REACH {
                continue;
            }
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            let refund: Vec<stubby_recipes::Stack> = segment_cost(&cost)
                .into_iter()
                .map(|(item, quantity)| stubby_recipes::Stack { item, quantity })
                .collect();
            if pack.items.grant(&refund).is_err() {
                says.send::<HostChannel>(HostSays::Line(
                    "No room for what this segment gives back.".into(),
                ));
                continue;
            }
            dismantle_segment(
                &mut commands,
                &templates,
                &mut meshes,
                &mut burblers,
                &mut streams,
                stream,
                k,
            );
            gone.push(stream);
        }
    }
}

/// Every laid stream's record follows it, and on a running server it is everyone's.
fn keep_records(
    servers: Query<(), With<Started>>,
    streams: Query<(Entity, &LaidStream, Option<&StreamRecord>, Has<Replicate>)>,
    laid: Query<&LaidStream>,
    placed: Query<&PlacedAsset>,
    mut commands: Commands,
) {
    if servers.is_empty() {
        return;
    }
    for (entity, stream, record, replicated) in &streams {
        let upstream = stream
            .upstream
            .and_then(|owner| upstream_record(owner, &placed, &laid));
        let now = StreamRecord::of(stream, upstream);
        let mut entity = commands.entity(entity);
        if record != Some(&now) {
            entity.insert(now);
        }
        if !replicated {
            entity.insert(Replicate::to_clients(NetworkTarget::All));
        }
    }
}

// --- On a joined client --- //

fn ask_to_lay(
    mut requests: MessageReader<LayRequest>,
    mut senders: Query<&mut MessageSender<Lay>, (With<client::Client>, Without<LinkOf>)>,
    placed: Query<&PlacedAsset>,
    laid: Query<&LaidStream>,
) {
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        let Some(upstream) = upstream_record(request.upstream, &placed, &laid) else {
            continue;
        };
        sender.send::<HostChannel>(Lay {
            points: request.path.points.map(|p| p.to_array()),
            segments: request.segments,
            upstream,
            joins: request.joins,
        });
    }
}

fn ask_to_cut(
    mut requests: MessageReader<Dismantle>,
    mut senders: Query<&mut MessageSender<Cut>, (With<client::Client>, Without<LinkOf>)>,
    laid: Query<&LaidStream>,
) {
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        if let Ok(stream) = laid.get(request.stream) {
            sender.send::<HostChannel>(Cut(stream.segment_key(request.segment)));
        }
    }
}

/// A stream the host laid arrived: bend it here from its record, once what it is laid
/// from is here too.
#[allow(clippy::type_complexity)]
fn show_streams(
    arrived: Query<(Entity, &StreamRecord), (With<client::Remote>, Without<LaidStream>)>,
    placed: Query<(Entity, &PlacedAsset)>,
    keys: Query<(Entity, &LaidStream)>,
    templates: Option<Res<Templates>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let Some(templates) = templates else {
        return;
    };
    for (entity, record) in &arrived {
        let upstream = match &record.upstream {
            None => None,
            Some(name) => match find_upstream(name, &placed, &keys) {
                Some(found) => Some(found),
                None => continue,
            },
        };
        stubby_streams::rebuild_stream(
            &mut commands,
            &templates,
            &mut meshes,
            entity,
            record,
            upstream,
        );
    }
}

/// A shown stream follows its record (its end taken or freed, its upstream changed).
fn follow_records(
    mut streams: Query<(Entity, Ref<StreamRecord>, &mut LaidStream)>,
    placed: Query<(Entity, &PlacedAsset)>,
) {
    let keys: Vec<(Entity, u64)> = streams.iter().map(|(e, _, s)| (e, s.key())).collect();
    for (_, record, mut laid) in &mut streams {
        if !record.is_changed() {
            continue;
        }
        let upstream = match &record.upstream {
            None => None,
            Some(UpstreamRecord::Burbler(instance)) => placed
                .iter()
                .find(|(_, p)| p.instance == *instance)
                .map(|(e, _)| PortOwner::Burbler(e)),
            Some(UpstreamRecord::Stream(key)) => keys
                .iter()
                .find(|(_, k)| k == key)
                .map(|(e, _)| PortOwner::Stream(*e)),
        };
        if laid.end_used != record.end_used {
            laid.end_used = record.end_used;
        }
        if laid.upstream != upstream {
            laid.upstream = upstream;
        }
    }
}

/// A burbler is taken while a stream is laid from it, here as on the host.
fn connect_burblers(
    streams: Query<&LaidStream>,
    mut burblers: Query<(Entity, &PlacedAsset, &mut AssetState)>,
) {
    for (entity, placed, mut state) in &mut burblers {
        if placed.asset != "mudpit_burbler" {
            continue;
        }
        let connected = streams
            .iter()
            .any(|s| s.upstream == Some(PortOwner::Burbler(entity)));
        if state.stream_connected != connected {
            state.stream_connected = connected;
        }
    }
}
