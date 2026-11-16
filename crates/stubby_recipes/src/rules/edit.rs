//! Editing the rules as a model, not as files (add-crafting-designer, `rules-editing`).
//!
//! A [`RulesDocument`] holds every rules file as JSON, applies typed [`Edit`]s, keeps
//! undo and redo, validates the whole set after every edit exactly as loading does, and
//! saves through the same canonical writer and `ids.lock` update as `rules-fmt`. Every
//! front end (the Jackdaw designer, a later in-game one, tests) edits rules through it.
//!
//! Which fields hold IDs comes from the generated JSON Schema's `x-stubby-ref` marks, so
//! renames and where-used follow the schema code without a second list here.
use super::{
    IDS_LOCK, MIGRATIONS, Registry, Rules, RulesError, RulesFiles, canonical_files, check_ids_lock,
    schemas, updated_ids_lock, valid_id,
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// Every rules file the document holds, in `RulesFiles` order.
use super::FILES;

impl Registry {
    /// The array in the registry's file that holds its rows.
    pub fn rows_key(self) -> &'static str {
        match self {
            Self::Items => "items",
            Self::Resources => "resources",
            Self::Mines => "mines",
            Self::Recipes => "recipes",
            Self::Buildables => "buildables",
            Self::Goals => "goals",
            Self::Air => "air",
        }
    }
    pub const ALL: [Registry; 7] = [
        Self::Items,
        Self::Resources,
        Self::Mines,
        Self::Recipes,
        Self::Buildables,
        Self::Goals,
        Self::Air,
    ];
}

/// One change to the rules. Serialised with a `kind` tag (`{"kind": "set", ...}`), so a
/// front end such as the web designer sends edits as JSON.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    /// Add a row (it must carry a new, valid `id`), at `at` or at the end.
    Add {
        registry: Registry,
        row: Value,
        #[serde(default)]
        at: Option<usize>,
    },
    /// Set (or, with `None`, remove) a value inside a row, by JSON pointer from the row
    /// (`/yields/0/quantity`). An array index one past the end appends.
    Set {
        registry: Registry,
        id: String,
        pointer: String,
        value: Option<Value>,
    },
    /// Remove a row. An ID that has shipped (`ids.lock`) needs a migration: `to` names
    /// what replaces it, or `retire` records that it is gone for good.
    Remove {
        registry: Registry,
        id: String,
        #[serde(default)]
        to: Option<String>,
        #[serde(default)]
        retire: bool,
    },
    /// Rename a row's ID, every reference to it, and record the migration if it shipped.
    Rename {
        registry: Registry,
        from: String,
        to: String,
    },
}

/// An edit that could not be applied at all. Edits that apply but leave the rules
/// invalid are kept and reported by [`RulesDocument::problems`] instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    NoSuchRow {
        registry: Registry,
        id: String,
    },
    BadId(String),
    IdTaken(String),
    BadPointer(String),
    /// The ID has shipped, so removing it needs `to` or `retire`.
    NeedsMigration(String),
    /// Saving would overwrite a file changed on disk since the document loaded it.
    ChangedOnDisk(String),
    /// Saving was refused because the rules are invalid.
    Invalid(Vec<RulesError>),
    Io(String),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchRow { registry, id } => write!(f, "no {id} in {}", registry.file()),
            Self::BadId(id) => write!(f, "`{id}` is not a valid ID ([a-z0-9_])"),
            Self::IdTaken(id) => write!(f, "`{id}` is already used; IDs share one namespace"),
            Self::BadPointer(p) => write!(f, "`{p}` does not point into the row"),
            Self::NeedsMigration(id) => write!(
                f,
                "{id} has shipped; removing it needs a replacement or an explicit retirement"
            ),
            Self::ChangedOnDisk(file) => {
                write!(
                    f,
                    "{file} changed on disk since it was loaded; reload first"
                )
            }
            Self::Invalid(errors) => write!(f, "the rules have {} problem(s)", errors.len()),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for EditError {}

/// Where an ID is used: the file, the row that uses it and the field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Use {
    pub file: &'static str,
    pub row: String,
    pub field: String,
}

