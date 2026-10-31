//! World resource collection and the first inventory/recipe boundary.
use bevy::prelude::*;
use stubby_recipes::{
    Catalog, HarvestLedger, Inventory, RecipeBook, RecipeError, SignalUse, Stack,
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
    pub fn lightning_bug(index: usize) -> Self {
        Self {
            instance: format!("lightning_bug-{index}"),
            ..default()
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
            .add_message::<CraftRequest>()
            .add_message::<CraftCompleted>()
            .add_systems(Startup, setup)
            .add_systems(Update, (collect_nearby, craft_requests, update_hud).chain());
    }
}
fn setup(mut commands: Commands) {
    match Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")) {
        Ok(catalog) => commands.insert_resource(RecipeCatalog(catalog)),
        Err(error) => error!("Recipe catalog failed validation: {error}"),
    }
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
                transform.translation.distance_squared(camera.translation),
                node.instance.clone(),
            )
        })
        .filter(|(distance, _)| *distance <= 9.)
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
        match catalog
            .0
            .harvest(&node.resource, &node.instance, items, collected)
        {
            Ok(yields) => {
                for stack in yields {
                    book.discover(&stack.item);
                }
                node.collected = true;
                *feedback = "Square wave recovered. Available as a recipe ingredient.".into();
            }
            Err(error) => *feedback = error.to_string(),
        }
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
    let nearby = camera.single().ok().is_some_and(|camera| {
        nodes.iter().any(|(t, node)| {
            !node.collected && t.translation.distance_squared(camera.translation) <= 9.
        })
    });
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
    fn holding_e_collects_new_resources_while_f_is_not_collect() {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
        ))
        .init_resource::<PlayerInventory>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, collect_nearby);
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        for (index, x) in [1., 2., 20.].into_iter().enumerate() {
            app.world_mut().spawn((
                Transform::from_xyz(x, 0., 0.),
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
            1
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
            3
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
            1
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
            1
        );
    }
}
