//! The rules designer's server (add-crafting-designer, `crafting-designer`).
//!
//! One [`RulesDocument`] (the Bevy-free editing model in `stubby_recipes`) behind a small
//! JSON API, plus the page that edits it. [`Designer::handle`] is the whole server as a
//! function of method, path and body, so it is tested without sockets; `main.rs` only
//! connects it to `tiny_http` on 127.0.0.1.
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
mod lore;

use stubby_recipes::rules::{
    FILES, Registry,
    edit::{Edit, EditError, RulesDocument},
};

const INDEX: &str = include_str!("../web/index.html");
const JSON_EDITOR: &str = include_str!("../web/vendor/jsoneditor.min.js");

/// A response: status, content type and body.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}
impl Reply {
    fn json(status: u16, value: &Value) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: value.to_string().into(),
        }
    }
    fn error(status: u16, message: impl std::fmt::Display) -> Self {
        Self::json(status, &json!({ "error": message.to_string() }))
    }
}

/// The rules being designed, and when their files last changed on disk.
pub struct Designer {
    dir: PathBuf,
    /// The repository root, for the lore guide (`/lore`).
    root: PathBuf,
    doc: RulesDocument,
    seen: Option<SystemTime>,
}

/// A whole row from a form: the designer turns it into the model's edits.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowUpdate {
    registry: Registry,
    id: String,
    row: Value,
}

impl Designer {
    pub fn open(dir: &Path) -> Result<Self, EditError> {
        Ok(Self {
            dir: dir.to_owned(),
            root: dir.join("../.."),
            doc: RulesDocument::open(dir)?,
            seen: newest(dir),
        })
    }

    /// The repository the lore guide reads (default: two folders above the rules).
    pub fn with_root(mut self, root: &Path) -> Self {
        self.root = root.to_owned();
        self
    }

