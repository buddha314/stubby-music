//! Inventories in a shared world (task 3.2, design D4 and open question 1): each player
//! carries their own, and the settlement's storage is shared. The host keeps both. A
//! client's inventory is its `Pack` on the host, replicated to that client alone and
//! mirrored into its `PlayerInventory`; storage replicates to everyone. Crafting and
//! storage moves are requests the host checks by the same rules a single player uses.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_net::{
    HostChannel, JoinedHost, NetPlayer,
    lightyear::prelude::{server::Started, *},
};
use stubby_recipes::{Inventory, RecipeBook, Stack};

use crate::recipes_plugin::{
    CraftCompleted, CraftRequest, DecidedElsewhere, InventoryFull, PlayerInventory, RecipeCatalog,
    SharedStorage, StorageRequest,
};

/// A remote player's inventory and recipe book, kept by the host.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    pub owner: PeerId,
    pub items: Inventory,
    pub book: RecipeBook,
}

impl Pack {
    /// A new player's: the shipped starter inventory, known from the start.
    pub fn starter(owner: PeerId) -> Self {
        let mut pack = Self {
            owner,
            items: Inventory::default(),
            book: RecipeBook::default(),
        };
        if let Ok(rules) = stubby_recipes::rules::shipped_rules() {
            if let Err(error) = pack.items.grant(&rules.start) {
                error!("Starter inventory: {error}");
            }
            for stack in &rules.start {
                pack.book.discover(&stack.item);
            }
        }
        pack
    }
}

/// The settlement's storage, replicated to everyone.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Storage(pub Inventory);

/// A client asks to move items between its pack and the storage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stash(pub StorageRequest);

/// A client asks to craft (at a bench whose clock runs at `bpm`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Craft {
    pub recipe: String,
    pub bpm: Option<f32>,
}

/// The host crafted for this client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crafted {
    pub recipe: String,
    pub outputs: Vec<Stack>,
}

/// What the host has to say to one client about its request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HostSays {
    /// A line for the player.
    Line(String),
    /// No room in the player's inventory for `item`.
    Full(String),
}

pub struct InventoriesPlugin;

impl Plugin for InventoriesPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<Stash>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<Craft>()
            .add_direction(NetworkDirection::ClientToServer);
        app.register_message::<Crafted>()
            .add_direction(NetworkDirection::ServerToClient);
        app.register_message::<HostSays>()
            .add_direction(NetworkDirection::ServerToClient);
        app.component::<Pack>().replicate();
        app.component::<Storage>().replicate();
        app.add_observer(give_packs)
            .add_observer(publish_storage)
            .add_systems(
                Update,
                (
                    (serve_stashes, serve_crafts, share_storage).chain(),
                    (
                        defer_to_host.run_if(resource_added::<JoinedHost>),
                        (
                            ask_to_stash,
                            ask_to_craft,
                            hear_the_host,
                            follow_pack,
                            follow_storage,
                        ),
                    )
                        .chain()
                        .run_if(resource_exists::<JoinedHost>),
                ),
            );
    }
}

// --- On the host --- //

/// Every remote player gets a pack, once: one who leaves and returns (the same Steam
/// id) finds theirs as they left it. The host's own player keeps `PlayerInventory`.
fn give_packs(
    added: On<Add<NetPlayer>>,
    servers: Query<(), With<Started>>,
    players: Query<&NetPlayer>,
    packs: Query<&Pack>,
    mut commands: Commands,
) {
    let Ok(NetPlayer(owner)) = players.get(added.entity) else {
        return;
    };
    if servers.is_empty() || owner.is_local() || packs.iter().any(|p| p.owner == *owner) {
        return;
    }
    commands.spawn((
        Name::new(format!("Pack of {owner:?}")),
        Pack::starter(*owner),
        Replicate::to_clients(NetworkTarget::Single(*owner)),
    ));
}

