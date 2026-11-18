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
//! crafts.write(CraftRequest { recipe: "stream_segment_from_reeds".into(), bpm: Some(90.) });
//! // and reacts to the results after `RecipesSystems`:
//! for collected in reader.read() { /* mark the node, play a cue */ }
//! ```
//!
//! Jackdaw's adapter compiles this file as a module of the game (against the editor's
//! Bevy version), so it must not use `crate::` paths. Its Bevy version is selected
//! by the workspace's shared Jackdaw pin; the core never depends on Bevy.
use bevy::prelude::*;
use stubby_recipes::{
    Catalog, GoalEvent, GoalOutcome, GoalProgress, HarvestLedger, Inventory, RecipeBook,
    RecipeError, ResourceTime, Rules, SourceObject, Stack,
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
            // Names for every system that shows items (the rack, the inventory screen).
            app.insert_resource(stubby_core::ItemNames(
                catalog
                    .item_definitions()
                    .map(|item| (item.id.clone(), item.name.clone()))
                    .collect(),
            ));
            app.insert_resource(RecipeCatalog(catalog));
        }
        // Foliage that cleans the air (`air.json`, clean-air-by-foliage), and the goals.
        if let Ok(rules) = stubby_recipes::shipped_rules() {
            app.insert_resource(AirRules(rules.air.clone()));
            if self.catalog.is_none() {
                app.insert_resource(Goals {
                    rules,
                    progress: GoalProgress::default(),
                });
            }
        }
        // A new player starts with the shipped starter inventory (`start.json`).
        if self.catalog.is_none() && !app.world().contains_resource::<PlayerInventory>() {
            let mut player = PlayerInventory::default();
            if let Ok(rules) = stubby_recipes::shipped_rules() {
                if let Err(error) = player.items.grant(&rules.start) {
                    error!("Starter inventory: {error}");
                }
                for stack in &rules.start {
                    player.book.discover(&stack.item);
                }
            }
            app.insert_resource(player);
        }
        // Dev builds pick up edits to `assets/recipes/` while the game runs.
        #[cfg(debug_assertions)]
        if self.catalog.is_none() {
            app.add_systems(First, reload::reload_rules);
        }
        app.add_message::<RulesReloaded>();
        app.init_resource::<PlayerInventory>()
            .register_type::<ResourceNode>()
            .register_type::<MineNode>()
            .add_message::<HarvestRequest>()
            .add_message::<ResourceCollected>()
            .add_message::<CraftRequest>()
            .add_message::<CraftCompleted>()
            .add_message::<InventoryFull>()
            .add_message::<GoalRequest>()
            .add_message::<GoalReported>()
            .add_systems(
                Update,
                (apply_harvests, apply_crafts, apply_goals)
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

/// Where dev builds watch for rules edits: `assets/recipes/` in this checkout unless set.
#[derive(Resource, Clone, Debug)]
pub struct RulesDir(pub std::path::PathBuf);

/// The rules on disk changed and loaded cleanly; the catalog and air rules now hold
/// them. Dev builds only (`add-crafting-designer` 4.1).
#[derive(Message, Debug, Clone, Copy)]
pub struct RulesReloaded;

#[cfg(debug_assertions)]
mod reload {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// The rules directory in this checkout.
    pub(super) fn default_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/recipes")
    }

    /// The newest modification time among the rules files.
    fn newest(dir: &std::path::Path) -> Option<SystemTime> {
        stubby_recipes::rules::FILES
            .iter()
            .filter_map(|file| std::fs::metadata(dir.join(file)).ok()?.modified().ok())
            .max()
    }

    /// Once a second, if a rules file changed, reload and validate them all: on success
    /// swap in the new catalog, item names and air rules; on failure keep the old rules
    /// and report every problem.
    pub(super) fn reload_rules(
        dir: Option<Res<RulesDir>>,
        time: Option<Res<Time<Real>>>,
        mut since: Local<f32>,
        mut seen: Local<Option<SystemTime>>,
        mut commands: Commands,
        mut reloaded: MessageWriter<RulesReloaded>,
        goals: Option<ResMut<Goals>>,
    ) {
        let Some(time) = time else { return };
        let dir = dir.map_or_else(default_dir, |dir| dir.0.clone());
        if seen.is_none() {
            // The game started with the rules as they are now.
            *seen = newest(&dir);
            return;
        }
        *since += time.delta_secs();
        if *since < 1. {
            return;
        }
        *since = 0.;
        let Some(modified) = newest(&dir) else { return };
        if *seen == Some(modified) {
            return;
        }
        *seen = Some(modified);
        let rules = match stubby_recipes::rules::load_dir(&dir) {
            Ok(rules) => rules,
            Err(errors) => {
                error!("Rules not reloaded, keeping the previous ones: {errors}");
                return;
            }
        };
        let catalog = match Catalog::new(rules.definition.clone()) {
            Ok(catalog) => catalog,
            Err(error) => {
                error!("Rules not reloaded, keeping the previous ones: {error}");
                return;
            }
        };
        commands.insert_resource(stubby_core::ItemNames(
            catalog
                .item_definitions()
                .map(|item| (item.id.clone(), item.name.clone()))
                .collect(),
        ));
        commands.insert_resource(RecipeCatalog(catalog));
        commands.insert_resource(AirRules(rules.air.clone()));
        // Progress is kept; the goals it refers to are the edited ones.
        if let Some(mut goals) = goals {
            goals.rules = rules;
        }
        reloaded.write(RulesReloaded);
        info!("Rules reloaded from {}", dir.display());
    }
}

