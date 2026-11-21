//! A bounded rack review using public-domain impact samples.

use bevy::prelude::*;
use bevy_stubby_audio::Synth;
use bevy_stubby_rack::{
    machine_config::{JackRef, Profiles, RackCable, RackPlace},
    rack::{RackEngine, decode_wav},
    samples::{Clip, SampleLibrary},
};
use stubby_audio::SampleId;

const CLIPS: [(&str, &str, &str, &[u8]); 3] = [
    (
        "rock_wood",
        "Rock hit",
        "Stone striking wood",
        include_bytes!("../../../assets/audio/samples/impact_review/rock-wood.wav"),
    ),
    (
        "metal_click",
        "Metal click",
        "Yarmonics metal recording",
        include_bytes!("../../../assets/audio/samples/impact_review/metal-click.wav"),
    ),
    (
        "hammer_drop",
        "Hammer drop",
        "Hammer landing on a wooden floor",
        include_bytes!("../../../assets/audio/samples/impact_review/hammer-drop.wav"),
    ),
];

pub struct SamplerReviewPlugin;

impl Plugin for SamplerReviewPlugin {
    fn build(&self, app: &mut App) {
        if std::env::args().any(|arg| arg == "--sampler-review") {
            app.add_systems(Startup, review_rack)
                .add_systems(Update, (load_clips, open_review_rack).chain());
        }
    }
}

fn jack(place: &str, jack: &str) -> JackRef {
    JackRef {
        place: place.into(),
        port: jack.into(),
    }
}

fn review_rack(mut profiles: ResMut<Profiles>) {
    let Some(config) = profiles.0.get_mut("mudpit_burbler") else {
        return;
    };
    config.title = "SAMPLER REVIEW".into();
    config.targets.clear();
    config.places.retain(|place| place.place == "clock");
    config.cables.clear();
    for (place, module, hp) in [
        ("sampler_a", "sample_player", 8),
        ("sampler_b", "sample_player", 8),
        ("mixer", "mixer", 8),
    ] {
        config.places.push(RackPlace {
            place: place.into(),
            kind: module.into(),
            hp,
            fixed: true,
            module: Some(module.into()),
            accepts: Vec::new(),
            optional: false,
            borrowed: false,
            inherited: false,
        });
    }
    for (from, to) in [
        (jack("clock", "gate"), jack("sampler_a", "trigger")),
        (jack("clock", "gate"), jack("sampler_b", "trigger")),
        (jack("sampler_a", "out"), jack("mixer", "in_1")),
        (jack("sampler_b", "out"), jack("mixer", "in_2")),
    ] {
        config.cables.push(RackCable {
            color: "amber".into(),
            from,
            to,
            hint: String::new(),
            optional: false,
            inherited: true,
        });
    }
    config.normalled_output = Some(jack("mixer", "mix"));
}

fn open_review_rack(
    library: Res<SampleLibrary>,
    mut screens: ResMut<stubby_core::ScreenMode>,
    racks: Query<(
        Entity,
        &stubby_core::PlacedAsset,
        &GlobalTransform,
        &RackEngine,
    )>,
    mut player: Query<
        &mut Transform,
        (
            With<crate::controls::Player>,
            Without<stubby_core::HudCamera>,
        ),
    >,
    ground: Res<stubby_core::GroundHeight>,
    mut opened: Local<bool>,
) {
    if *opened || library.0.len() != CLIPS.len() {
        return;
    }
    let Some((entity, _, transform, _)) = racks
        .iter()
        .filter(|(_, asset, _, rack)| asset.asset == "mudpit_burbler" && rack.sounding)
        .min_by(|a, b| {
            let distance = |transform: &GlobalTransform| {
                let p = transform.translation();
                Vec2::new(p.x, p.z).distance_squared(stubby_core::OPENING_SPAWN_XZ)
            };
            distance(a.2)
                .total_cmp(&distance(b.2))
                .then_with(|| a.1.instance.cmp(&b.1.instance))
        })
    else {
        return;
    };
    let Ok(mut player) = player.single_mut() else {
        return;
    };
    let at = transform.translation();
    let x = at.x + 2.5;
    let z = at.z + 1.5;
    let eye = Vec3::new(x, ground.at(x, z) + 1.7, z);
    *player = Transform::from_translation(eye).looking_at(at + Vec3::Y, Vec3::Y);
    screens.open = Some(stubby_core::Screen::MachineConfiguration(entity));
    info!("Sampler review rack opened with {} clips", library.0.len());
    *opened = true;
}

fn load_clips(
    synth: Option<Res<Synth>>,
    mut library: ResMut<SampleLibrary>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let Some(thread) = synth.as_ref().and_then(|synth| synth.thread()) else {
        return;
    };
    for (index, (id, name, source, wav)) in CLIPS.iter().enumerate() {
        if let Some((rate, channels)) = decode_wav(wav) {
            let seconds = channels[0].len() as f32 / rate as f32;
            let engine_id = SampleId(128 + index as u32);
            thread.load_sample(engine_id, rate, channels);
            library.0.insert(
                (*id).into(),
                Clip {
                    engine_id,
                    name: (*name).into(),
                    source: (*source).into(),
                    seconds,
                },
            );
        }
    }
    *loaded = true;
}
