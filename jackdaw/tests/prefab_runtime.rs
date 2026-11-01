//! Resolve the actual saved prefabs, then run the same binding plugin as Play.
#[path = "../src/prefab_bridge.rs"]
mod prefab_bridge;
use bevy::prelude::*;
use jackdaw_bsn::{JackdawBsnPlugin, UnresolvedTypes, load_bsn_document, parse_bsn_text};
use jackdaw_prefab::{PrefabTypesPlugin, resolve_scene};
use jackdaw_scene_types::{GltfSource, LodGroup, LodLevel};
use std::path::{Path, PathBuf};
use stubby_jackdaw::{
    WorldSeed,
    audio::{AudioOutput, SynthAudioPlugin},
    prefabs::{AuthoredLevel, PlacedAsset, PlacedPrefabsPlugin, PrefabError, WorldSources},
    pulse::LightningBugPlugin,
    resources::{MineNode, ResourceNode, ResourceRecipesPlugin},
    synth_modules::{SynthModule, SynthModulesPlugin},
};
fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "bsn") {
            out.push(path);
        }
    }
}
#[test]
fn saved_prefabs_become_live_runtime_assets() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut prefabs = Vec::new();
    files(&assets.join("prefabs"), &mut prefabs);
    prefabs.sort();
    assert!(prefabs.len() >= 27);
    for path in prefabs {
        let prefab = parse_bsn_text(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let rel = path.strip_prefix(&assets).unwrap();
        let scene=parse_bsn_text(&format!("#Placed\njackdaw::prefab::components::IsA {{ source: \"{}\", deleted: [] }}\njackdaw::prefab::components::PrefabEntityId(0)\n",rel.display())).unwrap();
        let resolved = resolve_scene(&scene, &|p: &Path| (p == rel).then_some(&prefab)).unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::transform::TransformPlugin,
            JackdawBsnPlugin,
            PrefabTypesPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>()
        .init_asset::<bevy::gltf::Gltf>()
        .init_asset::<AnimationClip>()
        .init_asset::<bevy::animation::graph::AnimationGraph>()
        .init_asset::<bevy::world_serialization::WorldAsset>()
        .insert_resource(AuthoredLevel)
        .insert_resource(WorldSeed(42))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<stubby_jackdaw::calendar::Calendar>();
        app.add_plugins(SynthAudioPlugin {
            output: AudioOutput::Off,
            demo: false,
        })
        .add_plugins((
            LightningBugPlugin { seed: 42 },
            ResourceRecipesPlugin,
            SynthModulesPlugin::default(),
            PlacedPrefabsPlugin,
            prefab_bridge::PrefabBridgePlugin,
        ))
        .register_type::<Transform>()
        .register_type::<Visibility>()
        .register_type::<GltfSource>()
        .register_type::<LodGroup>()
        .register_type::<LodLevel>();
        load_bsn_document(app.world_mut(), resolved);
        assert!(
            app.world().resource::<UnresolvedTypes>().types().is_empty(),
            "{}",
            rel.display()
        );
        app.update();
        app.update();
        let world = app.world_mut();
        let sources: Vec<_> = world
            .query::<(Entity, &PlacedAsset)>()
            .iter(world)
            .map(|(entity, asset)| (entity, asset.clone()))
            .collect();
        assert_eq!(sources.len(), 1, "{}", rel.display());
        let (entity, asset) = &sources[0];
        assert!(
            world.get::<PrefabError>(*entity).is_none(),
            "{}: {:?}",
            rel.display(),
            world.get::<PrefabError>(*entity)
        );
        if asset.asset == "lightning_bug" || asset.asset.starts_with("reed_clump_") {
            assert!(
                world.get::<ResourceNode>(*entity).is_some(),
                "{}",
                rel.display()
            );
        }
        if asset.asset.starts_with("scrub_oak_") || asset.asset == "mudpit_burbler" {
            assert!(world.get::<MineNode>(*entity).is_some());
            assert!(world.get::<ResourceNode>(*entity).is_none());
            assert_eq!(world.resource::<WorldSources>().0.len(), 1);
        }
        if asset.asset.ends_with("_vco") || asset.asset == "clock_lamp" {
            assert!(
                world.get::<SynthModule>(*entity).is_some(),
                "{}",
                rel.display()
            );
        }
    }
}