/// The air cleaners (`air.json`): which placed assets clean the air, how far and how much.
#[derive(Resource, Clone, Default)]
pub struct AirRules(pub Vec<stubby_recipes::air::AirCleanerDefinition>);

/// The player's items, harvest ledger and recipe book, plus the last feedback line.
/// One player for now; machines with their own inventories come later.
#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub items: Inventory,
    pub collected: HarvestLedger,
    pub book: RecipeBook,
    pub feedback: String,
}

/// The goals (`goals.json`, with the rules they refer to) and the player's progress on
/// them. Change it only through [`GoalRequest`], so every change is one atomic event.
#[derive(Resource)]
pub struct Goals {
    pub rules: Rules,
    pub progress: GoalProgress,
}

/// Apply a goal event (deliver, a condition met or cleared, choose, pin) to the player's
/// progress. `Crafted` is sent for you on every [`CraftCompleted`], and the goals are
/// checked again whenever the inventory changes (`item_held` steps).
#[derive(Message, Clone)]
pub struct GoalRequest(pub GoalEvent);

/// What a goal event did: an item paid, a goal completed or opened, an unlock, a pin
/// done, or what a delivery still needs.
#[derive(Message, Clone)]
pub struct GoalReported(pub GoalOutcome);

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
    /// The tempo of the bench's clock (the nearest running burbler or flowing stream), or
    /// None with no clock in reach. A recipe with a `bpm` range is made only inside it.
    pub bpm: Option<f32>,
}

