//! The rules files under `assets/recipes/`: loading, strict validation with errors that
//! say where, migrations, the ID lock and the canonical file format.
//!
//! The designer owns the rows; this module owns the schema (AGENTS.md, "Rules are data;
//! the schema is code"). A new row of an existing type needs no code; a new rule type
//! is an OpenSpec change, then code here, then rows.
use crate::{
    CatalogDefinition, ItemDefinition, MineDefinition, RecipeDefinition, ResourceDefinition, Stack,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The file format every rules file is written in. Bump it (and migrate the files)
/// only when a change to the schema is not backwards compatible.
pub const FORMAT: u32 = 1;

pub const ITEMS: &str = "items.json";
pub const RESOURCES: &str = "resources.json";
pub const MINES: &str = "mines.json";
pub const RECIPES: &str = "recipes.json";
pub const MIGRATIONS: &str = "migrations.json";
pub const IDS_LOCK: &str = "ids.lock";

/// One problem in the rules, located well enough to fix without searching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesError {
    pub file: String,
    /// The row's ID, or empty when the problem is not in one row.
    pub id: String,
    /// The field, or empty when the problem is the whole row or file.
    pub field: String,
    pub message: String,
}
impl RulesError {
    fn new(file: &str, id: &str, field: &str, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            id: id.into(),
            field: field.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for RulesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        if !self.id.is_empty() {
            write!(f, " [{}]", self.id)?;
        }
        if !self.field.is_empty() {
            write!(f, " .{}", self.field)?;
        }
        write!(f, ": {}", self.message)
    }
}

/// Every problem found in one load, so one run reports them all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesErrors(pub Vec<RulesError>);
impl fmt::Display for RulesErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} problem(s) in the rules:", self.0.len())?;
        for error in &self.0 {
            writeln!(f, "  {error}")?;
        }
        Ok(())
    }
}
impl std::error::Error for RulesErrors {}

macro_rules! rules_file {
    ($name:ident, $field:ident, $row:ty) => {
        #[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            /// Points editors at the generated JSON Schema, for autocomplete.
            #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
            pub schema: Option<String>,
            pub format: u32,
            pub $field: Vec<$row>,
        }
    };
}
rules_file!(ItemsFile, items, ItemDefinition);
rules_file!(ResourcesFile, resources, ResourceDefinition);
rules_file!(MinesFile, mines, MineDefinition);
rules_file!(RecipesFile, recipes, RecipeDefinition);
rules_file!(MigrationsFile, migrations, Migration);

/// A renamed (`to` set) or removed (`to` null) rule ID. IDs are save-game API: saves
/// record the migrations they have applied and apply the rest on load.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Migration {
    pub id: String,
    /// Which rules file the ID lived in. Before IDs shared one namespace, two
    /// registries could use the same ID (the item and the resource `clock`).
    pub registry: Registry,
    pub from: String,
    pub to: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Registry {
    Items,
    Resources,
    Mines,
    Recipes,
}
impl Registry {
    fn file(self) -> &'static str {
        match self {
            Self::Items => ITEMS,
            Self::Resources => RESOURCES,
            Self::Mines => MINES,
            Self::Recipes => RECIPES,
        }
    }
}

/// The text of every rules file, so callers choose where it comes from (embedded,
/// read from disk, or a test's fixture).
#[derive(Debug, Clone, Copy)]
pub struct RulesFiles<'a> {
    pub items: &'a str,
    pub resources: &'a str,
    pub mines: &'a str,
    pub recipes: &'a str,
    pub migrations: &'a str,
}

/// The rules shipped with the game, embedded at build time.
pub fn shipped_files() -> RulesFiles<'static> {
    RulesFiles {
        items: include_str!("../../../assets/recipes/items.json"),
        resources: include_str!("../../../assets/recipes/resources.json"),
        mines: include_str!("../../../assets/recipes/mines.json"),
        recipes: include_str!("../../../assets/recipes/recipes.json"),
        migrations: include_str!("../../../assets/recipes/migrations.json"),
    }
}