fn publish_storage(_: On<Add<Started>>, storage: Res<SharedStorage>, mut commands: Commands) {
    commands.spawn((
        Name::new("Shared storage"),
        Storage(storage.0.clone()),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

/// The pack of whoever is on `link`.
pub fn pack_of<'a>(
    link: Entity,
    remotes: &Query<&RemoteId>,
    packs: &'a mut Query<&mut Pack>,
) -> Option<Mut<'a, Pack>> {
    let peer = remotes.get(link).ok()?.0;
    packs.iter_mut().find(|pack| pack.owner == peer)
}

fn serve_stashes(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<Stash>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    mut storage: ResMut<SharedStorage>,
) {
    for (link, mut receiver, mut says) in &mut links {
        for Stash(request) in receiver.receive() {
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            let (Ok(line) | Err(line)) = request.apply(&mut pack.items, &mut storage.0);
            says.send::<HostChannel>(HostSays::Line(line));
        }
    }
}

#[allow(clippy::type_complexity)]
fn serve_crafts(
    mut links: Query<(
        Entity,
        &mut MessageReceiver<Craft>,
        &mut MessageSender<Crafted>,
        &mut MessageSender<HostSays>,
    )>,
    remotes: Query<&RemoteId>,
    mut packs: Query<&mut Pack>,
    catalog: Option<Res<RecipeCatalog>>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    for (link, mut receiver, mut crafted, mut says) in &mut links {
        for Craft { recipe, bpm } in receiver.receive() {
            let Some(mut pack) = pack_of(link, &remotes, &mut packs) else {
                continue;
            };
            let Pack { items, book, .. } = &mut *pack;
            // The tempo is the client's to report until benches are shared (3.4).
            match crate::recipes_plugin::craft(&catalog.0, &recipe, bpm, items, book) {
                Ok((done, line)) => {
                    crafted.send::<HostChannel>(Crafted {
                        recipe: done.recipe,
                        outputs: done.outputs,
                    });
                    says.send::<HostChannel>(HostSays::Line(line));
                }
                Err(line) => says.send::<HostChannel>(HostSays::Line(line)),
            }
        }
    }
}

/// The storage changed: everyone sees it.
fn share_storage(storage: Res<SharedStorage>, mut shared: Query<&mut Storage>) {
    if !storage.is_changed() {
        return;
    }
    for mut shared in &mut shared {
        if shared.0 != storage.0 {
            shared.0.clone_from(&storage.0);
        }
    }
}

// --- On a joined client --- //

/// From now on the host decides harvests, crafts and storage moves.
fn defer_to_host(mut commands: Commands) {
    commands.insert_resource(DecidedElsewhere);
}

fn ask_to_stash(
    mut requests: MessageReader<StorageRequest>,
    mut senders: Query<&mut MessageSender<Stash>, (With<client::Client>, Without<LinkOf>)>,
) {
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        sender.send::<HostChannel>(Stash(request.clone()));
    }
}

fn ask_to_craft(
    mut requests: MessageReader<CraftRequest>,
    mut senders: Query<&mut MessageSender<Craft>, (With<client::Client>, Without<LinkOf>)>,
) {
    let Ok(mut sender) = senders.single_mut() else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        sender.send::<HostChannel>(Craft {
            recipe: request.recipe.clone(),
            bpm: request.bpm,
        });
    }
}

/// The host's answers, as a local craft or refusal would report them.
fn hear_the_host(
    mut crafts: Query<&mut MessageReceiver<Crafted>, (With<client::Client>, Without<LinkOf>)>,
    mut says: Query<&mut MessageReceiver<HostSays>, (With<client::Client>, Without<LinkOf>)>,
    mut player: ResMut<PlayerInventory>,
    mut completed: MessageWriter<CraftCompleted>,
    mut full: MessageWriter<InventoryFull>,
) {
    for mut receiver in &mut crafts {
        for Crafted { recipe, outputs } in receiver.receive() {
            completed.write(CraftCompleted { recipe, outputs });
        }
    }
    for mut receiver in &mut says {
        for said in receiver.receive() {
            match said {
                HostSays::Line(line) => player.feedback = line,
                HostSays::Full(item) => {
                    player.feedback.clear();
                    full.write(InventoryFull { item });
                }
            }
        }
    }
}

/// This player's pack is its inventory.
fn follow_pack(packs: Query<Ref<Pack>, With<client::Remote>>, mut player: ResMut<PlayerInventory>) {
    for pack in &packs {
        if pack.is_changed() {
            player.items.clone_from(&pack.items);
            player.book.clone_from(&pack.book);
        }
    }
}

fn follow_storage(
    shared: Query<Ref<Storage>, With<client::Remote>>,
    mut storage: ResMut<SharedStorage>,
) {
    for shared in &shared {
        if shared.is_changed() {
            storage.0.clone_from(&shared.0);
        }
    }
}