/// Something could not go into the player's inventory: no box had room for `item`
/// (the inventory is 16 boxes of up to 100). Nothing was taken or changed.
#[derive(Message, Clone)]
pub struct InventoryFull {
    pub item: String,
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
    mut full: MessageWriter<InventoryFull>,
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
            Err(RecipeError::InventoryFull(item)) => {
                feedback.clear();
                full.write(InventoryFull { item });
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
        match catalog
            .0
            .craft_at(&request.recipe, &mut player.items, request.bpm)
        {
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

fn apply_goals(
    mut requests: MessageReader<GoalRequest>,
    mut crafted: MessageReader<CraftCompleted>,
    goals: Option<ResMut<Goals>>,
    mut player: ResMut<PlayerInventory>,
    mut reported: MessageWriter<GoalReported>,
) {
    let Some(mut goals) = goals else {
        requests.clear();
        crafted.clear();
        return;
    };
    let mut events: Vec<_> = crafted
        .read()
        .map(|craft| GoalEvent::Crafted(craft.recipe.clone()))
        .chain(requests.read().map(|request| request.0.clone()))
        .collect();
    // Held items count as they change (harvests, the rack, stream building), and the
    // first run checks what the player starts with.
    if player.is_changed() || goals.is_added() {
        events.push(GoalEvent::Refresh);
    }
    if events.is_empty() {
        return;
    }
    let Goals { rules, progress } = &mut *goals;
    let PlayerInventory { items, book, .. } = &mut *player;
    for event in events {
        match stubby_recipes::apply_goal_event(rules, progress, items, book, event) {
            Ok(outcomes) => {
                reported.write_batch(outcomes.into_iter().map(GoalReported));
            }
            Err(error) => warn!("Goal event refused: {error}"),
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

    #[test]
    fn a_met_condition_completes_its_step_once() {
        let mut app = App::new();
        app.add_plugins(RecipesPlugin::default());
        // A fixture, never the shipped goal list.
        let camp = stubby_recipes::GoalDefinition {
            id: "camp".into(),
            name: "Camp".into(),
            description: String::new(),
            scale: stubby_recipes::Scale::Major,
            part_of: None,
            requires: vec![],
            condition: None,
            after: vec![],
            unlocks: vec![],
        };
        let beat = stubby_recipes::GoalDefinition {
            id: "beat".into(),
            scale: stubby_recipes::Scale::Minor,
            part_of: Some("camp".into()),
            condition: Some(stubby_recipes::Condition::MachineConfigured {
                machine: "mudpit_burbler".into(),
            }),
            ..camp.clone()
        };
        let goals = vec![camp, beat];
        app.world_mut().resource_mut::<Goals>().rules.goals = goals;
        let configured = GoalRequest(GoalEvent::ConditionMet(
            stubby_recipes::Condition::MachineConfigured {
                machine: "mudpit_burbler".into(),
            },
        ));
        app.world_mut().write_message(configured.clone());
        app.update();
        app.world_mut().write_message(configured);
        app.update();
        let progress = &app.world().resource::<Goals>().progress;
        assert!(progress.is_complete("beat") && progress.is_complete("camp"));
        let completed = app
            .world_mut()
            .resource_mut::<Messages<GoalReported>>()
            .drain()
            .filter(|r| matches!(&r.0, GoalOutcome::GoalCompleted(_)))
            .count();
        assert_eq!(completed, 2, "the step and its major, once each");
    }
}

/// A finite source placed in the world: which resource it is, its instance ID, and
/// whether it has been collected this availability period. The game decides how it is
/// collected (E pickup, a machine); this plugin owns what it is.
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct ResourceNode {
    pub resource: String,
    pub instance: String,
    pub collected: bool,
}
impl ResourceNode {
    pub fn source_object(&self) -> stubby_recipes::SourceObject {
        stubby_recipes::SourceObject::Exhaustible {
            resource: self.resource.clone(),
            instance: self.instance.clone(),
            collected: self.collected,
        }
    }
}

/// An infinite source (a mine) placed in the world. Never a finite pickup.
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct MineNode {
    pub mine: String,
    pub instance: String,
}
impl MineNode {
    pub fn source_object(&self) -> stubby_recipes::SourceObject {
        stubby_recipes::SourceObject::InfiniteMine {
            mine: self.mine.clone(),
            instance: self.instance.clone(),
        }
    }
}

/// The player's inventory is the stock other systems (the rack) take from and give to.
impl stubby_core::Stock for PlayerInventory {
    fn quantity(&self, item: &str) -> u32 {
        self.items.quantity(item)
    }
    fn boxes(&self) -> Vec<(String, u32)> {
        self.items
            .boxes()
            .map(|(item, count)| (item.to_owned(), count))
            .collect()
    }
    fn box_count(&self) -> usize {
        stubby_recipes::INVENTORY_BOXES as usize
    }
    fn take(&mut self, item: &str, count: u32) -> bool {
        self.items.take(item, count).is_ok()
    }
    fn give(&mut self, item: &str, count: u32) -> bool {
        self.items
            .grant(&[stubby_recipes::Stack {
                item: item.to_owned(),
                quantity: count,
            }])
            .is_ok()
    }
    fn say(&mut self, message: String) {
        self.feedback = message;
    }
}

#[cfg(all(test, debug_assertions))]
mod reload_tests {
    use super::*;
    #[test]
    fn edited_rules_reload_and_broken_ones_are_kept_out() {
        let dir = std::env::temp_dir().join(format!("stubby-rules-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for file in stubby_recipes::rules::FILES {
            std::fs::copy(reload::default_dir().join(file), dir.join(file)).unwrap();
        }
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RulesDir(dir.clone()))
            .add_plugins(RecipesPlugin::default());
        let tick = |app: &mut App| {
            std::thread::sleep(std::time::Duration::from_millis(1100));
            app.update();
        };
        tick(&mut app); // first look: the files the game started with
        let radius = |app: &App| {
            app.world()
                .resource::<AirRules>()
                .0
                .iter()
                .find(|a| a.asset == "scrub_oak")
                .unwrap()
                .radius_m
        };
        assert_eq!(radius(&app), 18.);
        let air = dir.join("air.json");
        let text = std::fs::read_to_string(&air).unwrap();
        std::fs::write(
            &air,
            text.replace("\"radius_m\": 18.0", "\"radius_m\": 25.0"),
        )
        .unwrap();
        tick(&mut app);
        tick(&mut app);
        assert_eq!(radius(&app), 25.);
        // A broken edit is reported and the last good rules stay.
        std::fs::write(&air, "{ not json").unwrap();
        tick(&mut app);
        tick(&mut app);
        assert_eq!(radius(&app), 25.);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