type Snapshot = BTreeMap<&'static str, Value>;

/// The rules being edited.
pub struct RulesDocument {
    values: Snapshot,
    /// The values as last loaded or saved, for `is_dirty`.
    saved: Snapshot,
    /// Each file's text as loaded or last saved, to refuse overwriting a newer file.
    on_disk: BTreeMap<&'static str, String>,
    lock: String,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    problems: Vec<RulesError>,
    /// Property name -> the registry it references, per file (from `x-stubby-ref`).
    refs: BTreeMap<&'static str, BTreeMap<String, Registry>>,
}

impl RulesDocument {
    /// Load the rules directory (`assets/recipes`).
    pub fn open(dir: &Path) -> Result<Self, EditError> {
        let mut texts = BTreeMap::new();
        for file in FILES {
            let path = dir.join(file);
            let text = std::fs::read_to_string(&path)
                .map_err(|e| EditError::Io(format!("{}: {e}", path.display())))?;
            texts.insert(file, text);
        }
        let lock = std::fs::read_to_string(dir.join(IDS_LOCK)).unwrap_or_default();
        Self::from_texts(texts, lock)
    }

    /// A document over file texts (tests, embedded rules).
    pub fn from_texts(
        texts: BTreeMap<&'static str, String>,
        lock: String,
    ) -> Result<Self, EditError> {
        let mut values = BTreeMap::new();
        for file in FILES {
            let text = texts
                .get(file)
                .ok_or_else(|| EditError::Io(format!("{file} missing")))?;
            let value =
                serde_json::from_str(text).map_err(|e| EditError::Io(format!("{file}: {e}")))?;
            values.insert(file, value);
        }
        let mut document = Self {
            saved: values.clone(),
            values,
            on_disk: texts,
            lock,
            undo: Vec::new(),
            redo: Vec::new(),
            problems: Vec::new(),
            refs: reference_fields(),
        };
        document.validate();
        Ok(document)
    }

    /// Every problem in the rules as they stand, as loading would report them.
    pub fn problems(&self) -> &[RulesError] {
        &self.problems
    }

    /// Whether anything has changed since loading or saving.
    pub fn is_dirty(&self) -> bool {
        self.values != self.saved
    }

    /// The rows of a registry, as JSON.
    pub fn rows(&self, registry: Registry) -> &[Value] {
        self.values[registry.file()][registry.rows_key()]
            .as_array()
            .map_or(&[], Vec::as_slice)
    }

    /// One row, by ID.
    pub fn row(&self, registry: Registry, id: &str) -> Option<&Value> {
        self.rows(registry).iter().find(|r| r["id"] == id)
    }

