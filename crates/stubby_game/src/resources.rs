//! World resource collection and the first inventory/recipe boundary.
use bevy::prelude::*;
use stubby_recipes::SignalUse;

// The crafting system is the `bevy_stubby_recipes` plugin; these names stay importable
// from here so the rest of the game is unchanged.
pub use super::recipes_plugin::{
    CraftCompleted, CraftRequest, HarvestRequest, MineNode, PlayerInventory, RecipeCatalog,
    RecipesPlugin, RecipesSystems, ResourceCollected, ResourceNode, SourceLedger, StorageMove,
    StorageRequest, Store,
};

/// The opening's placed lightning bug, as a source (`lightning_bug-<index>`).
pub fn lightning_bug_node(index: usize) -> ResourceNode {
    ResourceNode {
        resource: "lightning_bug".into(),
        instance: format!("lightning_bug-{index}"),
        collected: false,
    }
}

/// A scrub oak, as an infinite mine (`scrub_oak-<index>`).
pub fn scrub_oak_node(index: usize) -> MineNode {
    MineNode {
        mine: "scrub_oak".into(),
        instance: format!("scrub_oak-{index}"),
    }
}

#[derive(Component)]
struct InventoryHud;
#[derive(Component)]
struct CollectionPopup;
#[derive(Component)]
struct CollectionPopupText;
/// The prompt's muted second word ("hold for more", for pickups).
#[derive(Component)]
struct CollectionPopupHold;
#[derive(Resource, Default)]
struct RecipeBookView {
    open: bool,
}

/// Where the game asks to harvest (`HarvestRequest`): after it, before `RecipesSystems`,
/// a shared world's client takes its requests to the host.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectRequests;

pub struct ResourceRecipesPlugin;
impl Plugin for ResourceRecipesPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<RecipesPlugin>() {
            app.add_plugins(RecipesPlugin::default());
        }
        app.init_resource::<stubby_core::InteractTarget>()
            // What is in the hand gates collecting sounds (the hand plugin owns it).
            .init_resource::<stubby_player::hand::Hand>()
            .init_resource::<RecipeBookView>()
            .add_systems(Startup, setup)
            .configure_sets(Update, RecipesSystems.after(stubby_core::PrefabBinding))
            .add_systems(
                Update,
                (
                    pickups_interactable
                        .after(stubby_core::PrefabBinding)
                        .before(super::interaction::InteractionTargeting),
                    // A joined client's sources return when the host says so.
                    (
                        replenish_nodes.run_if(not(resource_exists::<stubby_net::JoinedHost>)),
                        collect_nearby,
                    )
                        .chain()
                        .in_set(CollectRequests)
                        .after(stubby_core::PrefabBinding)
                        .after(stubby_core::MachineInteractionSet)
                        .before(RecipesSystems),
                    (mark_collected, update_hud, update_collection_popup)
                        .chain()
                        .after(RecipesSystems),
                    (opening_bugs_are_sources, harvested_state).after(mark_collected),
                ),
            );
    }
}
/// Glue between crafting and the player: an uncollected, working source is something E
/// can pick up, so it is marked `Interactable` for the player's reach.
fn pickups_interactable(
    mut commands: Commands,
    nodes: Query<(
        Entity,
        &ResourceNode,
        Has<super::prefabs::PrefabError>,
        Has<stubby_core::Interactable>,
    )>,
) {
    for (entity, node, broken, marked) in &nodes {
        let available = !node.collected && !broken;
        if available && !marked {
            commands
                .entity(entity)
                .insert(stubby_core::Interactable::Pickup);
        } else if !available && marked {
            commands
                .entity(entity)
                .remove::<stubby_core::Interactable>();
        }
    }
}

/// Glue between crafting and life: the built-in opening's bugs are lightning bug sources.
fn opening_bugs_are_sources(
    mut commands: Commands,
    bugs: Query<(Entity, &stubby_life::bug::OpeningBug), Without<ResourceNode>>,
) {
    for (entity, bug) in &bugs {
        commands.entity(entity).insert(lightning_bug_node(bug.0));
    }
}

