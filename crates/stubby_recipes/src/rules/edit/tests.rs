use super::*;
use crate::rules::{ITEMS, RECIPES, RESOURCES};
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/recipes");

fn shipped() -> RulesDocument {
    RulesDocument::open(Path::new(DIR)).unwrap_or_else(|e| panic!("{e}"))
}

/// A private copy of the rules directory to save into.
fn scratch() -> std::path::PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "stubby-rules-edit-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    for file in FILES.iter().chain([&IDS_LOCK]) {
        std::fs::copy(Path::new(DIR).join(file), dir.join(file)).unwrap();
    }
    dir
}

fn test_recipe() -> Edit {
    Edit::Add {
        registry: Registry::Recipes,
        row: json!({ "id": "test_recipe", "name": "Test recipe",
            "inputs": [{ "item": "square_wave", "quantity": 1 }],
            "outputs": [{ "item": "reed", "quantity": 1 }] }),
        at: None,
    }
}

#[test]
fn the_shipped_rules_open_clean() {
    let doc = shipped();
    assert_eq!(doc.problems(), &[] as &[RulesError]);
    assert!(!doc.is_dirty());
}

#[test]
fn add_a_recipe() {
    let mut doc = shipped();
    doc.apply(test_recipe()).unwrap();
    assert!(doc.row(Registry::Recipes, "test_recipe").is_some());
    assert_eq!(doc.problems(), &[] as &[RulesError]);
    assert!(doc.is_dirty());
}

#[test]
fn a_dangling_reference_is_reported_and_kept() {
    let mut doc = shipped();
    doc.apply(test_recipe()).unwrap();
    doc.apply(Edit::Remove {
        registry: Registry::Items,
        id: "square_wave".into(),
        to: None,
        retire: true,
    })
    .unwrap();
    let problems = doc.problems();
    assert!(
        problems
            .iter()
            .any(|p| p.id == "test_recipe" && p.message.contains("square_wave")),
        "{problems:?}"
    );
    assert!(
        doc.row(Registry::Items, "square_wave").is_none(),
        "the edit is kept"
    );
}

#[test]
fn undo_and_redo() {
    let mut doc = shipped();
    let before = doc.rows(Registry::Items).to_vec();
    doc.apply(Edit::Remove {
        registry: Registry::Items,
        id: "square_wave".into(),
        to: None,
        retire: true,
    })
    .unwrap();
    assert!(!doc.problems().is_empty());
    assert!(doc.undo());
    assert_eq!(doc.rows(Registry::Items), before.as_slice());
    assert_eq!(doc.problems(), &[] as &[RulesError]);
    assert!(!doc.is_dirty());
    assert!(doc.redo());
    assert!(doc.row(Registry::Items, "square_wave").is_none());
}

#[test]
fn a_shipped_id_needs_a_migration_to_go() {
    let mut doc = shipped();
    let err = doc.apply(Edit::Remove {
        registry: Registry::Items,
        id: "reed".into(),
        to: None,
        retire: false,
    });
    assert_eq!(err, Err(EditError::NeedsMigration("reed".into())));
    assert!(
        doc.row(Registry::Items, "reed").is_some(),
        "nothing changed"
    );
    assert!(!doc.can_undo());
}

#[test]
fn rename_follows_every_reference_and_writes_its_migration() {
    let mut doc = shipped();
    assert!(!doc.where_used("square_wave").is_empty());
    doc.apply(test_recipe()).unwrap();
    doc.apply(Edit::Rename {
        registry: Registry::Items,
        from: "square_wave".into(),
        to: "square_pattern".into(),
    })
    .unwrap();
    assert_eq!(doc.problems(), &[] as &[RulesError]);
    assert!(doc.where_used("square_wave").is_empty());
    let uses = doc.where_used("square_pattern");
    assert!(
        uses.iter()
            .any(|u| u.file == RESOURCES && u.row == "lightning_bug"),
        "{uses:?}"
    );
    assert!(
        uses.iter()
            .any(|u| u.file == RECIPES && u.row == "test_recipe"),
        "{uses:?}"
    );
    let migrations = doc.values[MIGRATIONS]["migrations"].as_array().unwrap();
    assert!(
        migrations
            .iter()
            .any(|m| m["from"] == "square_wave" && m["to"] == "square_pattern")
    );
}