    /// Apply an edit. On success it can be undone; a structural failure changes nothing.
    pub fn apply(&mut self, edit: Edit) -> Result<(), EditError> {
        let before = self.values.clone();
        if let Err(e) = self.change(edit) {
            self.values = before;
            return Err(e);
        }
        self.undo.push(before);
        self.redo.clear();
        self.validate();
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(before) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.values, before));
        self.validate();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(after) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.values, after));
        self.validate();
        true
    }

    /// Every place an ID is referenced, across all rules files.
    /// The JSON Schema of one row of `registry`, for building a form: the row's schema
    /// from the generated file schema, its `$defs`, and every `x-stubby-ref` field given
    /// an `enum` of the IDs its registry holds now (so a form offers only real IDs).
    pub fn row_schema(&self, registry: Registry) -> Value {
        let wanted = super::schema_ref(registry.file()).unwrap_or_default();
        let file: Value = schemas()
            .into_iter()
            .find(|(path, _)| *path == wanted)
            .and_then(|(_, text)| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null);
        let mut row = file["properties"][registry.rows_key()]["items"].clone();
        // The row is usually a `$ref` to its definition: inline it, so the root of the
        // schema is the row itself.
        let target = row
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|r| r.strip_prefix("#/$defs/"))
            .and_then(|name| file["$defs"].get(name))
            .cloned();
        if let Some(target) = target {
            row = target;
        }
        if let (Some(row), Some(defs)) = (row.as_object_mut(), file.get("$defs")) {
            row.insert("$defs".into(), defs.clone());
        }
        self.offer_ids(&mut row);
        row
    }

    fn offer_ids(&self, schema: &mut Value) {
        match schema {
            Value::Object(map) => {
                let registry = map
                    .get("x-stubby-ref")
                    .and_then(|r| serde_json::from_value::<Registry>(r.clone()).ok());
                if let Some(registry) = registry {
                    let mut ids: Vec<Value> = self
                        .rows(registry)
                        .iter()
                        .filter_map(|row| row.get("id").cloned())
                        .collect();
                    let nullable = map
                        .get("type")
                        .and_then(Value::as_array)
                        .is_some_and(|types| types.iter().any(|t| t == "null"));
                    if nullable {
                        ids.push(Value::Null);
                    }
                    map.insert("enum".into(), Value::Array(ids));
                }
                for child in map.values_mut() {
                    self.offer_ids(child);
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|item| self.offer_ids(item)),
            _ => {}
        }
    }

    pub fn where_used(&self, id: &str) -> Vec<Use> {
        let mut uses = BTreeSet::new();
        for file in FILES {
            let props = &self.refs[file];
            visit(&self.values[file], props, &mut |row, field, value| {
                if value == id {
                    uses.insert(Use {
                        file,
                        row: row.to_owned(),
                        field: field.to_owned(),
                    });
                }
            });
        }
        uses.into_iter().collect()
    }

    /// Save the changed files in canonical form and add new IDs to `ids.lock`. Refuses
    /// while the rules are invalid, or when a file changed on disk since it was loaded.
    pub fn save(&mut self, dir: &Path) -> Result<Vec<&'static str>, EditError> {
        if !self.problems.is_empty() {
            return Err(EditError::Invalid(self.problems.clone()));
        }
        for file in FILES {
            let disk = std::fs::read_to_string(dir.join(file)).ok();
            if disk.as_ref() != self.on_disk.get(file) {
                return Err(EditError::ChangedOnDisk(file.to_owned()));
            }
        }
        let (outputs, lock) = self.canonical()?;
        let mut written = Vec::new();
        for (file, text) in outputs {
            if self.on_disk.get(file) != Some(&text) {
                std::fs::write(dir.join(file), &text).map_err(|e| EditError::Io(e.to_string()))?;
                written.push(file);
            }
            self.on_disk.insert(file, text.clone());
            self.values.insert(
                file,
                serde_json::from_str(&text).map_err(|e| EditError::Io(e.to_string()))?,
            );
        }
        self.saved = self.values.clone();
        if lock != self.lock {
            std::fs::write(dir.join(IDS_LOCK), &lock).map_err(|e| EditError::Io(e.to_string()))?;
            self.lock = lock;
            written.push(IDS_LOCK);
        }
        Ok(written)
    }

    /// The files and lock as `save` would write them.
    pub fn canonical(&self) -> Result<(Vec<(&'static str, String)>, String), EditError> {
        let texts: Vec<String> = FILES.iter().map(|f| self.text(f)).collect();
        let files = files_of(&texts);
        let outputs = canonical_files(&files).map_err(|e| EditError::Invalid(e.0))?;
        let rules = Rules::load(&files).map_err(|e| EditError::Invalid(e.0))?;
        Ok((outputs, updated_ids_lock(&self.lock, &rules)))
    }

    // -------------------------------------------------------------- internals

    fn text(&self, file: &str) -> String {
        let mut text = serde_json::to_string_pretty(&self.values[file]).unwrap_or_default();
        text.push('\n');
        text
    }

    fn validate(&mut self) {
        let texts: Vec<String> = FILES.iter().map(|f| self.text(f)).collect();
        let files = files_of(&texts);
        self.problems = match Rules::load(&files) {
            // A removed or renamed shipped ID needs its migration; new IDs are fine
            // here (saving adds them to the lock).
            Ok(rules) => check_ids_lock(&self.lock, &rules)
                .into_iter()
                .filter(|e| !e.message.starts_with("new ID"))
                .collect(),
            Err(errors) => errors.0,
        };
    }

    fn all_ids(&self) -> BTreeSet<String> {
        Registry::ALL
            .iter()
            .flat_map(|r| {
                self.rows(*r)
                    .iter()
                    .filter_map(|row| row["id"].as_str().map(str::to_owned))
            })
            .collect()
    }

    fn shipped(&self, id: &str) -> bool {
        self.lock.lines().map(str::trim).any(|l| l == id)
    }

    fn rows_mut(&mut self, registry: Registry) -> Result<&mut Vec<Value>, EditError> {
        self.values
            .get_mut(registry.file())
            .and_then(|v| v.get_mut(registry.rows_key()))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| EditError::Io(format!("{} has no rows", registry.file())))
    }

    fn index(&self, registry: Registry, id: &str) -> Result<usize, EditError> {
        self.rows(registry)
            .iter()
            .position(|r| r["id"] == id)
            .ok_or_else(|| EditError::NoSuchRow {
                registry,
                id: id.to_owned(),
            })
    }

    fn new_id(&self, id: &str) -> Result<(), EditError> {
        if !valid_id(id) {
            return Err(EditError::BadId(id.to_owned()));
        }
        if self.all_ids().contains(id) {
            return Err(EditError::IdTaken(id.to_owned()));
        }
        Ok(())
    }

    fn migrate(
        &mut self,
        registry: Registry,
        from: &str,
        to: Option<&str>,
    ) -> Result<(), EditError> {
        let id = match to {
            Some(to) => format!("{from}_to_{to}"),
            None => format!("retire_{from}"),
        };
        let mut row = Map::new();
        row.insert("id".into(), id.into());
        row.insert(
            "registry".into(),
            serde_json::to_value(registry).map_err(|e| EditError::Io(e.to_string()))?,
        );
        row.insert("from".into(), from.into());
        row.insert("to".into(), to.map_or(Value::Null, Into::into));
        self.values
            .get_mut(MIGRATIONS)
            .and_then(|v| v.get_mut("migrations"))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| EditError::Io("migrations.json has no rows".into()))?
            .push(Value::Object(row));
        Ok(())
    }

    fn change(&mut self, edit: Edit) -> Result<(), EditError> {
        match edit {
            Edit::Add { registry, row, at } => {
                let id = row["id"].as_str().unwrap_or_default().to_owned();
                self.new_id(&id)?;
                let rows = self.rows_mut(registry)?;
                let at = at.unwrap_or(rows.len()).min(rows.len());
                rows.insert(at, row);
            }
            Edit::Set {
                registry,
                id,
                pointer,
                value,
            } => {
                let i = self.index(registry, &id)?;
                let row = &mut self.rows_mut(registry)?[i];
                set(row, &pointer, value).ok_or(EditError::BadPointer(pointer))?;
            }
            Edit::Remove {
                registry,
                id,
                to,
                retire,
            } => {
                let i = self.index(registry, &id)?;
                if self.shipped(&id) {
                    if to.is_none() && !retire {
                        return Err(EditError::NeedsMigration(id));
                    }
                    self.migrate(registry, &id, to.as_deref())?;
                }
                self.rows_mut(registry)?.remove(i);
            }
            Edit::Rename { registry, from, to } => {
                let i = self.index(registry, &from)?;
                self.new_id(&to)?;
                self.rows_mut(registry)?[i]["id"] = to.clone().into();
                for file in FILES {
                    let props = self.refs[file].clone();
                    if let Some(value) = self.values.get_mut(file) {
                        rename_refs(value, &props, registry, &from, &to);
                    }
                }
                if self.shipped(&from) {
                    self.migrate(registry, &from, Some(&to))?;
                }
            }
        }
        Ok(())
    }
}

