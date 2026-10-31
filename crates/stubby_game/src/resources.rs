//! World resource collection and the first inventory/recipe boundary.
use bevy::prelude::*;
use stubby_recipes::{
    Catalog, HarvestLedger, Inventory, RecipeBook, RecipeError, SignalUse, SourceObject, Stack,
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

#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub items: Inventory,
    pub collected: HarvestLedger,
    pub book: RecipeBook,
    pub feedback: String,
}
#[derive(Resource)]
pub struct RecipeCatalog(pub Catalog);

/// Future machines and crafting interfaces use this same transaction boundary.
/// Successful world harvest; presentation removes the depleted resource's effects.
#[derive(Message, Clone)]
pub struct ResourceCollected {
    pub instance: String,
    pub yields: Vec<Stack>,
}
#[derive(Message, Clone)]
pub struct CraftRequest {
    pub recipe: String,
}
#[derive(Message, Clone)]
pub struct CraftCompleted {
    pub recipe: String,
    pub outputs: Vec<Stack>,
}
#[derive(Component)]
struct InventoryHud;
#[derive(Component)]
struct CollectionPopup;
const COLLECTION_RADIUS_SQUARED: f32 = 9.;
fn ground_distance_squared(a: &Transform, b: &Transform) -> f32 {
    let delta = a.translation - b.translation;
    delta.x * delta.x + delta.z * delta.z
}
fn within_reach(camera: &Transform, target: &Transform, node: &ResourceNode) -> bool {
    !node.collected && ground_distance_squared(camera, target) <= COLLECTION_RADIUS_SQUARED
}
#[derive(Resource, Default)]
struct RecipeBookView {
    open: bool,
}