    /// Answer one request. `path` may carry a query (`/api/rows?registry=items`).
    pub fn handle(&mut self, method: &str, path: &str, body: &str) -> Reply {
        self.reload_if_changed();
        let (route, query) = path.split_once('?').unwrap_or((path, ""));
        let param = |name: &str| {
            query
                .split('&')
                .filter_map(|pair| pair.split_once('='))
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_owned())
        };
        match (method, route) {
            ("GET", "/") => Reply {
                status: 200,
                content_type: "text/html; charset=utf-8",
                body: INDEX.into(),
            },
            ("GET", "/jsoneditor.min.js") => Reply {
                status: 200,
                content_type: "text/javascript; charset=utf-8",
                body: JSON_EDITOR.into(),
            },
            ("GET", "/lore") => Reply {
                status: 200,
                content_type: "text/html; charset=utf-8",
                body: lore::PAGE.into(),
            },
            ("GET", "/marked.min.js") => Reply {
                status: 200,
                content_type: "text/javascript; charset=utf-8",
                body: lore::MARKED.into(),
            },
            ("GET", "/api/lore") => lore::index(&self.root),
            ("GET", "/lore/file") => match param("path") {
                Some(path) => lore::file(&self.root, &lore::decode(&path)),
                None => Reply::error(400, "missing ?path="),
            },
            ("GET", "/api/state") => Reply::json(200, &self.state()),
            ("GET", "/api/rows") => match param("registry").as_deref().map(registry) {
                Some(Ok(registry)) => Reply::json(
                    200,
                    &json!({
                        "rows": self.doc.rows(registry),
                        "schema": for_json_editor(self.doc.row_schema(registry)),
                    }),
                ),
                Some(Err(error)) => Reply::error(400, error),
                None => Reply::error(400, "missing ?registry="),
            },
            ("GET", "/api/where-used") => match param("id") {
                Some(id) => Reply::json(200, &json!(self.doc.where_used(&id))),
                None => Reply::error(400, "missing ?id="),
            },
            ("POST", "/api/edit") => match serde_json::from_str::<Edit>(body) {
                Ok(edit) => self.apply([edit]),
                Err(error) => Reply::error(400, error),
            },
            ("POST", "/api/row") => match serde_json::from_str::<RowUpdate>(body) {
                Ok(update) => match self.row_edits(&update) {
                    Ok(edits) => self.apply(edits),
                    Err(error) => Reply::error(400, error),
                },
                Err(error) => Reply::error(400, error),
            },
            ("POST", "/api/undo") => {
                self.doc.undo();
                Reply::json(200, &self.state())
            }
            ("POST", "/api/redo") => {
                self.doc.redo();
                Reply::json(200, &self.state())
            }
            ("POST", "/api/save") => match self.doc.save(&self.dir) {
                Ok(written) => {
                    self.seen = newest(&self.dir);
                    let mut state = self.state();
                    state["written"] = json!(written);
                    Reply::json(200, &state)
                }
                Err(error) => Reply::error(409, error),
            },
            ("POST", "/api/reload") => match RulesDocument::open(&self.dir) {
                Ok(doc) => {
                    self.doc = doc;
                    self.seen = newest(&self.dir);
                    Reply::json(200, &self.state())
                }
                Err(error) => Reply::error(409, error),
            },
            _ => Reply::error(404, format!("no {method} {route}")),
        }
    }

    /// Problems, whether there is anything to save, undo or redo, and every registry
    /// with its row count.
    fn state(&self) -> Value {
        let registries: Vec<Value> = Registry::ALL
            .iter()
            .map(|registry| {
                json!({
                    "registry": registry,
                    "file": registry.file(),
                    "count": self.doc.rows(*registry).len(),
                })
            })
            .collect();
        json!({
            "registries": registries,
            "problems": self.doc.problems(),
            "dirty": self.doc.is_dirty(),
            "can_undo": self.doc.can_undo(),
            "can_redo": self.doc.can_redo(),
        })
    }

    /// Apply edits in order, stopping at the first that cannot apply. Edits that apply
    /// but leave the rules invalid are kept and show up as problems.
    fn apply(&mut self, edits: impl IntoIterator<Item = Edit>) -> Reply {
        for edit in edits {
            if let Err(error) = self.doc.apply(edit) {
                let mut state = self.state();
                state["error"] = json!(error.to_string());
                return Reply::json(422, &state);
            }
        }
        Reply::json(200, &self.state())
    }

    /// The model's edits that turn a row into the form's version of it: a rename if the
    /// ID changed, then one `Set` per changed top-level field (absent fields are removed).
    fn row_edits(&self, update: &RowUpdate) -> Result<Vec<Edit>, String> {
        let old = self
            .doc
            .row(update.registry, &update.id)
            .ok_or_else(|| format!("no {} in {}", update.id, update.registry.file()))?;
        let (Some(old), Some(new)) = (old.as_object(), update.row.as_object()) else {
            return Err("a row is a JSON object".into());
        };
        let mut edits = Vec::new();
        let id = match new.get("id").and_then(Value::as_str) {
            Some(id) if id != update.id => {
                edits.push(Edit::Rename {
                    registry: update.registry,
                    from: update.id.clone(),
                    to: id.to_owned(),
                });
                id.to_owned()
            }
            _ => update.id.clone(),
        };
        let mut keys: Vec<&String> = old.keys().chain(new.keys()).collect();
        keys.sort();
        keys.dedup();
        for key in keys.into_iter().filter(|key| *key != "id") {
            if old.get(key) != new.get(key) {
                edits.push(Edit::Set {
                    registry: update.registry,
                    id: id.clone(),
                    pointer: format!("/{}", key.replace('~', "~0").replace('/', "~1")),
                    value: new.get(key).cloned(),
                });
            }
        }
        Ok(edits)
    }

    /// Pick up files changed on disk (a hand edit, `rules-fmt`, an agent) while nothing is
    /// unsaved here. With unsaved edits the model refuses to save over newer files.
    fn reload_if_changed(&mut self) {
        let now = newest(&self.dir);
        if now == self.seen || self.doc.is_dirty() {
            return;
        }
        if let Ok(doc) = RulesDocument::open(&self.dir) {
            self.doc = doc;
            self.seen = now;
        }
    }
}

fn registry(name: &str) -> Result<Registry, String> {
    serde_json::from_value(json!(name)).map_err(|_| format!("no registry `{name}`"))
}

/// The newest modification time among the rules files.
fn newest(dir: &Path) -> Option<SystemTime> {
    FILES
        .iter()
        .filter_map(|file| std::fs::metadata(dir.join(file)).ok()?.modified().ok())
        .max()
}

