//! Rack configurations in a shared world (task 3.5, design D7 and open question 3): anyone
//! may configure any machine. A joined client edits a rack on its own screen and sends the
//! result; the host checks the player is at the machine and charges or refunds the
//! difference (a module's item, patch cables) from that player's pack, then applies it, or
//! sends the rack as it is back. Every rack's configuration, by machine instance, is one
//! replicated table, so late joiners hear what everyone hears; each peer's engine plays
//! its own copy (audio never crosses the network).
use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_core::{PlacedAsset, Screen, ScreenMode};
use stubby_net::{
    HostChannel, JoinedHost, NetPlayer,
    lightyear::prelude::{server::Started, *},
};

use super::inventory::{HostSays, Pack, pack_of};
use crate::machine_config::Profiles;
use crate::rack::{RackKnobs, RackState};
use bevy_stubby_rack::record::{RackRecord, price};
use bevy_stubby_rack::samples::RackSamples;

/// How far from a machine the host lets a player configure it: E's 3 m, plus room.
pub const RACK_REACH: f32 = 6.0;
/// A knob being dragged is sent at most this often.
const SEND_EVERY_S: f32 = 0.1;

/// Every configured rack, by machine instance, replicated to everyone.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Racks(pub BTreeMap<String, RackRecord>);

/// A client's rack as it configured it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetRack {
    pub machine: String,
    pub rack: RackRecord,
}

/// The host refused a client's rack: this is the rack as it is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RackIs {
    pub machine: String,
    pub rack: RackRecord,
}

pub struct RacksPlugin;

impl Plugin for RacksPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<SetRack>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<RackIs>()
            .add_direction(NetworkDirection::ServerToClient);
        app.component::<Racks>().replicate();
        app.init_resource::<Sent>();
        app.add_observer(publish_racks).add_systems(
            Update,
            (
                (serve_racks, keep_table)
                    .chain()
                    .run_if(not(resource_exists::<JoinedHost>)),
                (correct_racks, send_racks, follow_table)
                    .chain()
                    .run_if(resource_exists::<JoinedHost>),
            ),
        );
    }
}

type Rack = (
    &'static PlacedAsset,
    &'static GlobalTransform,
    &'static mut RackState,
    &'static mut RackKnobs,
    Option<&'static mut RackSamples>,
);

fn record_of(state: &RackState, knobs: &RackKnobs, samples: Option<&RackSamples>) -> RackRecord {
    RackRecord::of(state, knobs, samples)
}

// --- On the host --- //

