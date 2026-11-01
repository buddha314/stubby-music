//! Every world-asset prefab under `assets/prefabs` places in Jackdaw: it parses, a
//! scene instancing it resolves, every component it names is a type the editor
//! knows, the model it points at exists, and no prefab carries its own LodGroup
//! (levels of detail belong to the model).
//! Prefabs are made and edited in Jackdaw (Save as Prefab); this checks whatever is there.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use jackdaw_bsn::{JackdawBsnPlugin, UnresolvedTypes, load_bsn_document, parse_bsn_text};
use jackdaw_prefab::{PrefabTypesPlugin, resolve_scene};
use jackdaw_scene_types::{GltfSource, LodGroup, LodLevel};
use stubby_jackdaw::pulse::PulseFollower;
use stubby_jackdaw::resources::{MineNode, ResourceNode};

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn prefab_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            prefab_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "bsn") {
            out.push(path);
        }
    }
}

/// An app that knows the types a placed prefab may name, as the editor does.
fn editor_world() -> App {
    let mut app = App::new();
    app.add_plugins((JackdawBsnPlugin, PrefabTypesPlugin))
        .register_type::<Transform>()
        .register_type::<Visibility>()
        .register_type::<GltfSource>()
        .register_type::<LodGroup>()
        .register_type::<LodLevel>()
        .register_type::<MineNode>()
        .register_type::<ResourceNode>()
        .register_type::<PulseFollower>();
    app
}

#[test]
fn every_prefab_places_with_known_types_and_real_models() {
    let mut files = Vec::new();
    prefab_files(&assets().join("prefabs"), &mut files);
    files.sort();
    assert!(
        files.len() >= 27,
        "expected the world-asset prefabs, found {}",
        files.len()
    );

    for file in files {
        let rel = file.strip_prefix(assets()).unwrap().to_path_buf();
        let prefab = parse_bsn_text(&std::fs::read_to_string(&file).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e:?}", rel.display()));
        // A scene at the assets root placing one instance, as a drag into the viewport does.
        let scene = parse_bsn_text(&format!(
            "#Placed\njackdaw::prefab::components::IsA {{ source: \"{}\", deleted: [] }}\n\
             jackdaw::prefab::components::PrefabEntityId(0)\n",
            rel.display()
        ))
        .unwrap();
        let lookup = |p: &Path| (p == rel.as_path()).then_some(&prefab);
        let resolved = resolve_scene(&scene, &lookup)
            .unwrap_or_else(|e| panic!("{}: does not resolve: {e:?}", rel.display()));

        let mut app = editor_world();
        load_bsn_document(app.world_mut(), resolved);
        let unresolved = app.world().resource::<UnresolvedTypes>().types().clone();
        assert!(
            unresolved.is_empty(),
            "{}: unknown types {unresolved:?}",
            rel.display()
        );

        let world = app.world_mut();
        let mut sources = world.query::<(&GltfSource, Option<&LodGroup>, Option<&MineNode>)>();
        let placed: Vec<_> = sources.iter(world).collect();
        assert_eq!(placed.len(), 1, "{}: one model per prefab", rel.display());
        let (source, lod, mine) = placed[0];
        assert!(
            assets().join(&source.path).is_file(),
            "{}: no model at {}",
            rel.display(),
            source.path
        );
        // Levels of detail belong to the model (its `_LOD<n>` nodes and import
        // settings), never to a placement.
        assert!(
            lod.is_none(),
            "{}: LodGroup on a prefab; put the levels in the model",
            rel.display()
        );
        if rel.starts_with("prefabs/flora") && rel.to_string_lossy().contains("scrub_oak") {
            assert_eq!(
                mine.map(|m| m.mine.as_str()),
                Some("scrub_oak"),
                "{}",
                rel.display()
            );
        }
    }
}

#[test]
fn multi_prefab_review_scene_resolves_seven_separate_instances() {
    let mut files = Vec::new();
    prefab_files(&assets().join("prefabs"), &mut files);
    let library: std::collections::HashMap<_, _> = files
        .into_iter()
        .map(|file| {
            (
                file.strip_prefix(assets()).unwrap().to_path_buf(),
                parse_bsn_text(&std::fs::read_to_string(&file).unwrap()).unwrap(),
            )
        })
        .collect();
    let scene = parse_bsn_text(
        &std::fs::read_to_string(assets().join("review_prefab_runtime.bsn")).unwrap(),
    )
    .unwrap();
    let resolved = resolve_scene(&scene, &|path: &Path| library.get(path)).unwrap();
    let mut app = editor_world();
    load_bsn_document(app.world_mut(), resolved);
    assert!(app.world().resource::<UnresolvedTypes>().types().is_empty());
    assert_eq!(
        app.world_mut()
            .query::<&GltfSource>()
            .iter(app.world())
            .count(),
        7
    );
}