/// The parsed, validated rules: the catalog's definition plus the migrations.
#[derive(Debug, Clone)]
pub struct Rules {
    pub definition: CatalogDefinition,
    pub migrations: Vec<Migration>,
}
impl Rules {
    /// Parse every file strictly and validate the set. Reports every problem found.
    pub fn load(files: &RulesFiles) -> Result<Self, RulesErrors> {
        let mut errors = Vec::new();
        let items = parse::<ItemsFile>(ITEMS, files.items, "items", &mut errors);
        let resources =
            parse::<ResourcesFile>(RESOURCES, files.resources, "resources", &mut errors);
        let mines = parse::<MinesFile>(MINES, files.mines, "mines", &mut errors);
        let recipes = parse::<RecipesFile>(RECIPES, files.recipes, "recipes", &mut errors);
        let migrations =
            parse::<MigrationsFile>(MIGRATIONS, files.migrations, "migrations", &mut errors);
        let (Some(items), Some(resources), Some(mines), Some(recipes), Some(migrations)) =
            (items, resources, mines, recipes, migrations)
        else {
            return Err(RulesErrors(errors));
        };
        for (file, format) in [
            (ITEMS, items.format),
            (RESOURCES, resources.format),
            (MINES, mines.format),
            (RECIPES, recipes.format),
            (MIGRATIONS, migrations.format),
        ] {
            if format != FORMAT {
                errors.push(RulesError::new(
                    file,
                    "",
                    "format",
                    format!("format {format}, but this build reads format {FORMAT}"),
                ));
            }
        }
        let definition = CatalogDefinition {
            items: items.items,
            resources: resources.resources,
            mines: mines.mines,
            recipes: recipes.recipes,
        };
        errors.extend(validate(&definition));
        errors.extend(validate_migrations(&definition, &migrations.migrations));
        if errors.is_empty() {
            Ok(Self {
                definition,
                migrations: migrations.migrations,
            })
        } else {
            Err(RulesErrors(errors))
        }
    }
    /// Every rule ID, in one namespace across all registries.
    pub fn ids(&self) -> BTreeSet<String> {
        ids(&self.definition)
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    }
}

/// Load the shipped rules into a catalog.
pub fn shipped_catalog() -> Result<crate::Catalog, RulesErrors> {
    let rules = Rules::load(&shipped_files())?;
    crate::Catalog::new(rules.definition)
        .map_err(|e| RulesErrors(vec![RulesError::new("", "", "", e.to_string())]))
}

/// Strict parse. A shape error names the row's ID (not just its index) and the field.
fn parse<T: DeserializeOwned>(
    file: &str,
    text: &str,
    domain: &str,
    errors: &mut Vec<RulesError>,
) -> Option<T> {
    let mut de = serde_json::Deserializer::from_str(text);
    match serde_path_to_error::deserialize::<_, T>(&mut de) {
        Ok(value) => Some(value),
        Err(error) => {
            let path = error.path().to_string();
            let (id, field) = locate(text, domain, &path);
            errors.push(RulesError::new(
                file,
                &id,
                &field,
                error.inner().to_string(),
            ));
            None
        }
    }
}

/// Turn a serde path such as `resources[1].yeilds` into the row's ID and the field.
fn locate(text: &str, domain: &str, path: &str) -> (String, String) {
    let Some(rest) = path.strip_prefix(&format!("{domain}[")) else {
        return (String::new(), path.trim_start_matches('.').into());
    };
    let Some((index, field)) = rest.split_once(']') else {
        return (String::new(), path.into());
    };
    let id = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| {
            let row = value
                .get(domain)?
                .get(index.parse::<usize>().ok()?)?
                .clone();
            row.get("id")?.as_str().map(str::to_owned)
        })
        .unwrap_or_else(|| format!("#{index}"));
    (id, field.trim_start_matches('.').into())
}

/// `snake_case`: a lowercase letter, then lowercase letters, digits and underscores.
pub fn valid_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    bytes.next().is_some_and(|c| c.is_ascii_lowercase())
        && bytes.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

fn ids(definition: &CatalogDefinition) -> Vec<(String, &'static str)> {
    let mut ids = Vec::new();
    ids.extend(definition.items.iter().map(|r| (r.id.clone(), ITEMS)));
    ids.extend(
        definition
            .resources
            .iter()
            .map(|r| (r.id.clone(), RESOURCES)),
    );
    ids.extend(definition.mines.iter().map(|r| (r.id.clone(), MINES)));
    ids.extend(definition.recipes.iter().map(|r| (r.id.clone(), RECIPES)));
    ids
}