/// Glue between crafting and anything that shows a source: a collected source is
/// `Harvested` until it returns.
fn harvested_state(
    mut commands: Commands,
    nodes: Query<(Entity, &ResourceNode, Has<stubby_core::Harvested>), Changed<ResourceNode>>,
) {
    for (entity, node, marked) in &nodes {
        if node.collected && !marked {
            commands.entity(entity).insert(stubby_core::Harvested);
        } else if !node.collected && marked {
            commands.entity(entity).remove::<stubby_core::Harvested>();
        }
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let reticle = Image::from_buffer(
        include_bytes!("../../../assets/ui/sunflower_reticle.png"),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::linear(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("embedded sunflower reticle is a valid PNG");
    commands
        .spawn((
            Name::new("Sunflower reticle"),
            // Decorative and full-screen: it must never catch the pointer.
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Pickable::IGNORE,
                ImageNode::new(images.add(reticle)),
                Node {
                    width: px(28),
                    height: px(28),
                    ..default()
                },
            ));
        });
    commands
        .spawn((
            CollectionPopup,
            Visibility::Hidden,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                top: percent(55),
                left: px(0),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Satisfactory's prompt: the key as a keycap, then the action, over the world.
            parent
                .spawn((
                    Node {
                        column_gap: px(8),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    stubby_ui::style::keycap(
                        row,
                        &stubby_ui::style::key_name(stubby_core::INTERACT),
                        16.,
                    );
                    row.spawn((
                        CollectionPopupText,
                        stubby_ui::style::world_label("", 18., stubby_ui::style::INK),
                    ));
                    row.spawn((
                        CollectionPopupHold,
                        stubby_ui::style::world_label("", 13., stubby_ui::style::MUTED),
                    ));
                });
        });
    // The recipe book (O), until it has its own screen (2.7): a panel on the left.
    commands.spawn((
        InventoryHud,
        Visibility::Hidden,
        stubby_ui::style::label("", 15., stubby_ui::style::INK),
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            top: percent(28),
            max_width: px(360),
            padding: UiRect::all(px(14)),
            border: UiRect::left(px(3)),
            ..default()
        },
        BackgroundColor(stubby_ui::style::PANEL),
        BorderColor::all(stubby_ui::style::AMBER),
    ));
}
/// The game decides *when* to harvest (E held, nearest available node within reach) and
/// asks the plugin; `mark_collected` applies the result to the node.
#[allow(clippy::too_many_arguments)] // Collection spans input, world transforms and the request.
fn collect_nearby(
    keys: Res<ButtonInput<KeyCode>>,
    catalog: Option<Res<RecipeCatalog>>,
    target: Res<stubby_core::InteractTarget>,
    mut player: ResMut<PlayerInventory>,
    nodes: Query<&ResourceNode, Without<super::prefabs::PrefabError>>,
    mut requests: MessageWriter<HarvestRequest>,
    calendar: Option<Res<super::calendar::Calendar>>,
    screens: Option<Res<stubby_core::ScreenMode>>,
    hand: Res<stubby_player::hand::Hand>,
) {
    if screens.as_ref().is_some_and(|s| s.open.is_some())
        || target.machine().is_some()
        || !keys.pressed(stubby_core::INTERACT)
        || catalog.is_none()
    {
        return;
    }
    let Some(node) = target
        .pickup()
        .and_then(|e| nodes.get(e).ok())
        .filter(|n| !n.collected)
    else {
        player.feedback = "Move within 3 m of an available resource to collect.".into();
        return;
    };
    // Sounds are collected with the field recorder (sync-recorder-feedback); materials
    // such as reeds need nothing in hand. A refusal is silent and takes nothing.
    let sound = catalog.as_ref().is_some_and(|catalog| {
        catalog.0.resource(&node.resource).is_some_and(|resource| {
            resource.yields.iter().any(|stack| {
                catalog
                    .0
                    .item(&stack.item)
                    .is_some_and(stubby_recipes::ItemDefinition::is_sound)
            })
        })
    });
    if sound && hand.0 != Some(stubby_player::hand::HandItem::FieldRecorder) {
        player.feedback = "Hold up the field recorder (H) to collect sounds.".into();
        return;
    }
    requests.write(HarvestRequest {
        source: node.source_object(),
        now: calendar
            .as_ref()
            .map(|c| resource_time(c))
            .unwrap_or_default(),
    });
}
/// A harvested node is depleted until it replenishes.
fn mark_collected(
    mut collected: MessageReader<ResourceCollected>,
    mut nodes: Query<&mut ResourceNode>,
) {
    for event in collected.read() {
        for mut node in &mut nodes {
            if node.instance == event.instance {
                node.collected = true;
            }
        }
    }
}
/// The one interaction prompt, for whatever E would act on: a pickup or a machine.
fn update_collection_popup(
    target: Res<stubby_core::InteractTarget>,
    nodes: Query<&ResourceNode, Without<super::prefabs::PrefabError>>,
    catalog: Option<Res<RecipeCatalog>>,
    machine: Option<Res<super::machine_config::MachineInteraction>>,
    mut popups: Query<&mut Visibility, With<CollectionPopup>>,
    mut labels: Query<&mut Text, (With<CollectionPopupText>, Without<CollectionPopupHold>)>,
    mut holds: Query<&mut Text, With<CollectionPopupHold>>,
    screens: Option<Res<stubby_core::ScreenMode>>,
    fires: Query<&super::campfire::Fire>,
) {
    let mut hold = "";
    let action = if screens.as_ref().is_some_and(|s| s.open.is_some()) {
        None
    } else if let Some(node) = target
        .pickup()
        .and_then(|e| nodes.get(e).ok())
        .filter(|n| !n.collected)
    {
        let name = catalog
            .as_ref()
            .and_then(|catalog| catalog.0.resource(&node.resource))
            .map_or(node.resource.as_str(), |resource| resource.name.as_str());
        hold = "hold for more";
        Some(format!("Collect {}", name.to_lowercase()))
    } else {
        machine
            .as_ref()
            .filter(|m| m.target.is_some() && !m.prompt.is_empty())
            .map(|m| {
                // A lit campfire is configured, not lit (user, 2026-11-18: the prompt still
                // said to light it).
                let burning = m
                    .target
                    .and_then(|t| fires.get(t).ok())
                    .is_some_and(|f| *f == super::campfire::Fire::Burning);
                if burning {
                    "Configure the campfire".into()
                } else {
                    m.prompt.clone()
                }
            })
    };
    for mut visibility in &mut popups {
        visibility.set_if_neq(if action.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    if let Some(action) = action {
        for mut text in &mut labels {
            if text.0 != action {
                text.0.clone_from(&action);
            }
        }
        for mut text in &mut holds {
            if text.0 != hold {
                text.0 = hold.to_owned();
            }
        }
    }
}
/// The recipe book panel (O): what the player has discovered and holds.
fn update_hud(
    keys: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<RecipeBookView>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    mut hud: Query<(&mut Text, &mut Visibility), With<InventoryHud>>,
    screens: Option<Res<stubby_core::ScreenMode>>,
) {
    let Ok((mut hud, mut visibility)) = hud.single_mut() else {
        return;
    };
    if keys.just_pressed(stubby_core::RECIPE_BOOK) {
        view.open = !view.open;
    }
    let shown = view.open && !screens.as_ref().is_some_and(|s| s.open.is_some());
    visibility.set_if_neq(if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let mut text = String::new();
    if view.open {
        text.push_str("RECIPE BOOK\n\n");
        if player.book.items().next().is_none() {
            text.push_str("Mine a lightning bug to discover Square wave.\n");
        }
        if let Some(catalog) = catalog {
            for id in player.book.items() {
                if let Some(item) = catalog.0.item(id) {
                    let uses: Vec<_> = item
                        .uses
                        .iter()
                        .map(|role| match role {
                            SignalUse::Vco => "VCO",
                            SignalUse::Lfo => "LFO",
                            SignalUse::Gate => "Gate",
                        })
                        .collect();
                    text.push_str(&format!(
                        "{} | {} owned\nUses: {}\n",
                        item.name,
                        player.items.quantity(id),
                        uses.join(" | ")
                    ));
                }
            }
        }
    }
    if hud.0 != text {
        hud.0 = text;
    }
}

/// Glue between the world's calendar and crafting's replenishment clock.
pub fn resource_time(calendar: &stubby_world::calendar::Calendar) -> stubby_recipes::ResourceTime {
    stubby_recipes::ResourceTime {
        elapsed_millis: calendar.elapsed().as_millis().min(u64::MAX as u128) as u64,
        nights_started: calendar.nights_started(),
    }
}

fn replenish_nodes(
    calendar: Res<super::calendar::Calendar>,
    catalog: Option<Res<RecipeCatalog>>,
    mut ledger: ResMut<super::recipes_plugin::SourceLedger>,
    mut nodes: Query<&mut ResourceNode>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    for mut node in &mut nodes {
        if !node.collected {
            continue;
        }
        let mut source = node.source_object();
        if catalog
            .0
            .replenish_source(&mut source, &mut ledger.0, resource_time(&calendar))
        {
            node.collected = false;
        }
    }
}

#[cfg(test)]
mod tests {
    /// Every item shows its icon in the inventory and the rack (`stubby_ui::item_icon`
    /// loads `assets/ui/icons/<item>.png`; 2026-11-18: the snail shell had none, so its box
    /// showed only a count).
    #[test]
    fn every_item_has_an_icon() {
        let rules = stubby_recipes::rules::shipped_rules().expect("rules");
        let icons = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/ui/icons"
        ));
        for item in &rules.definition.items {
            assert!(
                icons.join(format!("{}.png", item.id)).is_file(),
                "{} has no icon: add it to tools/icons/make_item_icons.py",
                item.id
            );
        }
        // The build menu shows each buildable's icon too (2026-11-27: the storage barrel
        // had none).
        for buildable in &rules.definition.buildables {
            assert!(
                icons.join(format!("{}.png", buildable.id)).is_file(),
                "{} has no icon: add it to tools/icons/make_item_icons.py",
                buildable.id
            );
        }
    }

    use super::*;
    use stubby_core::InteractTarget;
    /// Collection as the game runs it: the plugin plus the game's systems around it.
    /// Reach is avian's (interaction.rs); here the target is set directly.
    fn collection_app() -> App {
        let mut app = App::new();
        app.add_plugins(RecipesPlugin::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InteractTarget>()
            .init_resource::<stubby_player::hand::Hand>()
            .add_systems(
                Update,
                (
                    collect_nearby.before(RecipesSystems),
                    mark_collected.after(RecipesSystems),
                ),
            );
        app
    }
    #[test]
    fn night_return_restores_existing_pickup_without_free_items() {
        let catalog = stubby_recipes::shipped_catalog().unwrap();
        let mut player = PlayerInventory::default();
        let mut ledger = super::super::recipes_plugin::SourceLedger::default();
        let mut node = lightning_bug_node(0);
        let mut source = node.source_object();
        catalog
            .harvest_source_at(
                &mut source,
                &mut player.items,
                &mut ledger.0,
                stubby_recipes::ResourceTime::default(),
            )
            .unwrap();
        node.collected = true;
        let mut time = Time::<Virtual>::default();
        time.set_max_delta(std::time::Duration::from_secs(2700));
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs(2700),
            ))
            .insert_resource(RecipeCatalog(catalog))
            .insert_resource(player)
            .insert_resource(ledger)
            .insert_resource(time)
            .init_resource::<super::super::calendar::Calendar>()
            .add_systems(Update, replenish_nodes);
        // Drive the real shared calendar without rendering or an audio device.
        app.add_systems(PreUpdate, super::super::calendar::advance);
        let entity = app.world_mut().spawn(node).id();
        app.update();
        app.update();
        assert!(!app.world().get::<ResourceNode>(entity).unwrap().collected);
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            2
        );
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            2
        );
    }
    #[test]
    fn mine_objects_are_authorable_but_not_finite_collection_nodes() {
        let mut app = collection_app();
        let tree = app.world_mut().spawn(scrub_oak_node(7)).id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            0
        );
        let mine = app.world().get::<MineNode>(tree).unwrap().source_object();
        assert_eq!(mine.kind(), stubby_recipes::SourceKind::InfiniteMine);
        assert_eq!(mine.instance_id(), "scrub_oak-7");
    }

    #[test]
    fn sounds_need_the_recorder_in_hand_and_reeds_do_not() {
        let mut app = collection_app();
        app.world_mut()
            .resource_mut::<stubby_player::hand::Hand>()
            .0 = None;
        let bug = app.world_mut().spawn(lightning_bug_node(0)).id();
        let reeds = app
            .world_mut()
            .spawn(ResourceNode {
                resource: "reed_clump".into(),
                instance: "reed_clump-0".into(),
                collected: false,
            })
            .id();
        let held = |app: &App, item: &str| {
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity(item)
        };
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        *app.world_mut().resource_mut::<InteractTarget>() = InteractTarget::Pickup(bug);
        app.update();
        assert_eq!(held(&app, "square_wave"), 0, "no recorder: nothing taken");
        assert!(
            app.world()
                .resource::<PlayerInventory>()
                .feedback
                .contains("(H)"),
            "the refusal names the recorder's key"
        );
        assert!(!app.world().get::<ResourceNode>(bug).unwrap().collected);
        *app.world_mut().resource_mut::<InteractTarget>() = InteractTarget::Pickup(reeds);
        app.update();
        assert!(held(&app, "reed") > 0, "reeds need nothing in hand");
        app.world_mut()
            .resource_mut::<stubby_player::hand::Hand>()
            .0 = Some(stubby_player::hand::HandItem::FieldRecorder);
        *app.world_mut().resource_mut::<InteractTarget>() = InteractTarget::Pickup(bug);
        app.update();
        assert_eq!(
            held(&app, "square_wave"),
            2,
            "with the recorder, the bug is caught"
        );
    }

    #[test]
    fn e_collects_the_targeted_pickup_once_and_f_does_not() {
        let mut app = collection_app();
        let bug = app.world_mut().spawn(lightning_bug_node(0)).id();
        *app.world_mut().resource_mut::<InteractTarget>() = InteractTarget::Pickup(bug);
        let squares = |app: &App| {
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave")
        };
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert_eq!(squares(&app), 0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(squares(&app), 2);
        assert!(
            app.world()
                .resource::<PlayerInventory>()
                .book
                .knows("square_wave")
        );
        app.update(); // E still held: the collected bug yields nothing more
        assert_eq!(squares(&app), 2);
        assert!(app.world().get::<ResourceNode>(bug).unwrap().collected);
    }

    #[test]
    fn popup_shows_for_a_targeted_pickup_and_hides_after_harvest() {
        let mut app = App::new();
        app.init_resource::<InteractTarget>()
            .add_systems(Update, update_collection_popup);
        let bug = app.world_mut().spawn(lightning_bug_node(0)).id();
        let popup = app
            .world_mut()
            .spawn((CollectionPopup, Visibility::Hidden))
            .id();
        *app.world_mut().resource_mut::<InteractTarget>() = InteractTarget::Pickup(bug);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(popup).unwrap(),
            Visibility::Visible
        );
        app.world_mut()
            .get_mut::<ResourceNode>(bug)
            .unwrap()
            .collected = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(popup).unwrap(),
            Visibility::Hidden
        );
    }
}