#[test]
fn ids_are_checked() {
    let mut doc = shipped();
    let add = |id: &str| Edit::Add {
        registry: Registry::Items,
        row: json!({ "id": id, "name": "x", "description": "x" }),
        at: None,
    };
    assert_eq!(
        doc.apply(add("Bad ID")),
        Err(EditError::BadId("Bad ID".into()))
    );
    // One namespace: an item may not take a resource's ID.
    assert_eq!(
        doc.apply(add("lightning_bug")),
        Err(EditError::IdTaken("lightning_bug".into()))
    );
}

#[test]
fn set_edits_inside_a_row() {
    let mut doc = shipped();
    doc.apply(Edit::Set {
        registry: Registry::Resources,
        id: "lightning_bug".into(),
        pointer: "/yields/0/quantity".into(),
        value: Some(json!(3)),
    })
    .unwrap();
    assert_eq!(
        doc.row(Registry::Resources, "lightning_bug").unwrap()["yields"][0]["quantity"],
        3
    );
    let bad = doc.apply(Edit::Set {
        registry: Registry::Resources,
        id: "lightning_bug".into(),
        pointer: "/nowhere/0".into(),
        value: Some(json!(1)),
    });
    assert!(matches!(bad, Err(EditError::BadPointer(_))));
}

#[test]
fn saving_is_canonical_and_locks_new_ids() {
    let dir = scratch();
    let mut doc = RulesDocument::open(&dir).unwrap();
    doc.apply(test_recipe()).unwrap();
    let written = doc.save(&dir).unwrap();
    assert!(
        written.contains(&RECIPES) && written.contains(&IDS_LOCK),
        "{written:?}"
    );
    assert!(
        !written.contains(&ITEMS),
        "unchanged files are not rewritten"
    );
    assert!(!doc.is_dirty());
    // What rules-fmt --check would see: nothing to change.
    let texts: Vec<String> = FILES
        .iter()
        .map(|f| std::fs::read_to_string(dir.join(f)).unwrap())
        .collect();
    for (file, canonical) in canonical_files(&files_of(&texts)).unwrap() {
        assert_eq!(
            std::fs::read_to_string(dir.join(file)).unwrap(),
            canonical,
            "{file}"
        );
    }
    let lock = std::fs::read_to_string(dir.join(IDS_LOCK)).unwrap();
    assert!(lock.lines().any(|l| l == "test_recipe"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn saving_refuses_invalid_rules_and_newer_files() {
    let dir = scratch();
    let mut doc = RulesDocument::open(&dir).unwrap();
    doc.apply(Edit::Remove {
        registry: Registry::Items,
        id: "square_wave".into(),
        to: None,
        retire: true,
    })
    .unwrap();
    assert!(matches!(doc.save(&dir), Err(EditError::Invalid(_))));
    doc.undo();
    doc.apply(test_recipe()).unwrap();
    std::fs::write(dir.join(ITEMS), "{}").unwrap();
    assert_eq!(doc.save(&dir), Err(EditError::ChangedOnDisk(ITEMS.into())));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn row_schemas_offer_existing_ids_and_edits_arrive_as_json() {
    let doc = shipped();
    let schema = doc.row_schema(Registry::Recipes);
    let stack = &schema["$defs"]["Stack"]["properties"]["item"];
    let offered = stack["enum"].as_array().unwrap();
    assert!(offered.contains(&json!("square_wave")));
    assert!(schema["properties"]["id"].is_object());
    let edit: Edit = serde_json::from_value(json!({
        "kind": "set", "registry": "air", "id": "scrub_oak_air",
        "pointer": "/radius_m", "value": 20.0
    }))
    .unwrap();
    assert!(matches!(
        edit,
        Edit::Set {
            registry: Registry::Air,
            ..
        }
    ));
}
