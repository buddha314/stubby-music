//! Every world-asset prefab under `assets/` places in Jackdaw: it parses, a
//! scene instancing it resolves, every component it names is a type the editor
//! knows, the model it points at exists, and no prefab carries its own LodGroup
//! (levels of detail belong to the model).
//! Prefabs are made and edited in Jackdaw (Save as Prefab); this checks whatever is there.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_stubby_audio::pulse::PulseFollower;
use jackdaw_bsn::{JackdawBsnPlugin, UnresolvedTypes, load_bsn_document, parse_bsn_text};
use jackdaw_prefab::{PrefabTypesPlugin, resolve_scene};
use jackdaw_scene_types::{GltfSource, LodGroup, LodLevel};
use stubby_jackdaw::resources::{MineNode, ResourceNode};

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

fn prefab_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            prefab_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "bsn")
            && std::fs::read_to_string(&path)
                .unwrap()
                .lines()
                .any(|line| line.trim() == "jackdaw::prefab::components::Prefab")
        {
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
        .register_type::<PulseFollower>()
        .register_type::<stubby_core::AcousticOccluder>();
    app
}

#[test]
fn every_prefab_places_with_known_types_and_real_models() {
    let mut files = Vec::new();
    prefab_files(&assets(), &mut files);
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
        if rel.starts_with("flora") && rel.to_string_lossy().contains("scrub_oak") {
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
    prefab_files(&assets(), &mut files);
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

#[test]
fn settlement_review_scenes_resolve_and_preserve_the_shared_site() {
    use jackdaw_scene_types::Brush;

    let root = assets();
    let mut files = Vec::new();
    prefab_files(&root, &mut files);
    let library: std::collections::HashMap<_, _> = files
        .into_iter()
        .map(|file| {
            (
                file.strip_prefix(&root).unwrap().to_path_buf(),
                parse_bsn_text(&std::fs::read_to_string(&file).unwrap()).unwrap(),
            )
        })
        .collect();
    let landmarks = [
        ("mudpit_burbler.glb", Vec3::new(7., 0., 4.)),
        ("campfire.glb", Vec3::new(11., 0., 1.4)),
        ("stream_segment.glb", Vec3::new(9.6, 0., 4.)),
        ("rock_formation_2.glb", Vec3::new(12., 0., 10.4)),
    ];
    for (phase, brush_count, tree_count) in [
        ("camp", 0, 0),
        ("grove", 1, 3),
        ("village", 8, 3),
        ("city", 17, 4),
    ] {
        let path = root.join(format!("world/phase_reviews/{phase}.bsn"));
        let scene = parse_bsn_text(&std::fs::read_to_string(path).unwrap())
            .unwrap_or_else(|err| panic!("{phase}: {err:?}"));
        let resolved = resolve_scene(&scene, &|path: &Path| library.get(path))
            .unwrap_or_else(|err| panic!("{phase}: {err:?}"));
        let mut app = editor_world();
        app.register_type::<Brush>()
            .register_type::<StandardMaterial>()
            .register_type::<stubby_core::AssetState>();
        let world = app.world_mut();
        load_bsn_document(world, resolved);
        assert!(
            world.resource::<UnresolvedTypes>().types().is_empty(),
            "{phase}: {:?}",
            world.resource::<UnresolvedTypes>().types()
        );

        let mut models = world.query::<(&GltfSource, &Transform)>();
        for (model, position) in landmarks {
            assert!(
                models.iter(world).any(|(source, transform)| {
                    source.path.ends_with(model) && transform.translation == position
                }),
                "{phase}: missing or moved landmark {model}"
            );
        }
        let mut brushes = world.query::<&Brush>();
        assert_eq!(brushes.iter(world).count(), brush_count, "{phase}");
        assert!(brushes.iter(world).all(|brush| brush.faces.len() == 6));
        let mut trees = world.query::<(&MineNode, &stubby_core::AssetState)>();
        assert_eq!(trees.iter(world).count(), tree_count, "{phase}");
        let mut ids = std::collections::HashSet::new();
        for (mine, state) in trees.iter(world) {
            assert!(
                ids.insert(&mine.instance),
                "{phase}: duplicate tree instance"
            );
            assert!(state.growth >= 0.6, "{phase}: oak lacks authored growth");
        }
    }
}

/// Validate our authored references using Jackdaw's actual metadata schema.
fn validate_model_lods(
    model: &Path,
    settings: &jackdaw_scene_types::model_import::ModelSettings,
) -> Result<(), String> {
    use jackdaw_scene_types::model_import::LevelShow;
    let gltf = gltf::Gltf::open(model).map_err(|error| format!("{}: {error}", model.display()))?;
    let Some(lod) = &settings.lod else {
        return Err(format!("{}: missing model LOD settings", model.display()));
    };
    let names: std::collections::HashSet<_> = gltf.nodes().filter_map(|node| node.name()).collect();
    let mut previous = f32::INFINITY;
    for level in &lod.levels {
        if !level.screen_height.is_finite()
            || level.screen_height < 0.0
            || level.screen_height >= previous
        {
            return Err(format!("{}: unordered LOD thresholds", model.display()));
        }
        previous = level.screen_height;
        if let LevelShow::Nodes(nodes) = &level.show {
            for node in nodes {
                if !names.contains(node.as_str()) {
                    return Err(format!("{}: missing LOD node {node}", model.display()));
                }
            }
        }
    }
    Ok(())
}

fn check_model_tree(dir: &Path, count: &mut usize) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_model_tree(&path, count);
        } else if path.extension().is_some_and(|ext| ext == "glb") {
            let gltf = gltf::Gltf::open(&path).unwrap();
            let authored = gltf.nodes().filter_map(|node| node.name()).any(|name| {
                name.rsplit_once("_LOD")
                    .is_some_and(|(_, suffix)| suffix.parse::<usize>().is_ok())
            });
            let meta = path.with_extension("glb.meta");
            assert!(
                !authored || meta.is_file(),
                "{}: missing LOD metadata",
                path.display()
            );
            if meta.is_file() {
                let settings = jackdaw_scene_types::model_import::read_model_meta(
                    &std::fs::read(&meta).unwrap(),
                )
                .unwrap_or_else(|| panic!("{}: invalid Jackdaw metadata", meta.display()));
                validate_model_lods(&path, &settings).unwrap_or_else(|error| panic!("{error}"));
                *count += 1;
            }
        }
    }
}