fn publish_racks(_: On<Add<Started>>, mut commands: Commands) {
    commands.spawn((
        Name::new("Racks"),
        Racks::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

#[allow(clippy::type_complexity)]
fn serve_racks(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<SetRack>,
        &mut MessageSender<RackIs>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    bodies: Query<(&ControlledBy, &avian3d::prelude::Position), With<NetPlayer>>,
    mut racks: Query<Rack>,
    profiles: Option<Res<Profiles>>,
) {
    let Some(profiles) = profiles else {
        return;
    };
    for (link, mut receiver, mut answer, mut says) in &mut links {
        for SetRack { machine, rack } in receiver.receive() {
            let Some((placed, place, mut state, mut knobs, mut samples)) = racks
                .iter_mut()
                .find(|(placed, ..)| placed.instance == machine)
            else {
                continue;
            };
            let Some(config) = profiles.0.get(&placed.asset) else {
                continue;
            };
            let current = record_of(&state, &knobs, samples.as_deref());
            let mut refuse = |line: String| {
                says.send::<HostChannel>(HostSays::Line(line));
                answer.send::<HostChannel>(RackIs {
                    machine: machine.clone(),
                    rack: current.clone(),
                });
            };
            let near = bodies
                .iter()
                .find(|(owner, _)| owner.owner == link)
                .is_some_and(|(_, at)| place.translation().distance(at.0) <= RACK_REACH);
            if !near {
                refuse("Too far from the machine.".into());
                continue;
            }
            let (take, give) = match price(config, &current, &rack) {
                Ok(price) => price,
                Err(line) => {
                    refuse(line);
                    continue;
                }
            };
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            let stacks = |list: &[(String, u32)]| -> Vec<stubby_recipes::Stack> {
                list.iter()
                    .map(|(item, quantity)| stubby_recipes::Stack {
                        item: item.clone(),
                        quantity: *quantity,
                    })
                    .collect()
            };
            let mut items = pack.items.clone();
            if let Err(error) = items
                .take_stacks(&stacks(&take))
                .and_then(|()| items.grant(&stacks(&give)))
            {
                refuse(error.to_string());
                continue;
            }
            if pack.items != items {
                pack.items = items;
            }
            rack.apply(&mut state, &mut knobs, samples.as_mut());
        }
    }
}

/// The table follows every rack, whoever changed it.
fn keep_table(
    racks: Query<(&PlacedAsset, &RackState, &RackKnobs, Option<&RackSamples>)>,
    mut tables: Query<&mut Racks>,
) {
    let Ok(mut table) = tables.single_mut() else {
        return;
    };
    let now: BTreeMap<String, RackRecord> = racks
        .iter()
        .map(|(placed, state, knobs, samples)| {
            (placed.instance.clone(), record_of(state, knobs, samples))
        })
        .filter(|(_, record)| *record != RackRecord::default())
        .collect();
    if table.0 != now {
        table.0 = now;
    }
}

// --- On a joined client --- //

/// What this client last agreed with the host on, per machine: its edits are what differs.
#[derive(Default)]
struct Agreed(HashMap<String, RackRecord>);

/// The machine whose rack is open here.
fn open_rack(screens: &ScreenMode, placed: &Query<&PlacedAsset>) -> Option<String> {
    match screens.open {
        Some(Screen::MachineConfiguration(machine) | Screen::Crafting(machine)) => {
            placed.get(machine).ok().map(|p| p.instance.clone())
        }
        _ => None,
    }
}

/// The host's table, applied to every rack not open here (one being edited is the
/// player's until they close it), and to each rack as it appears (the scene can load
/// after the table arrives, and a campfire be built later).
#[allow(clippy::too_many_arguments)]
fn follow_table(
    tables: Query<Ref<Racks>, With<client::Remote>>,
    screens: Res<ScreenMode>,
    names: Query<&PlacedAsset>,
    mut racks: Query<Rack>,
    mut agreed: Local<Agreed>,
    mut was_open: Local<Option<String>>,
    sent: Res<Sent>,
) {
    let Ok(table) = tables.single() else {
        return;
    };
    let open = open_rack(&screens, &names);
    let closed = *was_open != open;
    *was_open = open.clone();
    let everything = table.is_changed() || closed;
    for (placed, _, mut state, mut knobs, mut samples) in &mut racks {
        if !everything && !state.is_added() {
            continue;
        }
        if open.as_ref() == Some(&placed.instance) {
            continue;
        }
        let want = table.0.get(&placed.instance).cloned().unwrap_or_default();
        // Not yet in the table: an edit of ours on its way.
        if sent
            .0
            .get(&placed.instance)
            .is_some_and(|ours| *ours != want)
            && !table.is_changed()
        {
            continue;
        }
        agreed.0.insert(placed.instance.clone(), want.clone());
        if record_of(&state, &knobs, samples.as_deref()) != want {
            want.apply(&mut state, &mut knobs, samples.as_mut());
        }
    }
}

/// The host refused an edit: the rack goes back to how it is, open or not.
fn correct_racks(
    mut receivers: Query<&mut MessageReceiver<RackIs>, (With<client::Client>, Without<LinkOf>)>,
    mut racks: Query<Rack>,
    mut sent: ResMut<Sent>,
) {
    for mut receiver in &mut receivers {
        for RackIs { machine, rack } in receiver.receive() {
            if let Some((_, _, mut state, mut knobs, mut samples)) = racks
                .iter_mut()
                .find(|(placed, ..)| placed.instance == machine)
            {
                rack.apply(&mut state, &mut knobs, samples.as_mut());
            }
            sent.0.insert(machine, rack);
        }
    }
}

/// What this client last sent, per machine.
#[derive(Resource, Default)]
pub struct Sent(pub HashMap<String, RackRecord>);

/// The rack open here, sent when the player changes it (a dragged knob at most every
/// tenth of a second), and its final setting when it closes.
#[allow(clippy::too_many_arguments)]
fn send_racks(
    screens: Res<ScreenMode>,
    names: Query<&PlacedAsset>,
    racks: Query<(&PlacedAsset, &RackState, &RackKnobs, Option<&RackSamples>)>,
    mut senders: Query<&mut MessageSender<SetRack>, (With<client::Client>, Without<LinkOf>)>,
    tables: Query<&Racks, With<client::Remote>>,
    mut sent: ResMut<Sent>,
    time: Res<Time<Real>>,
    mut last: Local<f32>,
    mut was_open: Local<Option<String>>,
) {
    let (Ok(mut sender), Ok(table)) = (senders.single_mut(), tables.single()) else {
        return;
    };
    let now = time.elapsed_secs();
    let open = open_rack(&screens, &names);
    let closed = was_open.take().filter(|was| open.as_ref() != Some(was));
    was_open.clone_from(&open);
    for (placed, state, knobs, samples) in &racks {
        let rack = record_of(state, knobs, samples);
        let known = sent
            .0
            .get(&placed.instance)
            .or_else(|| table.0.get(&placed.instance))
            .cloned()
            .unwrap_or_default();
        if rack == known {
            continue;
        }
        // Only the rack being edited here is the player's to send; the rest follow the
        // host's table.
        let closing = closed.as_ref() == Some(&placed.instance);
        if open.as_ref() != Some(&placed.instance) && !closing {
            continue;
        }
        if !closing && now - *last < SEND_EVERY_S {
            continue;
        }
        *last = now;
        sender.send::<HostChannel>(SetRack {
            machine: placed.instance.clone(),
            rack: rack.clone(),
        });
        sent.0.insert(placed.instance.clone(), rack);
    }
}
