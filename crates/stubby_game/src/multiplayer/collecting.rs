//! Collecting in a shared world (task 3.1, design D4): the host decides. A joined
//! client's harvest becomes a request to the host, which checks the player can reach an
//! available source, depletes it in the world's ledger, and sends the yields back. Which
//! sources are depleted replicates to everyone, late joiners included; sources return on
//! the host's clock.
use std::collections::HashMap;

use avian3d::prelude::Position;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_net::{
    HostChannel, JoinedHost, NetPlayer,
    lightyear::prelude::{server::Started, *},
};
use stubby_recipes::{HarvestLedger, Stack};

use crate::resources::{
    CollectRequests, GrantHarvest, HarvestRequest, PlayerInventory, RecipeCatalog, RecipesSystems,
    ResourceNode, SourceLedger, resource_time,
};

/// How far from a player's body the host accepts a harvest: the game's 3 m reach, plus
/// room for the body's height and a moment's lag.
pub const HOST_REACH: f32 = 4.5;

/// A client asks the host to harvest the source `instance` (its stable name, the same on
/// every peer).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collect {
    pub instance: String,
}

/// The host harvested `instance` for this client: these are its yields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collected {
    pub instance: String,
    pub yields: Vec<Stack>,
}

/// The world's depleted sources, from the host's ledger, replicated to everyone.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DepletedSources(pub Vec<String>);

pub struct CollectingPlugin;

impl Plugin for CollectingPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<Collect>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<Collected>()
            .add_direction(NetworkDirection::ServerToClient);
        app.component::<DepletedSources>().replicate();
        app.add_observer(publish_depletion).add_systems(
            Update,
            (
                ask_the_host
                    .run_if(resource_exists::<JoinedHost>)
                    .after(CollectRequests)
                    .before(RecipesSystems),
                (serve_collects, share_depletion).chain(),
                receive_yields
                    .run_if(resource_exists::<JoinedHost>)
                    .before(RecipesSystems),
                follow_depletion.run_if(resource_exists::<JoinedHost>),
            ),
        );
    }
}

// --- On a joined client --- //

/// Take this frame's harvests to the host instead of harvesting locally. A request in
/// flight is not repeated (E is held to collect), and one whose yields would not fit is
/// refused here, as a local harvest would be.
fn ask_the_host(
    mut requests: ResMut<Messages<HarvestRequest>>,
    catalog: Option<Res<RecipeCatalog>>,
    player: Res<PlayerInventory>,
    mut full: MessageWriter<crate::recipes_plugin::InventoryFull>,
    mut senders: Query<&mut MessageSender<Collect>, (With<client::Client>, Without<LinkOf>)>,
    time: Res<Time<Real>>,
    mut pending: Local<HashMap<String, f32>>,
) {
    let now = time.elapsed_secs();
    pending.retain(|_, asked| now - *asked < 1.0);
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.drain() {
        let instance = request.source.instance_id().to_string();
        if pending.contains_key(&instance) {
            continue;
        }
        let yields = catalog
            .as_ref()
            .and_then(|c| c.0.resource(request.source.definition_id()))
            .map(|r| r.yields.clone())
            .unwrap_or_default();
        if let Err(stubby_recipes::RecipeError::InventoryFull(item)) =
            player.items.clone().grant(&yields)
        {
            full.write(crate::recipes_plugin::InventoryFull { item });
            continue;
        }
        sender.send::<HostChannel>(Collect {
            instance: instance.clone(),
        });
        pending.insert(instance, now);
    }
}

/// The host's yields arrive: into the inventory, as a harvest of this player's own.
fn receive_yields(
    mut receivers: Query<&mut MessageReceiver<Collected>, (With<client::Client>, Without<LinkOf>)>,
    mut grants: MessageWriter<GrantHarvest>,
) {
    for mut receiver in &mut receivers {
        for collected in receiver.receive() {
            grants.write(GrantHarvest {
                instance: collected.instance,
                yields: collected.yields,
            });
        }
    }
}

/// The host's depleted sources are this world's: mirror them in the ledger (what spawns
/// later reads) and on every source.
fn follow_depletion(
    depleted: Query<Ref<DepletedSources>, With<client::Remote>>,
    mut nodes: Query<&mut ResourceNode>,
    mut ledger: ResMut<SourceLedger>,
) {
    let Ok(depleted) = depleted.single() else {
        return;
    };
    let all = depleted.is_changed();
    if all {
        ledger.0 = HarvestLedger::mirror(depleted.0.iter().cloned());
    }
    for mut node in &mut nodes {
        // Every source when the host's set changes; else only the ones just spawned.
        if !all && !node.is_added() {
            continue;
        }
        let collected = ledger.0.collected(&node.instance);
        if node.collected != collected {
            node.collected = collected;
        }
    }
}

// --- On the host --- //

fn publish_depletion(_started: On<Add<Started>>, mut commands: Commands) {
    commands.spawn((
        Name::new("Depleted sources"),
        DepletedSources::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

/// A client's harvest: allowed if its body is within reach of an available source. The
/// source is depleted for everyone and the yields go to that client alone.
#[allow(clippy::type_complexity)]
fn serve_collects(
    mut links: Query<
        (
            Entity,
            &mut MessageReceiver<Collect>,
            &mut MessageSender<Collected>,
        ),
        With<server::ClientOf>,
    >,
    bodies: Query<(&ControlledBy, &Position), With<NetPlayer>>,
    mut nodes: Query<(&mut ResourceNode, &GlobalTransform)>,
    catalog: Option<Res<RecipeCatalog>>,
    mut ledger: ResMut<SourceLedger>,
    calendar: Option<Res<crate::calendar::Calendar>>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    let now = calendar.map(|c| resource_time(&c)).unwrap_or_default();
    for (link, mut receiver, mut sender) in &mut links {
        let requests: Vec<Collect> = receiver.receive().collect();
        if requests.is_empty() {
            continue;
        }
        let Some(at) = bodies
            .iter()
            .find(|(owner, _)| owner.owner == link)
            .map(|(_, position)| position.0)
        else {
            continue;
        };
        for Collect { instance } in requests {
            let Some((mut node, place)) =
                nodes.iter_mut().find(|(node, _)| node.instance == instance)
            else {
                warn!("A client asked for {instance}, which this world does not have");
                continue;
            };
            if node.collected || place.translation().distance(at) > HOST_REACH {
                continue;
            }
            // The world's ledger decides; the yields are the client's (its inventory
            // checked the room before asking).
            let mut source = node.source_object();
            let mut scratch = stubby_recipes::Inventory::default();
            match catalog
                .0
                .harvest_source_at(&mut source, &mut scratch, &mut ledger.0, now)
            {
                Ok(yields) => {
                    node.collected = true;
                    sender.send::<HostChannel>(Collected { instance, yields });
                }
                Err(error) => debug!("Refused {instance}: {error}"),
            }
        }
    }
}

/// The ledger changed: tell everyone which sources are depleted.
fn share_depletion(ledger: Res<SourceLedger>, mut shared: Query<&mut DepletedSources>) {
    if !ledger.is_changed() {
        return;
    }
    let depleted: Vec<String> = ledger.0.depleted().map(str::to_owned).collect();
    for mut shared in &mut shared {
        if shared.0 != depleted {
            shared.0.clone_from(&depleted);
        }
    }
}