/// `json-editor` resolves `#/definitions/…`, not the `#/$defs/…` schemars writes: move
/// the definitions and rewrite every reference.
pub fn for_json_editor(mut schema: Value) -> Value {
    fn rewrite(value: &mut Value) {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(target)) = map.get_mut("$ref")
                    && let Some(name) = target.strip_prefix("#/$defs/")
                {
                    *target = format!("#/definitions/{name}");
                }
                map.values_mut().for_each(rewrite);
            }
            Value::Array(items) => items.iter_mut().for_each(rewrite),
            _ => {}
        }
    }
    if let Some(map) = schema.as_object_mut()
        && let Some(defs) = map.remove("$defs")
    {
        map.insert("definitions".into(), defs);
    }
    rewrite(&mut schema);
    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch copy of the shipped rules, so tests can save.
    fn scratch() -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/recipes");
        let dir = std::env::temp_dir().join(format!(
            "stubby-rules-server-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(dir.join("schema")).unwrap();
        for entry in std::fs::read_dir(&shipped).unwrap().flatten() {
            if entry.path().is_file() {
                std::fs::copy(entry.path(), dir.join(entry.file_name())).unwrap();
            }
        }
        for entry in std::fs::read_dir(shipped.join("schema")).unwrap().flatten() {
            std::fs::copy(entry.path(), dir.join("schema").join(entry.file_name())).unwrap();
        }
        dir
    }
    fn body(reply: &Reply) -> Value {
        serde_json::from_slice(&reply.body).unwrap()
    }

    #[test]
    fn serves_the_page_rows_and_form_schemas() {
        let mut designer = Designer::open(&scratch()).unwrap();
        assert!(String::from_utf8_lossy(&designer.handle("GET", "/", "").body).contains("<html"));
        let reply = designer.handle("GET", "/api/rows?registry=air", "");
        assert_eq!(reply.status, 200);
        let rows = body(&reply);
        assert!(rows["rows"].as_array().unwrap().len() >= 3);
        assert!(rows["schema"]["properties"]["radius_m"].is_object());
        let recipes = body(&designer.handle("GET", "/api/rows?registry=recipes", ""));
        assert!(recipes["schema"]["definitions"]["Stack"].is_object());
        assert!(!recipes.to_string().contains("#/$defs/"));
        assert_eq!(
            designer.handle("GET", "/api/rows?registry=nope", "").status,
            400
        );
    }

    #[test]
    fn a_form_edit_undoes_saves_and_reports_problems() {
        let dir = scratch();
        let mut designer = Designer::open(&dir).unwrap();
        let rows = body(&designer.handle("GET", "/api/rows?registry=air", ""));
        let mut oak = rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "scrub_oak_air")
            .unwrap()
            .clone();
        oak["radius_m"] = json!(24.0);
        let update = json!({"registry": "air", "id": "scrub_oak_air", "row": oak});
        let state = body(&designer.handle("POST", "/api/row", &update.to_string()));
        assert_eq!(state["dirty"], true);
        assert_eq!(state["can_undo"], true);
        // A bad value applies but is reported, and saving is refused until it is fixed.
        oak["strength"] = json!(3.0);
        let update = json!({"registry": "air", "id": "scrub_oak_air", "row": oak});
        let state = body(&designer.handle("POST", "/api/row", &update.to_string()));
        assert!(state["problems"].to_string().contains("strength"));
        assert_eq!(designer.handle("POST", "/api/save", "").status, 409);
        designer.handle("POST", "/api/undo", "");
        let saved = designer.handle("POST", "/api/save", "");
        assert_eq!(
            saved.status,
            200,
            "{}",
            String::from_utf8_lossy(&saved.body)
        );
        let text = std::fs::read_to_string(dir.join("air.json")).unwrap();
        assert!(text.contains("24.0"));
        let used = body(&designer.handle("GET", "/api/where-used?id=square_wave", ""));
        assert!(!used.as_array().unwrap().is_empty());
    }

    #[test]
    fn raw_edits_arrive_as_json() {
        let mut designer = Designer::open(&scratch()).unwrap();
        let edit = json!({"kind": "set", "registry": "air", "id": "reed_air",
            "pointer": "/radius_m", "value": 7.5});
        assert_eq!(
            designer
                .handle("POST", "/api/edit", &edit.to_string())
                .status,
            200
        );
        let bad = json!({"kind": "set", "registry": "air", "id": "no_such",
            "pointer": "/radius_m", "value": 1.0});
        assert_eq!(
            designer
                .handle("POST", "/api/edit", &bad.to_string())
                .status,
            422
        );
    }
}
