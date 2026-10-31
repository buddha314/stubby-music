//! Shared synthesis patches for waveform resources and successful collection cues.
//! All DSP is provided by stubby_audio's existing Firewheel/fundsp modules.
use super::{
    audio::Synth,
    resources::{RecipeCatalog, ResourceCollected},
};
use bevy::prelude::*;
use serde::Deserialize;
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};
use stubby_recipes::Waveform;

#[derive(Clone, Copy, Deserialize)]
pub(crate) struct WaveformSound {
    pub freq_hz: f32,
    pub attack_seconds: f32,
    pub decay_seconds: f32,
    pub gain: f32,
    pub hearing_radius_metres: f32,
}
#[derive(Resource)]
struct CollectionSounds(WaveformSound);
#[derive(Resource)]
struct CueVoices {
    next_id: u64,
    active: Vec<([ModuleId; 4], f64)>,
}
impl Default for CueVoices {
    fn default() -> Self {
        Self {
            next_id: 10_000,
            active: Vec::new(),
        }
    }
}
pub struct WaveformAudioPlugin;
impl Plugin for WaveformAudioPlugin {
    fn build(&self, app: &mut App) {
        #[derive(Deserialize)]
        struct Preset {
            click: WaveformSound,
        }
        let preset: Preset = serde_json::from_str(include_str!(
            "../../../assets/creatures/lightning_bug/asset.json"
        ))
        .expect("shipped waveform preset is valid");
        app.insert_resource(CollectionSounds(preset.click))
            .init_resource::<CueVoices>()
            .add_systems(PostUpdate, collection_cues);
    }
}
/// One oscillator, envelope and VCA, triggered by an existing gate source.
/// The same patch is used by bugs and listener-level collection confirmations.
pub(crate) fn voice_commands(
    waveform: Waveform,
    ids: [ModuleId; 3],
    gate: PortRef,
    gain: f32,
    sound: WaveformSound,
) -> [Command; 7] {
    let [oscillator, envelope, vca] = ids;
    let kind = match waveform {
        Waveform::Square => ModuleKind::SquareVco {
            freq_hz: sound.freq_hz,
        },
    };
    [
        Command::AddModule {
            id: oscillator,
            kind,
        },
        Command::AddModule {
            id: envelope,
            kind: ModuleKind::Envelope {
                attack_s: sound.attack_seconds,
                decay_s: sound.decay_seconds,
            },
        },
        Command::AddModule {
            id: vca,
            kind: ModuleKind::Vca { gain },
        },
        Command::Connect {
            from: gate,
            to: PortRef::new(envelope, "gate"),
        },
        Command::Connect {
            from: PortRef::new(envelope, "out"),
            to: PortRef::new(vca, "cv"),
        },
        Command::Connect {
            from: PortRef::new(oscillator, "out"),
            to: PortRef::new(vca, "in"),
        },
        Command::Connect {
            from: PortRef::new(vca, "out"),
            to: PortRef::output(),
        },
    ]
}
fn collected_waveforms(event: &ResourceCollected, catalog: &RecipeCatalog) -> Vec<Waveform> {
    let mut waveforms = Vec::new();
    for stack in &event.yields {
        if let Some(waveform) = catalog.0.item(&stack.item).and_then(|item| item.waveform)
            && !waveforms.contains(&waveform)
        {
            waveforms.push(waveform);
        }
    }
    waveforms
}
fn collection_cues(
    mut events: MessageReader<ResourceCollected>,
    catalog: Option<Res<RecipeCatalog>>,
    synth: Option<Res<Synth>>,
    preset: Res<CollectionSounds>,
    mut voices: ResMut<CueVoices>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    let (Some(catalog), Some(synth)) = (catalog, synth) else {
        return;
    };
    voices.active.retain(|(ids, until)| {
        if now < *until {
            return true;
        }
        for index in [3, 1, 2, 0] {
            synth.send(Command::RemoveModule { id: ids[index] });
        }
        false
    });
    for event in events.read() {
        if !synth.is_running() {
            continue;
        }
        for waveform in collected_waveforms(event, &catalog) {
            let Some(next_id) = voices.next_id.checked_add(4) else {
                continue;
            };
            let ids = std::array::from_fn(|index| ModuleId(voices.next_id + index as u64));
            voices.next_id = next_id;
            synth.send(Command::AddModule {
                id: ids[0],
                kind: ModuleKind::Constant { value: 1. },
            });
            for command in voice_commands(
                waveform,
                [ids[1], ids[2], ids[3]],
                PortRef::new(ids[0], "out"),
                preset.0.gain,
                preset.0,
            ) {
                synth.send(command);
            }
            // The audio envelope owns duration. This timer only retires silent nodes,
            // with a generous tail for block scheduling and declicking.
            voices.active.push((
                ids,
                now + f64::from(preset.0.attack_seconds + preset.0.decay_seconds) + 0.2,
            ));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use stubby_audio::{Engine, EngineConfig};
    use stubby_recipes::{Catalog, Stack};
    #[test]
    fn collection_cue_is_single_and_its_temporary_voice_is_retired() {
        use super::super::audio::{AudioOutput, SynthAudioPlugin};
        use std::time::Duration;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_millis(10),
            ))
            .insert_resource(RecipeCatalog(
                Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
            ))
            .add_message::<ResourceCollected>()
            .add_plugins(SynthAudioPlugin {
                output: AudioOutput::Offline,
                demo: false,
            })
            .add_plugins(WaveformAudioPlugin);
        app.update();
        app.world_mut().write_message(ResourceCollected {
            instance: "bug".into(),
            yields: vec![Stack {
                item: "square_wave".into(),
                quantity: 2,
            }],
        });
        app.update();
        assert_eq!(app.world().resource::<CueVoices>().active.len(), 1);
        for _ in 0..60 {
            app.update();
        }
        assert!(app.world().resource::<CueVoices>().active.is_empty());
        assert_eq!(app.world().resource::<CueVoices>().next_id, 10_004);
    }

    #[test]
    fn multiple_items_of_one_waveform_create_one_confirmation() {
        let catalog = RecipeCatalog(
            Catalog::from_json(include_str!("../../../assets/recipes/catalog.json")).unwrap(),
        );
        let event = ResourceCollected {
            instance: "test".into(),
            yields: vec![
                Stack {
                    item: "square_wave".into(),
                    quantity: 2,
                },
                Stack {
                    item: "square_wave".into(),
                    quantity: 3,
                },
            ],
        };
        assert_eq!(
            collected_waveforms(&event, &catalog),
            vec![Waveform::Square]
        );
    }
    #[test]
    fn shared_confirmation_patch_sounds_once_and_decays_without_a_world_clock() {
        let sound = WaveformSound {
            freq_hz: 2200.,
            attack_seconds: 0.015,
            decay_seconds: 0.285,
            gain: 0.12,
            hearing_radius_metres: 3.,
        };
        let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
        engine
            .send(Command::AddModule {
                id: ModuleId(1),
                kind: ModuleKind::Constant { value: 1. },
            })
            .unwrap();
        for command in voice_commands(
            Waveform::Square,
            [ModuleId(2), ModuleId(3), ModuleId(4)],
            PortRef::new(ModuleId(1), "out"),
            sound.gain,
            sound,
        ) {
            engine.send(command).unwrap();
        }
        engine.flush().unwrap();
        let mut samples = vec![0.; 48_000]; // half a second of stereo at 48kHz
        renderer.render(&mut samples);
        assert!(samples[960..4800].iter().any(|sample| sample.abs() > 0.01));
        assert!(samples[28_800..].iter().all(|sample| sample.abs() < 0.0002));
    }
}