/// Meaning checks over the whole set; every problem is collected.
pub fn validate(definition: &CatalogDefinition) -> Vec<RulesError> {
    let mut errors = Vec::new();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (id, file) in ids(definition) {
        if !valid_id(&id) {
            errors.push(RulesError::new(
                file,
                &id,
                "id",
                "IDs are snake_case: a lowercase letter, then a-z, 0-9 or _",
            ));
        }
        if let Some(first) = seen.insert(id.clone(), file) {
            errors.push(RulesError::new(
                file,
                &id,
                "id",
                format!("ID already used in {first}; IDs are unique across all rules files"),
            ));
        }
    }
    let items: BTreeSet<&str> = definition.items.iter().map(|i| i.id.as_str()).collect();
    let resources: BTreeSet<&str> = definition.resources.iter().map(|r| r.id.as_str()).collect();
    let stacks =
        |errors: &mut Vec<RulesError>, file: &str, id: &str, field: &str, list: &[Stack]| {
            if list.is_empty() {
                errors.push(RulesError::new(
                    file,
                    id,
                    field,
                    "must list at least one item",
                ));
            }
            let mut totals: BTreeMap<&str, u32> = BTreeMap::new();
            for stack in list {
                if stack.quantity == 0 {
                    errors.push(RulesError::new(
                        file,
                        id,
                        field,
                        format!("zero quantity of {}", stack.item),
                    ));
                }
                if !items.contains(stack.item.as_str()) {
                    errors.push(RulesError::new(
                        file,
                        id,
                        field,
                        format!("unknown item {}", stack.item),
                    ));
                }
                let total = totals.entry(&stack.item).or_insert(0);
                match total.checked_add(stack.quantity) {
                    Some(sum) => *total = sum,
                    None => errors.push(RulesError::new(
                        file,
                        id,
                        field,
                        format!("quantity overflow for {}", stack.item),
                    )),
                }
            }
        };
    for resource in &definition.resources {
        stacks(
            &mut errors,
            RESOURCES,
            &resource.id,
            "yields",
            &resource.yields,
        );
        if matches!(
            resource.replenishment,
            crate::ReplenishmentPolicy::After { seconds: 0 }
        ) {
            errors.push(RulesError::new(
                RESOURCES,
                &resource.id,
                "replenishment",
                "replenishment delay must be positive",
            ));
        }
    }
    for mine in &definition.mines {
        if !resources.contains(mine.produces_resource.as_str()) {
            errors.push(RulesError::new(
                MINES,
                &mine.id,
                "produces_resource",
                format!("unknown resource {}", mine.produces_resource),
            ));
        }
        for (index, factor) in mine.productivity_factors.iter().enumerate() {
            if mine.productivity_factors[..index].contains(factor) {
                errors.push(RulesError::new(
                    MINES,
                    &mine.id,
                    "productivity_factors",
                    format!("{factor:?} listed twice"),
                ));
            }
        }
    }
    for recipe in &definition.recipes {
        stacks(&mut errors, RECIPES, &recipe.id, "inputs", &recipe.inputs);
        stacks(&mut errors, RECIPES, &recipe.id, "outputs", &recipe.outputs);
    }
    // Reachability: an item is obtainable if a resource yields it or a recipe whose
    // inputs are all obtainable makes it. A recipe nothing can feed is an error.
    let mut obtainable: BTreeSet<&str> = definition
        .resources
        .iter()
        .flat_map(|r| r.yields.iter().map(|s| s.item.as_str()))
        .collect();
    let mut fed: BTreeSet<&str> = BTreeSet::new();
    loop {
        let before = fed.len();
        for recipe in &definition.recipes {
            if recipe
                .inputs
                .iter()
                .all(|s| obtainable.contains(s.item.as_str()))
            {
                fed.insert(&recipe.id);
                obtainable.extend(recipe.outputs.iter().map(|s| s.item.as_str()));
            }
        }
        if fed.len() == before {
            break;
        }
    }
    for recipe in &definition.recipes {
        if !fed.contains(recipe.id.as_str()) {
            let missing: Vec<_> = recipe
                .inputs
                .iter()
                .filter(|s| !obtainable.contains(s.item.as_str()))
                .map(|s| s.item.as_str())
                .collect();
            errors.push(RulesError::new(
                RECIPES,
                &recipe.id,
                "inputs",
                format!("unreachable: nothing produces {}", missing.join(", ")),
            ));
        }
    }
    errors
}