/// Every model with authored LOD nodes has valid metadata, wherever it lives. (No count
/// of models: the asset set changes all the time.)
#[test]
fn model_owned_lod_metadata_resolves_after_asset_moves() {
    let mut count = 0;
    check_model_tree(&assets(), &mut count);
    assert!(count > 0, "no model metadata found under assets/");
}

#[test]
fn missing_lod_node_reports_model_and_node() {
    use jackdaw_scene_types::model_import::LevelShow;
    let model = assets().join("flora/scrub_oak/scrub_oak_0/scrub_oak_0.glb");
    let mut settings = jackdaw_scene_types::model_import::read_model_meta(
        &std::fs::read(model.with_extension("glb.meta")).unwrap(),
    )
    .unwrap();
    settings.lod.as_mut().unwrap().levels[1].show = LevelShow::Nodes(vec!["missing_LOD1".into()]);
    let error = validate_model_lods(&model, &settings).unwrap_err();
    assert!(error.contains("scrub_oak_0.glb"), "{error}");
    assert!(error.contains("missing_LOD1"), "{error}");
}

#[test]
fn migrated_authored_and_generated_models_load_without_editor_cache() {
    use jackdaw_scene_types::model_import::{GeneratedLevelCache, level_key};
    use jackdaw_scene_types::model_parts::ModelParts;
    // Stage just the shipped model and meta pairs. No editor project/cache is copied.
    let project = tempfile::tempdir().unwrap();
    let paths = [
        "flora/scrub_oak/scrub_oak_0/scrub_oak_0.glb",
        "machines/mudpit_burbler/mudpit_burbler.glb",
    ];
    let mut keys = Vec::new();
    for path in paths {
        let target = project.path().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(assets().join(path), &target).unwrap();
        let meta = format!("{path}.meta");
        std::fs::copy(assets().join(&meta), project.path().join(&meta)).unwrap();
        let settings = jackdaw_scene_types::model_import::read_model_meta(
            &std::fs::read(assets().join(meta)).unwrap(),
        )
        .unwrap();
        keys.extend(
            settings
                .lod
                .unwrap()
                .levels
                .iter()
                .enumerate()
                .map(|(i, level)| level_key(path, i, &level.show)),
        );
    }
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    app.add_plugins(AssetPlugin {
        file_path: project.path().to_string_lossy().into_owned(),
        ..default()
    });
    app.add_plugins(bevy::world_serialization::WorldSerializationPlugin);
    app.init_asset::<Mesh>();
    app.init_asset::<AnimationClip>();
    app.add_plugins(bevy::gltf::GltfPlugin::default());
    app.add_plugins(jackdaw_runtime::JackdawPlugin);
    app.finish();
    app.cleanup();
    app.world().resource::<GeneratedLevelCache>().set(None);
    for key in &keys {
        app.world_mut().resource_mut::<ModelParts>().request(key);
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        app.update();
        let models = app.world().resource::<ModelParts>();
        if keys
            .iter()
            .all(|key| models.get(key).is_some_and(|model| !model.parts.is_empty()))
        {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let models = app.world().resource::<ModelParts>();
    let missing: Vec<_> = keys
        .iter()
        .filter(|key| models.get(key).is_none())
        .collect();
    panic!("migrated models did not load all LODs without cache: {missing:?}");
}

#[test]
fn production_opening_keeps_its_oak_and_burbler_in_front_of_spawn() {
    let mut files = Vec::new();
    prefab_files(&assets(), &mut files);
    let library: std::collections::HashMap<_, _> = files
        .into_iter()
        .map(|file| {
            (
                file.strip_prefix(assets()).unwrap().to_path_buf(),
                parse_bsn_text(&std::fs::read_to_string(file).unwrap()).unwrap(),
            )
        })
        .collect();
    let scene =
        parse_bsn_text(&std::fs::read_to_string(assets().join("scene.bsn")).unwrap()).unwrap();
    let resolved = resolve_scene(&scene, &|path: &Path| library.get(path)).unwrap();
    let mut app = editor_world();
    load_bsn_document(app.world_mut(), resolved);
    assert!(app.world().resource::<UnresolvedTypes>().types().is_empty());
    let camera = stubby_game::controls::spawn_transform(&Default::default());
    let mut found = [false; 2];
    for (source, transform, name) in app
        .world_mut()
        .query::<(&GltfSource, &Transform, Option<&Name>)>()
        .iter(app.world())
    {
        // The echo grotto's burbler (add-echo-grotto) stands far off; this is the opening's.
        let opening = name.is_none_or(|name| name.as_str() == "mudpit_burbler");
        let expected = if source.path.contains("scrub_oak_0.glb") {
            found[0] = true;
            Some((-24.144297, -94.36827))
        } else if source.path.contains("mudpit_burbler.glb") && opening {
            found[1] = true;
            Some((-1.45856, -41.11013))
        } else {
            None
        };
        if let Some((x, z)) = expected {
            let in_view = camera.rotation.inverse() * (transform.translation - camera.translation);
            assert!(
                (in_view.x - x).abs() < 0.001,
                "{}: {in_view:?}",
                source.path
            );
            assert!(
                (in_view.z - z).abs() < 0.001,
                "{}: {in_view:?}",
                source.path
            );
        }
    }
    assert!(
        found.into_iter().all(|found| found),
        "opening must keep its oak and burbler"
    );
}
