//! The `stubby_recipes` crafting system as a Bevy plugin.
//!
//! Thin on purpose: every rule (validation, harvesting, crafting, the recipe book) is in
//! the Bevy-free core, `stubby_recipes`. This crate only moves Bevy messages in and out
//! of it. The game keeps its world and presentation: which key collects, reach checks,
//! the components it places in levels, the calendar, the HUD.
//!
//! ```ignore
//! app.add_plugins(RecipesPlugin::default());
//! // the game decides *when* to harvest or craft:
//! harvests.write(HarvestRequest { source, now });
//! crafts.write(CraftRequest { recipe: "stream_segment_from_reeds".into() });
//! // and reacts to the results after `RecipesSystems`:
//! for collected in reader.read() { /* mark the node, play a cue */ }
//! ```
//!
//! Jackdaw's adapter compiles this file as a module of the game (against the editor's
//! Bevy version), so it must not use `crate::` paths. Its Bevy version is selected
//! by the workspace's shared Jackdaw pin; the core never depends on Bevy.
use bevy::prelude::*;
use stubby_recipes::{
    Catalog, HarvestLedger, Inventory, RecipeBook, RecipeError, ResourceTime, SourceObject, Stack,
};

/// Adds the rules catalog, the player's inventory, the request and result messages, and
/// the systems that apply them (in [`RecipesSystems`]).
#[derive(Default)]
pub struct RecipesPlugin {
    /// The rules to use; `None` loads the shipped rules (`stubby_recipes::shipped_catalog`).
    pub catalog: Option<Catalog>,
}
impl Plugin for RecipesPlugin {
    fn build(&self, app: &mut App) {
        let catalog = match &self.catalog {
            Some(catalog) => Some(catalog.clone()),
            None => match stubby_recipes::shipped_catalog() {
                Ok(catalog) => Some(catalog),
                Err(errors) => {
                    error!("Recipe rules failed validation: {errors}");
                    None
                }
            },
        };
        if let Some(catalog) = catalog {
            app.insert_resource(RecipeCatalog(catalog));
        }
        app.init_resource::<PlayerInventory>()
            .add_message::<HarvestRequest>()
            .add_message::<ResourceCollected>()
            .add_message::<CraftRequest>()
            .add_message::<CraftCompleted>()
            .add_systems(
                Update,
                (apply_harvests, apply_crafts)
                    .chain()
                    .in_set(RecipesSystems),
            );
    }
}

/// The plugin's systems. Send requests before it; read results after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecipesSystems;

/// The validated rules.
#[derive(Resource)]
pub struct RecipeCatalog(pub Catalog);

/// The player's items, harvest ledger and recipe book, plus the last feedback line.
/// One player for now; machines with their own inventories come later.
#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub items: Inventory,
    pub collected: HarvestLedger,
    pub book: RecipeBook,
    pub feedback: String,
}

/// Ask to harvest a finite source (a world pickup), at the given game time.
#[derive(Message, Clone)]
pub struct HarvestRequest {
    pub source: SourceObject,
    pub now: ResourceTime,
}

/// A harvest succeeded: the yields are in the inventory and the recipe book.
#[derive(Message, Clone)]
pub struct ResourceCollected {
    pub instance: String,
    pub yields: Vec<Stack>,
}

/// Ask to craft a recipe from the player's inventory.
#[derive(Message, Clone)]
pub struct CraftRequest {
    pub recipe: String,
}

/// A craft succeeded.
#[derive(Message, Clone)]
pub struct CraftCompleted {
    pub recipe: String,
    pub outputs: Vec<Stack>,
}

fn apply_harvests(
    mut requests: MessageReader<HarvestRequest>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    mut collected: MessageWriter<ResourceCollected>,
) {
    let Some(catalog) = catalog else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        let PlayerInventory {
            items,
            collected: ledger,
            book,
            feedback,
        } = &mut *player;
        let mut source = request.source.clone();
        match catalog
            .0
            .harvest_source_at(&mut source, items, ledger, request.now)
        {
            Ok(yields) => {
                for stack in &yields {
                    book.discover(&stack.item);
                }
                *feedback = format!(
                    "Collected {}. Available as recipe ingredients.",
                    yields
                        .iter()
                        .map(|stack| format!("{} x {}", stack.quantity, stack.item))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                collected.write(ResourceCollected {
                    instance: source.instance_id().into(),
                    yields,
                });
            }
            Err(error) => *feedback = error.to_string(),
        }
    }
}

fn apply_crafts(
    mut requests: MessageReader<CraftRequest>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    mut completed: MessageWriter<CraftCompleted>,
) {
    let Some(catalog) = catalog else {
        requests.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_harvest_request_fills_the_inventory_and_reports_it() {
        let mut app = App::new();
        app.add_plugins(RecipesPlugin::default());
        app.world_mut().write_message(HarvestRequest {
            source: SourceObject::Exhaustible {
                resource: "lightning_bug".into(),
                instance: "bug-1".into(),
                collected: false,
            },
            now: ResourceTime::default(),
        });
        app.update();
        let player = app.world().resource::<PlayerInventory>();
        assert_eq!(player.items.quantity("square_wave"), 2);
        assert!(player.book.knows("square_wave"));
        let collected: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<ResourceCollected>>()
            .drain()
            .collect();
        assert_eq!(collected.len(), 1);
    }
}
