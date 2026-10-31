//! Item quantities, resource yields and atomic recipes. No Bevy or audio dependency.
mod source;
pub use source::{MineDefinition, ProductivityFactor, SourceKind, SourceObject};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalUse {
    Vco,
    Lfo,
    Gate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Waveform {
    Square,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub waveform: Option<Waveform>,
    #[serde(default)]
    pub uses: Vec<SignalUse>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Stack {
    pub item: String,
    pub quantity: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResourceDefinition {
    pub id: String,
    pub name: String,
    pub yields: Vec<Stack>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecipeDefinition {
    pub id: String,
    pub name: String,
    pub inputs: Vec<Stack>,
    pub outputs: Vec<Stack>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogDefinition {
    pub items: Vec<ItemDefinition>,
    pub resources: Vec<ResourceDefinition>,
    #[serde(default)]
    pub mines: Vec<MineDefinition>,
    pub recipes: Vec<RecipeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RecipeError {
    #[error("invalid catalog: {0}")]
    InvalidCatalog(String),
    #[error("unknown recipe: {0}")]
    UnknownRecipe(String),
    #[error("unknown resource: {0}")]
    UnknownResource(String),
    #[error("unknown resource source: {0}")]
    UnknownSource(String),
    #[error("mine requires production, not finite collection: {0}")]
    MineRequiresProduction(String),
    #[error("resource instance already collected: {0}")]
    AlreadyCollected(String),
    #[error("need {needed} {item}; have {available}")]
    MissingIngredient {
        item: String,
        needed: u32,
        available: u32,
    },
    #[error("inventory quantity overflow for {0}")]
    InventoryOverflow(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inventory {
    quantities: BTreeMap<String, u32>,
}
impl Inventory {
    pub fn quantity(&self, item: &str) -> u32 {
        self.quantities.get(item).copied().unwrap_or(0)
    }
    pub fn items(&self) -> impl Iterator<Item = (&str, u32)> {
        self.quantities
            .iter()
            .map(|(item, count)| (item.as_str(), *count))
    }
    fn add(&mut self, item: &str, quantity: u32) -> Result<(), RecipeError> {
        let count = self
            .quantity(item)
            .checked_add(quantity)
            .ok_or_else(|| RecipeError::InventoryOverflow(item.into()))?;
        self.quantities.insert(item.into(), count);
        Ok(())
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HarvestLedger {
    instances: BTreeSet<String>,
}
impl HarvestLedger {
    pub fn collected(&self, instance: &str) -> bool {
        self.instances.contains(instance)
    }
}

/// Learning an ingredient survives consuming its inventory quantity in a recipe.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeBook {
    discovered: BTreeSet<String>,
}
impl RecipeBook {
    pub fn discover(&mut self, item: &str) {
        self.discovered.insert(item.into());
    }
    pub fn knows(&self, item: &str) -> bool {
        self.discovered.contains(item)
    }
    pub fn items(&self) -> impl Iterator<Item = &str> {
        self.discovered.iter().map(String::as_str)
    }
}

pub struct Catalog {
    items: BTreeMap<String, ItemDefinition>,
    resources: BTreeMap<String, ResourceDefinition>,
    mines: BTreeMap<String, MineDefinition>,
    recipes: BTreeMap<String, RecipeDefinition>,
}
fn insert_unique<T>(map: &mut BTreeMap<String, T>, id: &str, value: T) -> Result<(), RecipeError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(RecipeError::InvalidCatalog(format!("invalid ID {id:?}")));
    }
    if map.insert(id.into(), value).is_some() {
        return Err(RecipeError::InvalidCatalog(format!("duplicate ID {id}")));
    }
    Ok(())
}
fn aggregate(stacks: &[Stack]) -> Result<BTreeMap<String, u32>, RecipeError> {
    let mut totals = BTreeMap::new();
    for stack in stacks {
        if stack.quantity == 0 {
            return Err(RecipeError::InvalidCatalog(format!(
                "zero quantity for {}",
                stack.item
            )));
        }
        let total = totals.entry(stack.item.clone()).or_insert(0_u32);
        *total = total.checked_add(stack.quantity).ok_or_else(|| {
            RecipeError::InvalidCatalog(format!("quantity overflow for {}", stack.item))
        })?;
    }
    Ok(totals)
}
impl Catalog {
    pub fn from_json(json: &str) -> Result<Self, RecipeError> {
        let definition =
            serde_json::from_str(json).map_err(|e| RecipeError::InvalidCatalog(format!("{e}")))?;
        Self::new(definition)
    }
    pub fn new(definition: CatalogDefinition) -> Result<Self, RecipeError> {
        let mut catalog = Self {
            items: BTreeMap::new(),
            resources: BTreeMap::new(),
            mines: BTreeMap::new(),
            recipes: BTreeMap::new(),
        };
        for item in definition.items {
            insert_unique(&mut catalog.items, &item.id.clone(), item)?;
        }
        for resource in definition.resources {
            catalog.validate_stacks(&resource.yields)?;
            insert_unique(&mut catalog.resources, &resource.id.clone(), resource)?;
        }
        for mine in definition.mines {
            if catalog.resources.contains_key(&mine.id) {
                return Err(RecipeError::InvalidCatalog(format!(
                    "source ID reused as a mine: {}",
                    mine.id
                )));
            }
            if !catalog.resources.contains_key(&mine.produces_resource) {
                return Err(RecipeError::InvalidCatalog(format!(
                    "unknown mine output resource {}",
                    mine.produces_resource
                )));
            }
            for (index, factor) in mine.productivity_factors.iter().enumerate() {
                if mine.productivity_factors[..index].contains(factor) {
                    return Err(RecipeError::InvalidCatalog(format!(
                        "duplicate productivity factor in {}",
                        mine.id
                    )));
                }
            }
            insert_unique(&mut catalog.mines, &mine.id.clone(), mine)?;
        }
        for recipe in definition.recipes {
            catalog.validate_stacks(&recipe.inputs)?;
            catalog.validate_stacks(&recipe.outputs)?;
            insert_unique(&mut catalog.recipes, &recipe.id.clone(), recipe)?;
        }
        Ok(catalog)
    }
    fn validate_stacks(&self, stacks: &[Stack]) -> Result<(), RecipeError> {
        if stacks.is_empty() {
            return Err(RecipeError::InvalidCatalog(
                "empty ingredient/output list".into(),
            ));
        }
        for item in aggregate(stacks)?.keys() {
            if !self.items.contains_key(item) {
                return Err(RecipeError::InvalidCatalog(format!("unknown item {item}")));
            }
        }
        Ok(())
    }
    pub fn item(&self, id: &str) -> Option<&ItemDefinition> {
        self.items.get(id)
    }
    pub fn recipe(&self, id: &str) -> Option<&RecipeDefinition> {
        self.recipes.get(id)
    }
    pub fn resource(&self, id: &str) -> Option<&ResourceDefinition> {
        self.resources.get(id)
    }
    pub fn mine(&self, id: &str) -> Option<&MineDefinition> {
        self.mines.get(id)
    }
    pub fn mines(&self) -> impl Iterator<Item = &MineDefinition> {
        self.mines.values()
    }
    /// Create a validated object without spawning anything or granting inventory.
    pub fn source(&self, definition: &str, instance: &str) -> Result<SourceObject, RecipeError> {
        if self.resources.contains_key(definition) {
            Ok(SourceObject::Exhaustible {
                resource: definition.into(),
                instance: instance.into(),
                collected: false,
            })
        } else if self.mines.contains_key(definition) {
            Ok(SourceObject::InfiniteMine {
                mine: definition.into(),
                instance: instance.into(),
            })
        } else {
            Err(RecipeError::UnknownSource(definition.into()))
        }
    }
    /// Finite collection is never a shortcut for infinite mine production.
    pub fn harvest_source(
        &self,
        source: &mut SourceObject,
        inventory: &mut Inventory,
        ledger: &mut HarvestLedger,
    ) -> Result<Vec<Stack>, RecipeError> {
        match source {
            SourceObject::Exhaustible {
                resource,
                instance,
                collected,
            } => {
                if *collected {
                    return Err(RecipeError::AlreadyCollected(instance.clone()));
                }
                let yields = self.harvest(resource, instance, inventory, ledger)?;
                *collected = true;
                Ok(yields)
            }
            SourceObject::InfiniteMine { mine, .. } => {
                Err(RecipeError::MineRequiresProduction(mine.clone()))
            }
        }
    }
    pub fn recipes(&self) -> impl Iterator<Item = &RecipeDefinition> {
        self.recipes.values()
    }
    pub fn missing_ingredients(
        &self,
        recipe: &str,
        inventory: &Inventory,
    ) -> Result<Vec<Stack>, RecipeError> {
        let recipe = self
            .recipes
            .get(recipe)
            .ok_or_else(|| RecipeError::UnknownRecipe(recipe.into()))?;
        Ok(aggregate(&recipe.inputs)?
            .into_iter()
            .filter_map(|(item, needed)| {
                let available = inventory.quantity(&item);
                (available < needed).then(|| Stack {
                    item,
                    quantity: needed - available,
                })
            })
            .collect())
    }
    pub fn craft(
        &self,
        recipe: &str,
        inventory: &mut Inventory,
    ) -> Result<Vec<Stack>, RecipeError> {
        let recipe = self
            .recipes
            .get(recipe)
            .ok_or_else(|| RecipeError::UnknownRecipe(recipe.into()))?;
        let mut staged = inventory.clone();
        for (item, needed) in aggregate(&recipe.inputs)? {
            let available = staged.quantity(&item);
            if available < needed {
                return Err(RecipeError::MissingIngredient {
                    item,
                    needed,
                    available,
                });
            }
            if available == needed {
                staged.quantities.remove(&item);
            } else {
                staged.quantities.insert(item, available - needed);
            }
        }
        for (item, quantity) in aggregate(&recipe.outputs)? {
            staged.add(&item, quantity)?;
        }
        *inventory = staged;
        Ok(recipe.outputs.clone())
    }
    pub fn harvest(
        &self,
        resource: &str,
        instance: &str,
        inventory: &mut Inventory,
        ledger: &mut HarvestLedger,
    ) -> Result<Vec<Stack>, RecipeError> {
        let resource = self
            .resources
            .get(resource)
            .ok_or_else(|| RecipeError::UnknownResource(resource.into()))?;
        if ledger.collected(instance) {
            return Err(RecipeError::AlreadyCollected(instance.into()));
        }
        let mut staged = inventory.clone();
        for (item, quantity) in aggregate(&resource.yields)? {
            staged.add(&item, quantity)?;
        }
        *inventory = staged;
        ledger.instances.insert(instance.into());
        Ok(resource.yields.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> CatalogDefinition {
        serde_json::from_str(r#"{
            "items":[{"id":"square_wave","name":"Square wave","description":"Waveform resource"},{"id":"test_module","name":"Test output","description":"Test-only recipe output"}],
            "resources":[{"id":"lightning_bug","name":"Lightning bug","yields":[{"item":"square_wave","quantity":1}]}],
            "recipes":[{"id":"test_recipe","name":"Test only","inputs":[{"item":"square_wave","quantity":1},{"item":"square_wave","quantity":1}],"outputs":[{"item":"test_module","quantity":1}]}]
        }"#).unwrap()
    }
    #[test]
    fn finite_and_infinite_source_objects_preserve_distinct_lifecycles() {
        let catalog =
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap();
        let mut bug = catalog.source("lightning_bug", "spawn-1").unwrap();
        let mut tree = catalog.source("scrub_oak", "tree-1").unwrap();
        assert_eq!(bug.kind(), SourceKind::Exhaustible);
        assert_eq!(tree.kind(), SourceKind::InfiniteMine);
        let original_tree = tree.clone();
        let mut inventory = Inventory::default();
        let mut ledger = HarvestLedger::default();
        catalog
            .harvest_source(&mut bug, &mut inventory, &mut ledger)
            .unwrap();
        assert_eq!(inventory.quantity("square_wave"), 2);
        assert!(matches!(
            catalog.harvest_source(&mut bug, &mut inventory, &mut ledger),
            Err(RecipeError::AlreadyCollected(_))
        ));
        for _ in 0..3 {
            assert!(matches!(
                catalog.harvest_source(&mut tree, &mut inventory, &mut ledger),
                Err(RecipeError::MineRequiresProduction(_))
            ));
        }
        assert_eq!(tree, original_tree);
        assert!(!ledger.collected("tree-1"));
        assert_eq!(inventory.quantity("square_wave"), 2);
        for object in [bug, tree] {
            let json = serde_json::to_string(&object).unwrap();
            assert_eq!(serde_json::from_str::<SourceObject>(&json).unwrap(), object);
        }
        let mine = catalog.mine("scrub_oak").unwrap();
        assert_eq!(mine.produces_resource, "lightning_bug");
        assert_eq!(
            mine.productivity_factors,
            vec![
                ProductivityFactor::WaterQuality,
                ProductivityFactor::LightQuality
            ]
        );
    }
    #[test]
    fn mine_validation_rejects_unknown_outputs_duplicate_factors_and_reused_source_ids() {
        let shipped: CatalogDefinition =
            serde_json::from_str(include_str!("../../../assets/recipes/catalog.json")).unwrap();
        let mut bad = shipped.clone();
        bad.mines[0].produces_resource = "square_wave".into(); // item is not a resource
        assert!(Catalog::new(bad).is_err());
        let mut bad = shipped.clone();
        bad.mines[0]
            .productivity_factors
            .push(ProductivityFactor::WaterQuality);
        assert!(Catalog::new(bad).is_err());
        let mut bad = shipped.clone();
        bad.mines.push(bad.mines[0].clone());
        assert!(Catalog::new(bad).is_err());
        let mut bad = shipped;
        bad.mines[0].id = "lightning_bug".into();
        assert!(Catalog::new(bad).is_err());
    }
    #[test]
    fn failed_object_harvest_does_not_exhaust_the_resource() {
        let catalog = Catalog::new(fixture()).unwrap();
        let mut object = catalog.source("lightning_bug", "bug-1").unwrap();
        let before = object.clone();
        let mut inventory = Inventory::default();
        inventory.add("square_wave", u32::MAX).unwrap();
        let mut ledger = HarvestLedger::default();
        assert!(
            catalog
                .harvest_source(&mut object, &mut inventory, &mut ledger)
                .is_err()
        );
        assert_eq!(object, before);
        assert!(!ledger.collected("bug-1"));
    }

    #[test]
    fn harvest_once_then_craft_aggregates_duplicate_inputs() {
        let catalog = Catalog::new(fixture()).unwrap();
        let mut inventory = Inventory::default();
        let mut ledger = HarvestLedger::default();
        catalog
            .harvest("lightning_bug", "bug-1", &mut inventory, &mut ledger)
            .unwrap();
        let before = inventory.clone();
        assert!(matches!(
            catalog.harvest("lightning_bug", "bug-1", &mut inventory, &mut ledger),
            Err(RecipeError::AlreadyCollected(_))
        ));
        assert_eq!(inventory, before);
        assert_eq!(
            catalog
                .missing_ingredients("test_recipe", &inventory)
                .unwrap(),
            vec![Stack {
                item: "square_wave".into(),
                quantity: 1
            }]
        );
        assert!(matches!(
            catalog.craft("test_recipe", &mut inventory),
            Err(RecipeError::MissingIngredient {
                needed: 2,
                available: 1,
                ..
            })
        ));
        assert_eq!(inventory, before);
        catalog
            .harvest("lightning_bug", "bug-2", &mut inventory, &mut ledger)
            .unwrap();
        catalog.craft("test_recipe", &mut inventory).unwrap();
        assert_eq!(inventory.quantity("square_wave"), 0);
        assert_eq!(inventory.quantity("test_module"), 1);
    }
    #[test]
    fn invalid_catalogs_are_rejected_and_empty_recipes_are_valid() {
        let mut definition = fixture();
        definition.recipes.clear();
        assert!(Catalog::new(definition).is_ok());
        let mut definition = fixture();
        definition.items.push(definition.items[0].clone());
        assert!(Catalog::new(definition).is_err());
        let mut definition = fixture();
        definition.resources[0].yields[0].quantity = 0;
        assert!(Catalog::new(definition).is_err());
        let mut definition = fixture();
        definition.recipes[0].outputs[0].item = "unknown".into();
        assert!(Catalog::new(definition).is_err());
    }
    #[test]
    fn output_overflow_rolls_back_inputs_and_harvest_ledger() {
        let catalog = Catalog::new(fixture()).unwrap();
        let mut inventory = Inventory::default();
        let mut ledger = HarvestLedger::default();
        inventory.add("square_wave", 2).unwrap();
        inventory.add("test_module", u32::MAX).unwrap();
        let before = inventory.clone();
        assert!(matches!(
            catalog.craft("test_recipe", &mut inventory),
            Err(RecipeError::InventoryOverflow(_))
        ));
        assert_eq!(inventory, before);
        inventory.quantities.insert("square_wave".into(), u32::MAX);
        assert!(
            catalog
                .harvest("lightning_bug", "new-bug", &mut inventory, &mut ledger)
                .is_err()
        );
        assert!(!ledger.collected("new-bug"));
    }
    #[test]
    fn recipe_book_keeps_waveform_knowledge_when_ingredients_are_consumed() {
        let catalog = Catalog::new(fixture()).unwrap();
        let mut inventory = Inventory::default();
        let mut ledger = HarvestLedger::default();
        let mut book = RecipeBook::default();
        for instance in ["bug-1", "bug-2"] {
            for stack in catalog
                .harvest("lightning_bug", instance, &mut inventory, &mut ledger)
                .unwrap()
            {
                book.discover(&stack.item);
            }
        }
        catalog.craft("test_recipe", &mut inventory).unwrap();
        assert_eq!(inventory.quantity("square_wave"), 0);
        assert!(book.knows("square_wave"));
    }

    #[test]
    fn shipped_catalog_yields_a_square_wave_without_a_fake_production_recipe() {
        let catalog =
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap();
        let square = catalog.item("square_wave").unwrap();
        assert_eq!(square.waveform, Some(Waveform::Square));
        assert_eq!(
            square.uses,
            vec![SignalUse::Vco, SignalUse::Lfo, SignalUse::Gate]
        );
        assert_eq!(
            catalog.resource("lightning_bug").unwrap().yields,
            vec![Stack {
                item: "square_wave".into(),
                quantity: 2
            }]
        );
    }
}
