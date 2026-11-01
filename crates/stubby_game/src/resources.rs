//! World resource collection and the first inventory/recipe boundary.
use bevy::prelude::*;
use stubby_recipes::{SignalUse, SourceObject};

// The crafting system is the `bevy_stubby_recipes` plugin; these names stay importable
// from here so the rest of the game is unchanged.
pub use super::recipes_plugin::{
    CraftCompleted, CraftRequest, HarvestRequest, PlayerInventory, RecipeCatalog, RecipesPlugin,
    RecipesSystems, ResourceCollected,
};

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct ResourceNode {
    pub resource: String,
    pub instance: String,
    pub collected: bool,
}
impl Default for ResourceNode {
    fn default() -> Self {
        Self {
            resource: "lightning_bug".into(),
            instance: "lightning_bug-0".into(),
            collected: false,
        }
    }
}
impl ResourceNode {
    pub fn source_object(&self) -> SourceObject {
        SourceObject::Exhaustible {
            resource: self.resource.clone(),
            instance: self.instance.clone(),
            collected: self.collected,
        }
    }
    pub fn lightning_bug(index: usize) -> Self {
        Self {
            instance: format!("lightning_bug-{index}"),
            ..default()
        }
    }
}

/// An infinite source authoring object. Not queried as a finite E pickup.
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct MineNode {
    pub mine: String,
    pub instance: String,
}
impl Default for MineNode {
    fn default() -> Self {
        Self::scrub_oak(0)
    }
}
impl MineNode {
    pub fn scrub_oak(index: usize) -> Self {
        Self {
            mine: "scrub_oak".into(),
            instance: format!("scrub_oak-{index}"),
        }
    }
    pub fn source_object(&self) -> SourceObject {
        SourceObject::InfiniteMine {
            mine: self.mine.clone(),
            instance: self.instance.clone(),
        }
    }
}

#[derive(Component)]
struct InventoryHud;
#[derive(Component)]
struct CollectionPopup;
#[derive(Component)]
struct CollectionPopupText;
#[derive(Resource, Default)]
struct RecipeBookView {
    open: bool,
}