pub struct ResourceRecipesPlugin;
impl Plugin for ResourceRecipesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInventory>()
            .init_resource::<RecipeBookView>()
            .register_type::<ResourceNode>()
            .register_type::<MineNode>()
            .add_message::<ResourceCollected>()
            .add_message::<CraftRequest>()
            .add_message::<CraftCompleted>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    collect_nearby,
                    craft_requests,
                    update_hud,
                    update_collection_popup,
                )
                    .chain(),
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
                ImageNode::new(images.add(reticle)),
                Node {
                    width: px(28),
                    height: px(28),
                    ..default()
                },
            ));
        });
    match Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")) {
        Ok(catalog) => commands.insert_resource(RecipeCatalog(catalog)),
        Err(error) => error!("Recipe catalog failed validation: {error}"),
    }
    commands
        .spawn((
            CollectionPopup,
            Visibility::Hidden,
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
        Text::new("Square waves: 0"),
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
fn collect_nearby(
    keys: Res<ButtonInput<KeyCode>>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    camera: Query<&Transform, With<Camera3d>>,
    mut nodes: Query<(&Transform, &mut ResourceNode)>,
    mut events: MessageWriter<ResourceCollected>,
) {
    if !keys.pressed(super::controls::INTERACT) {
        return;
    }
    let (Some(catalog), Ok(camera)) = (catalog, camera.single()) else {
        return;
    };
    let candidate = nodes
        .iter_mut()
        .filter(|(_, node)| !node.collected)
        .map(|(transform, node)| {
            (
                ground_distance_squared(transform, camera),
                node.instance.clone(),
            )
        })
        .filter(|(distance, _)| *distance <= COLLECTION_RADIUS_SQUARED)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, instance)) = candidate else {
        player.feedback = "Move within 3 m of a lightning bug to recover its square wave.".into();
        return;
    };
    if let Some((_, mut node)) = nodes.iter_mut().find(|(_, node)| node.instance == instance) {
        let PlayerInventory {
            items,
            collected,
            book,
            feedback,
        } = &mut *player;
        let mut source = node.source_object();
        match catalog.0.harvest_source(&mut source, items, collected) {
            Ok(yields) => {
                for stack in &yields {
                    book.discover(&stack.item);
                }
                node.collected = true;
                events.write(ResourceCollected {
                    instance: node.instance.clone(),
                    yields: yields.clone(),
                });
                *feedback = format!(
                    "Collected {}. Available as recipe ingredients.",
                    yields
                        .iter()
                        .map(|stack| format!("{} x {}", stack.quantity, stack.item))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            Err(error) => *feedback = error.to_string(),
        }
    }
}
fn update_collection_popup(
    camera: Query<&Transform, With<Camera3d>>,
    nodes: Query<(&Transform, &ResourceNode)>,
    mut popups: Query<&mut Visibility, With<CollectionPopup>>,
) {
    let nearby = camera.single().ok().is_some_and(|camera| {
        nodes
            .iter()
            .any(|(target, node)| within_reach(camera, target, node))
    });
    for mut visibility in &mut popups {
        *visibility = if nearby {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn craft_requests(
    mut requests: MessageReader<CraftRequest>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    mut completed: MessageWriter<CraftCompleted>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    for request in requests.read() {
        match catalog.0.craft(&request.recipe, &mut player.items) {
            Ok(outputs) => {
                player.feedback = format!("Crafted {}.", request.recipe);
                completed.write(CraftCompleted {
                    recipe: request.recipe.clone(),
                    outputs,
                });
            }
            Err(RecipeError::MissingIngredient {
                item,
                needed,
                available,
            }) => {
                player.feedback = format!("Need {needed} {item}; have {available}.");
            }
            Err(error) => player.feedback = error.to_string(),
        }
    }
}
fn update_hud(
    keys: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<RecipeBookView>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    camera: Query<&Transform, With<Camera3d>>,
    nodes: Query<(&Transform, &ResourceNode)>,
    mut hud: Query<&mut Text, With<InventoryHud>>,
) {
    let Ok(mut hud) = hud.single_mut() else {
        return;
    };
    let nearby = camera
        .single()
        .ok()
        .is_some_and(|camera| nodes.iter().any(|(t, node)| within_reach(camera, t, node)));
    let prompt = if nearby {
        "E | Recover square wave (hold to collect)"
    } else {
        "Find a flashing lightning bug | approach within 3 m"
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
        "Square waves: {}\n{prompt}\n{recipes} | O: Recipe book\n{}",
        player.items.quantity("square_wave"),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mine_objects_are_authorable_but_not_finite_collection_nodes() {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
        ))
        .init_resource::<PlayerInventory>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<ResourceCollected>()
        .add_systems(Update, collect_nearby);
        app.world_mut()
            .spawn((Camera3d::default(), Transform::default()));
        let tree = app
            .world_mut()
            .spawn((MineNode::scrub_oak(7), Transform::default()))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
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
        assert_eq!(mine.definition_id(), "scrub_oak");
        assert_eq!(mine.instance_id(), "scrub_oak-7");
    }

    #[test]
    fn popup_matches_collection_range_and_hides_after_harvest() {
        let mut app = App::new();
        app.add_systems(Update, update_collection_popup);
        app.world_mut()
            .spawn((Camera3d::default(), Transform::default()));
        let bug = app
            .world_mut()
            .spawn((ResourceNode::default(), Transform::from_xyz(0., 3., -3.)))
            .id();
        let popup = app
            .world_mut()
            .spawn((CollectionPopup, Visibility::Hidden))
            .id();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(popup).unwrap(),
            Visibility::Visible
        );
        app.world_mut()
            .get_mut::<Transform>(bug)
            .unwrap()
            .translation
            .z = -3.01;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(popup).unwrap(),
            Visibility::Hidden
        );
        app.world_mut()
            .get_mut::<Transform>(bug)
            .unwrap()
            .translation
            .z = -2.;
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

    #[test]
    fn holding_e_collects_new_resources_while_f_is_not_collect() {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
        ))
        .init_resource::<PlayerInventory>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<ResourceCollected>()
        .add_systems(Update, collect_nearby);
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        for (index, x) in [1., 2., 20.].into_iter().enumerate() {
            app.world_mut().spawn((
                Transform::from_xyz(x, 3., 0.),
                ResourceNode::lightning_bug(index),
            ));
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            2
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            4
        );
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            4
        );
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation
            .x = 19.;
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            6
        );
    }

    #[test]
    fn interaction_collects_only_one_nearby_instance_and_cannot_duplicate_it() {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
        ))
        .init_resource::<PlayerInventory>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<ResourceCollected>()
        .add_systems(Update, collect_nearby);
        app.world_mut()
            .spawn((Camera3d::default(), Transform::default()));
        app.world_mut().spawn((
            Transform::from_xyz(1., 0., 0.),
            ResourceNode::lightning_bug(0),
        ));
        app.world_mut().spawn((
            Transform::from_xyz(20., 0., 0.),
            ResourceNode::lightning_bug(1),
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert!(
            app.world()
                .resource::<PlayerInventory>()
                .book
                .knows("square_wave")
        );
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            2
        );
        app.update();
        assert!(
            app.world()
                .resource::<PlayerInventory>()
                .book
                .knows("square_wave")
        );
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("square_wave"),
            2
        );
    }
}