fn files_of(texts: &[String]) -> RulesFiles<'_> {
    RulesFiles {
        items: &texts[0],
        resources: &texts[1],
        mines: &texts[2],
        recipes: &texts[3],
        buildables: &texts[4],
        migrations: &texts[5],
        goals: &texts[6],
        start: &texts[7],
        air: &texts[8],
    }
}

/// Set or remove the value at `pointer` inside `root`. `None` when the parent is missing.
fn set(root: &mut Value, pointer: &str, value: Option<Value>) -> Option<()> {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return None;
    }
    let (parent, last) = pointer.rsplit_once('/')?;
    let last = last.replace("~1", "/").replace("~0", "~");
    match root.pointer_mut(parent)? {
        Value::Object(map) => {
            if last == "id" {
                return None; // IDs change only by Rename.
            }
            match value {
                Some(v) => {
                    map.insert(last, v);
                }
                None => {
                    map.remove(&last)?;
                }
            }
        }
        Value::Array(items) => {
            let i: usize = last.parse().ok()?;
            match value {
                Some(v) if i < items.len() => items[i] = v,
                Some(v) if i == items.len() => items.push(v),
                None if i < items.len() => {
                    items.remove(i);
                }
                _ => return None,
            }
        }
        _ => return None,
    }
    Some(())
}