pub struct ResourceRecipesPlugin;
impl Plugin for ResourceRecipesPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<RecipesPlugin>() {
            app.add_plugins(RecipesPlugin::default());
        }
        app.init_resource::<super::interaction::InteractTarget>()
            .init_resource::<RecipeBookView>()
            .register_type::<ResourceNode>()
            .register_type::<MineNode>()
            .add_systems(Startup, setup)
            .configure_sets(Update, RecipesSystems.after(super::prefabs::PrefabBinding))
            .add_systems(
                Update,
                (
                    (replenish_nodes, collect_nearby)
                        .chain()
                        .after(super::prefabs::PrefabBinding)
                        .after(super::machine_config::MachineInteractionSet)
                        .before(RecipesSystems),
                    (mark_collected, update_hud, update_collection_popup)
                        .chain()
                        .after(RecipesSystems),
                ),
            );
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
            parent.spawn((
                CollectionPopupText,
                Text::new("[E] Collect lightning bug"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(20.),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.94, 0.87)),
                BackgroundColor(Color::srgba(0.04, 0.035, 0.025, 0.8)),
                Node {
                    padding: UiRect::axes(px(12), px(6)),
                    ..default()
                },
            ));
        });
    commands.spawn((
        InventoryHud,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(18.),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.94, 0.90)),
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            bottom: px(20),
            ..default()
        },
    ));
}
/// The game decides *when* to harvest (E held, nearest available node within reach) and
/// asks the plugin; `mark_collected` applies the result to the node.
#[allow(clippy::too_many_arguments)] // Collection spans input, world transforms and the request.
fn collect_nearby(
    keys: Res<ButtonInput<KeyCode>>,
    catalog: Option<Res<RecipeCatalog>>,
    target: Res<super::interaction::InteractTarget>,
    mut player: ResMut<PlayerInventory>,
    nodes: Query<&ResourceNode, Without<super::prefabs::PrefabError>>,
    mut requests: MessageWriter<HarvestRequest>,
    calendar: Option<Res<super::calendar::Calendar>>,
    screens: Option<Res<super::controls::ScreenMode>>,
) {
    if screens.as_ref().is_some_and(|s| s.open.is_some())
        || target.machine().is_some()
        || !keys.pressed(super::controls::INTERACT)
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
    requests.write(HarvestRequest {
        source: node.source_object(),
        now: calendar
            .as_ref()
            .map(|c| c.resource_time())
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
    target: Res<super::interaction::InteractTarget>,
    nodes: Query<&ResourceNode, Without<super::prefabs::PrefabError>>,
    catalog: Option<Res<RecipeCatalog>>,
    machine: Option<Res<super::machine_config::MachineInteraction>>,
    mut popups: Query<&mut Visibility, With<CollectionPopup>>,
    mut labels: Query<&mut Text, With<CollectionPopupText>>,
    screens: Option<Res<super::controls::ScreenMode>>,
) {
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
        Some(format!("Collect {}", name.to_lowercase()))
    } else {
        machine
            .as_ref()
            .filter(|m| m.target.is_some() && !m.prompt.is_empty())
            .map(|m| m.prompt.clone())
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
            let label = interaction_prompt(&action);
            if text.0 != label {
                text.0 = label;
            }
        }
    }
}
/// How every interaction prompt reads: the key, then the action.
pub(crate) fn interaction_prompt(action: &str) -> String {
    format!("[{}] {action}", key_label(super::controls::INTERACT))
}
fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyE => "E",
        _ => "?",
    }
}
fn update_hud(
    keys: Res<ButtonInput<KeyCode>>,
    target: Res<super::interaction::InteractTarget>,
    mut view: ResMut<RecipeBookView>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    mut hud: Query<&mut Text, With<InventoryHud>>,
    machine: Option<Res<super::machine_config::MachineInteraction>>,
    screens: Option<Res<super::controls::ScreenMode>>,
) {
    let Ok(mut hud) = hud.single_mut() else {
        return;
    };
    if screens
        .as_ref()
        .is_some_and(|screens| screens.open.is_some())
    {
        hud.0.clear();
        return;
    }
    let nearby = target.pickup().is_some();
    let prompt = if let Some(machine) = machine.as_ref().filter(|machine| machine.target.is_some())
    {
        interaction_prompt(&machine.prompt)
    } else if nearby {
        interaction_prompt("Collect (hold for nearby pickups)")
    } else {
        "Explore for resources | approach within 3 m".to_string()
    };
    let recipe_count = catalog
        .as_ref()
        .map_or(0, |catalog| catalog.0.recipes().count());
    let recipes = if recipe_count == 0 {
        "Production recipes: none".into()
    } else {
        format!("{recipe_count} recipes available")
    };
    if keys.just_pressed(super::controls::RECIPE_BOOK) {
        view.open = !view.open;
    }
    let mut text = format!(
        "{prompt}\n{recipes} | Tab: Inventory | H: Recorder | O: Recipe book\n{}",
        player.feedback
    );
    if view.open {
        text.push_str("\n\nRECIPE BOOK\n");
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

fn replenish_nodes(
    calendar: Res<super::calendar::Calendar>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
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
            .replenish_source(&mut source, &mut player.collected, calendar.resource_time())
        {
            node.collected = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::interaction::InteractTarget;
    use super::*;
    /// Collection as the game runs it: the plugin plus the game's systems around it.
    /// Reach is avian's (interaction.rs); here the target is set directly.
    fn collection_app() -> App {
        let mut app = App::new();
        app.add_plugins(RecipesPlugin::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InteractTarget>()
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
        let mut node = ResourceNode::default();
        let mut source = node.source_object();
        catalog
            .harvest_source_at(
                &mut source,
                &mut player.items,
                &mut player.collected,
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
        let tree = app.world_mut().spawn(MineNode::scrub_oak(7)).id();
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
    fn e_collects_the_targeted_pickup_once_and_f_does_not() {
        let mut app = collection_app();
        let bug = app.world_mut().spawn(ResourceNode::lightning_bug(0)).id();
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
        let bug = app.world_mut().spawn(ResourceNode::default()).id();
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