/// A migration's `from` must be gone and its `to` (if any) must exist.
pub fn validate_migrations(
    definition: &CatalogDefinition,
    migrations: &[Migration],
) -> Vec<RulesError> {
    let current: BTreeSet<(String, &str)> = ids(definition).into_iter().collect();
    let mut errors = Vec::new();
    let mut seen = BTreeSet::new();
    for migration in migrations {
        if !seen.insert(&migration.id) {
            errors.push(RulesError::new(
                MIGRATIONS,
                &migration.id,
                "id",
                "migration ID used twice",
            ));
        }
        let file = migration.registry.file();
        if current.contains(&(migration.from.clone(), file)) {
            errors.push(RulesError::new(
                MIGRATIONS,
                &migration.id,
                "from",
                format!("{} still exists in {file}", migration.from),
            ));
        }
        if let Some(to) = &migration.to
            && !current.contains(&(to.clone(), file))
        {
            errors.push(RulesError::new(
                MIGRATIONS,
                &migration.id,
                "to",
                format!("{to} does not exist in {file}"),
            ));
        }
    }
    errors
}

/// `ids.lock` lists every rule ID ever shipped. A listed ID that is gone needs a
/// migration naming it; a new ID must be added (by `rules-fmt`).
pub fn check_ids_lock(lock: &str, rules: &Rules) -> Vec<RulesError> {
    let locked: BTreeSet<&str> = lock_ids(lock).collect();
    let current = rules.ids();
    let migrated: BTreeSet<&str> = rules.migrations.iter().map(|m| m.from.as_str()).collect();
    let mut errors = Vec::new();
    for id in &locked {
        if !current.contains(*id) && !migrated.contains(id) {
            errors.push(RulesError::new(
                IDS_LOCK,
                id,
                "",
                "ID was shipped but is gone, and no migration names it; add one to migrations.json",
            ));
        }
    }
    for id in &current {
        if !locked.contains(id.as_str()) {
            errors.push(RulesError::new(
                IDS_LOCK,
                id,
                "",
                "new ID not in ids.lock; run `cargo run -p stubby_recipes --bin rules-fmt`",
            ));
        }
    }
    errors
}

fn lock_ids(lock: &str) -> impl Iterator<Item = &str> {
    lock.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// `ids.lock` with the current IDs added. IDs are never removed from it.
pub fn updated_ids_lock(lock: &str, rules: &Rules) -> String {
    let mut all: BTreeSet<String> = lock_ids(lock).map(str::to_owned).collect();
    all.extend(rules.ids());
    let mut out = String::from(
        "# Every rule ID ever shipped. Maintained by `cargo run -p stubby_recipes --bin rules-fmt`.\n\
         # Never remove a line: a renamed or removed ID needs a row in migrations.json.\n",
    );
    for id in all {
        out.push_str(&id);
        out.push('\n');
    }
    out
}

fn schema_ref(file: &str) -> Option<String> {
    Some(format!("schema/{}", file.replace(".json", ".schema.json")))
}

fn pretty<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    text
}

/// The canonical text of every rules file: rows sorted by `id`, fields in schema
/// order, defaults omitted, pretty-printed with a trailing newline. Fails only when a
/// file does not parse.
pub fn canonical_files(files: &RulesFiles) -> Result<Vec<(&'static str, String)>, RulesErrors> {
    let mut errors = Vec::new();
    let items = parse::<ItemsFile>(ITEMS, files.items, "items", &mut errors);
    let resources = parse::<ResourcesFile>(RESOURCES, files.resources, "resources", &mut errors);
    let mines = parse::<MinesFile>(MINES, files.mines, "mines", &mut errors);
    let recipes = parse::<RecipesFile>(RECIPES, files.recipes, "recipes", &mut errors);
    let migrations =
        parse::<MigrationsFile>(MIGRATIONS, files.migrations, "migrations", &mut errors);
    let (
        Some(mut items),
        Some(mut resources),
        Some(mut mines),
        Some(mut recipes),
        Some(mut migrations),
    ) = (items, resources, mines, recipes, migrations)
    else {
        return Err(RulesErrors(errors));
    };
    items.schema = schema_ref(ITEMS);
    items.items.sort_by(|a, b| a.id.cmp(&b.id));
    resources.schema = schema_ref(RESOURCES);
    resources.resources.sort_by(|a, b| a.id.cmp(&b.id));
    mines.schema = schema_ref(MINES);
    mines.mines.sort_by(|a, b| a.id.cmp(&b.id));
    recipes.schema = schema_ref(RECIPES);
    recipes.recipes.sort_by(|a, b| a.id.cmp(&b.id));
    migrations.schema = schema_ref(MIGRATIONS);
    migrations.migrations.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(vec![
        (ITEMS, pretty(&items)),
        (RESOURCES, pretty(&resources)),
        (MINES, pretty(&mines)),
        (RECIPES, pretty(&recipes)),
        (MIGRATIONS, pretty(&migrations)),
    ])
}