/// Call `f(row id, field, value)` for every ID-reference value under `value`.
fn visit(value: &Value, props: &BTreeMap<String, Registry>, f: &mut impl FnMut(&str, &str, &str)) {
    let rows = value
        .as_object()
        .into_iter()
        .flat_map(|o| o.values())
        .filter_map(Value::as_array);
    for row in rows.flatten() {
        let id = row["id"].as_str().unwrap_or_default();
        walk(row, None, props, &mut |field, v| f(id, field, v));
    }
}

fn walk(
    value: &Value,
    key: Option<&str>,
    props: &BTreeMap<String, Registry>,
    f: &mut impl FnMut(&str, &str),
) {
    match value {
        Value::String(s) => {
            if let Some(key) = key.filter(|k| props.contains_key(*k)) {
                f(key, s);
            }
        }
        Value::Array(items) => items.iter().for_each(|v| walk(v, key, props, f)),
        Value::Object(map) => map.iter().for_each(|(k, v)| walk(v, Some(k), props, f)),
        _ => {}
    }
}

fn rename_refs(
    value: &mut Value,
    props: &BTreeMap<String, Registry>,
    registry: Registry,
    from: &str,
    to: &str,
) {
    fn go(
        value: &mut Value,
        key: Option<&str>,
        props: &BTreeMap<String, Registry>,
        registry: Registry,
        from: &str,
        to: &str,
    ) {
        match value {
            Value::String(s) => {
                if key.and_then(|k| props.get(k)) == Some(&registry) && s == from {
                    *s = to.to_owned();
                }
            }
            Value::Array(items) => items
                .iter_mut()
                .for_each(|v| go(v, key, props, registry, from, to)),
            Value::Object(map) => {
                for (k, v) in map.iter_mut() {
                    // A migration's own `from` names the old ID on purpose.
                    if k != "from" {
                        go(v, Some(k), props, registry, from, to);
                    }
                }
            }
            _ => {}
        }
    }
    go(value, None, props, registry, from, to);
}

/// For each rules file, which property names hold IDs of which registry, read from the
/// `x-stubby-ref` marks in the generated schemas.
pub fn reference_fields() -> BTreeMap<&'static str, BTreeMap<String, Registry>> {
    let schemas: BTreeMap<String, String> = schemas().into_iter().collect();
    FILES
        .iter()
        .map(|file| {
            let path = format!("schema/{}", file.replace(".json", ".schema.json"));
            let mut props = BTreeMap::new();
            if let Some(schema) = schemas
                .get(&path)
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
            {
                collect_refs(&schema, &mut props);
            }
            (*file, props)
        })
        .collect()
}

fn collect_refs(schema: &Value, out: &mut BTreeMap<String, Registry>) {
    match schema {
        Value::Object(map) => {
            if let Some(Value::Object(properties)) = map.get("properties") {
                for (name, property) in properties {
                    if let Some(registry) = property
                        .get("x-stubby-ref")
                        .and_then(|r| serde_json::from_value::<Registry>(r.clone()).ok())
                    {
                        out.insert(name.clone(), registry);
                    }
                }
            }
            map.values().for_each(|v| collect_refs(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| collect_refs(v, out)),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
