//! The field recorder's persistent discoveries feed the rack's clip library.

use crate::resources::{PlayerInventory, RecipeCatalog};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use bevy_stubby_audio::Synth;
use bevy_stubby_rack::{
    rack::decode_wav,
    samples::{Clip, SampleLibrary},
};
use std::{collections::BTreeMap, io};
use stubby_audio::SampleId;

pub struct SampleLibraryPlugin;

impl Plugin for SampleLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SampleWav>()
            .init_asset_loader::<SampleLoader>()
            .add_systems(Update, sync_discoveries);
    }
}

#[derive(Asset, TypePath)]
struct SampleWav {
    rate: u32,
    channels: Vec<Vec<f32>>,
}

#[derive(Default, TypePath)]
struct SampleLoader;

impl AssetLoader for SampleLoader {
    type Asset = SampleWav;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<SampleWav, io::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let (rate, channels) = decode_wav(&bytes)
            .filter(|(rate, channels)| *rate > 0 && channels.first().is_some_and(|c| !c.is_empty()))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "expected nonempty PCM WAV")
            })?;
        Ok(SampleWav { rate, channels })
    }
}

fn sync_discoveries(
    catalog: Option<Res<RecipeCatalog>>,
    inventory: Res<PlayerInventory>,
    server: Res<AssetServer>,
    assets: Res<Assets<SampleWav>>,
    synth: Option<Res<Synth>>,
    mut library: ResMut<SampleLibrary>,
    mut pending: Local<BTreeMap<String, Handle<SampleWav>>>,
) {
    let (Some(catalog), Some(thread)) = (catalog, synth.as_ref().and_then(|synth| synth.thread()))
    else {
        return;
    };
    for id in inventory.book.items() {
        if library.0.contains_key(id) {
            continue;
        }
        let Some(item) = catalog.0.item(id) else {
            continue;
        };
        let Some(sample) = &item.sample else { continue };
        let handle = pending
            .entry(id.into())
            .or_insert_with(|| server.load(sample.path.clone()));
        let Some(wav) = assets.get(handle) else {
            continue;
        };
        // Reserve the low IDs for the existing UI cues and the impact review bank.
        let Ok(index) = u32::try_from(library.0.len()) else {
            break;
        };
        let Some(engine_id) = index.checked_add(1024).map(SampleId) else {
            break;
        };
        let seconds = wav.channels[0].len() as f32 / wav.rate as f32;
        thread.load_sample(engine_id, wav.rate, wav.channels.clone());
        library.0.insert(
            id.into(),
            Clip {
                engine_id,
                name: item.name.clone(),
                source: sample.source.clone(),
                seconds,
            },
        );
        pending.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_stubby_audio::{AudioOutput, SynthAudioPlugin};
    use std::time::{Duration, Instant};

    #[test]
    fn discovered_sample_loads_from_its_catalogue_wav() {
        let catalog = stubby_recipes::Catalog::from_json(
            r#"{
            "items": [{
                "id": "test_rock", "name": "Rock hit", "description": "Test impact",
                "sample": {
                    "path": "audio/samples/impact_review/rock-wood.wav",
                    "source": "Stone striking wood"
                }
            }],
            "resources": [], "recipes": []
        }"#,
        )
        .unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            },
        ))
        .add_plugins((
            SynthAudioPlugin {
                output: AudioOutput::Offline,
                demo: false,
            },
            SampleLibraryPlugin,
        ))
        .insert_resource(RecipeCatalog(catalog))
        .init_resource::<PlayerInventory>()
        .init_resource::<SampleLibrary>();

        app.update();
        assert!(app.world().resource::<SampleLibrary>().0.is_empty());
        app.world_mut()
            .resource_mut::<PlayerInventory>()
            .book
            .discover("test_rock");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            app.update();
            if let Some(clip) = app.world().resource::<SampleLibrary>().0.get("test_rock") {
                assert_eq!(clip.name, "Rock hit");
                assert!((clip.seconds - 0.094).abs() < 0.001);
                break;
            }
            assert!(
                Instant::now() < deadline,
                "sample WAV did not load after discovery"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