/// The JSON Schema of every rules file, as written under `assets/recipes/schema/`.
pub fn schemas() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (file, schema) in [
        (ITEMS, schemars::schema_for!(ItemsFile)),
        (RESOURCES, schemars::schema_for!(ResourcesFile)),
        (MINES, schemars::schema_for!(MinesFile)),
        (RECIPES, schemars::schema_for!(RecipesFile)),
        (MIGRATIONS, schemars::schema_for!(MigrationsFile)),
    ] {
        out.push((schema_ref(file).unwrap_or_default(), pretty(&schema)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/recipes")
    }

    fn with<'a>(field: &str, text: &'a str) -> RulesFiles<'a> {
        let mut files = shipped_files();
        match field {
            "items" => files.items = text,
            "resources" => files.resources = text,
            "mines" => files.mines = text,
            "recipes" => files.recipes = text,
            _ => files.migrations = text,
        }
        files
    }

    #[test]
    fn shipped_rules_load_with_no_errors() {
        if let Err(errors) = Rules::load(&shipped_files()) {
            panic!("{errors}");
        }
        shipped_catalog().unwrap();
    }

    #[test]
    fn shipped_files_are_canonical() {
        for (file, canonical) in canonical_files(&shipped_files()).unwrap() {
            let on_disk = std::fs::read_to_string(shipped_dir().join(file)).unwrap();
            assert!(
                on_disk == canonical,
                "{file} is not in canonical form; run `cargo run -p stubby_recipes --bin rules-fmt`"
            );
        }
    }

    #[test]
    fn schemas_are_current() {
        for (path, schema) in schemas() {
            let on_disk = std::fs::read_to_string(shipped_dir().join(&path)).unwrap_or_default();
            assert!(
                on_disk == schema,
                "{path} is stale; run `cargo run -p stubby_recipes --bin rules-fmt`"
            );
        }
    }

    #[test]
    fn ids_lock_covers_every_shipped_id() {
        let rules = Rules::load(&shipped_files()).unwrap();
        let lock = std::fs::read_to_string(shipped_dir().join(IDS_LOCK)).unwrap();
        let errors = check_ids_lock(&lock, &rules);
        assert!(errors.is_empty(), "{}", RulesErrors(errors));
    }

    #[test]
    fn misspelled_field_names_file_row_and_field() {
        let text = shipped_files()
            .resources
            .replace("\"yields\"", "\"yeilds\"");
        let errors = Rules::load(&with("resources", &text)).unwrap_err().0;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, RESOURCES);
        assert_eq!(errors[0].id, "clock_source");
        assert!(errors[0].message.contains("yeilds"), "{}", errors[0]);
    }

    #[test]
    fn ids_are_snake_case_and_unique_across_files() {
        let text = shipped_files()
            .items
            .replace("\"square_wave\"", "\"Square-Wave\"");
        let errors = Rules::load(&with("items", &text)).unwrap_err().0;
        assert!(
            errors
                .iter()
                .any(|e| e.id == "Square-Wave" && e.field == "id")
        );

        let text = shipped_files()
            .resources
            .replace("\"clock_source\"", "\"clock\"");
        let errors = Rules::load(&with("resources", &text)).unwrap_err().0;
        assert!(
            errors
                .iter()
                .any(|e| e.id == "clock" && e.message.contains(ITEMS)),
            "{}",
            RulesErrors(errors)
        );
    }
}
