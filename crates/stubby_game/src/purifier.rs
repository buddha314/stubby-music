//! A tuned purifier raises the clarity of its stream below the dam.
use bevy::prelude::*;
use bevy_stubby_rack::rack::RackTargets;
use std::collections::HashMap;
use stubby_core::{AssetClock, MachineVoice, PlacedAsset, PrefabError};
use stubby_streams::{LaidStream, PortOwner};

pub const PURIFIER: &str = "reed_water_purifier";
/// Maximum distance from a dam's origin to the stream centre line.
const CHANNEL_REACH_M: f32 = 0.75;
/// Seconds for a dam to add or lose one clarity level.
const TRANSITION_SECONDS: f32 = 1.0;

pub struct PurifierPlugin;
impl Plugin for PurifierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterClarity>()
            .add_systems(PostUpdate, (update_clarity, tint_water).chain());
    }
}

#[derive(Clone, Copy)]
struct Lift {
    stream: Entity,
    at: f32,
    amount: f32,
}

#[derive(Default, Resource)]
pub(crate) struct WaterClarity {
    lifts: HashMap<Entity, Lift>,
    profiles: HashMap<Entity, ClarityProfile>,
}

#[derive(Default)]
struct ClarityProfile {
    incoming: f32,
    lifts: Vec<(f32, f32)>,
}

impl WaterClarity {
    pub(crate) fn at(&self, entity: Entity, stream: &LaidStream, world: Vec3) -> f32 {
        let (along, _) = stream.nearest_along(world);
        self.along(entity, along).unwrap_or(stream.clarity)
    }

    fn along(&self, entity: Entity, along: f32) -> Option<f32> {
        let profile = self.profiles.get(&entity)?;
        let levels = profile.incoming
            + profile
                .lifts
                .iter()
                .filter(|(at, _)| *at <= along)
                .map(|(_, amount)| amount)
                .sum::<f32>();
        Some((levels / 3.0).clamp(0.0, 1.0))
    }

    fn advance(
        &mut self,
        streams: &HashMap<Entity, Option<PortOwner>>,
        active: HashMap<Entity, Lift>,
        seconds: f32,
    ) {
        if active.is_empty() && self.lifts.is_empty() {
            self.profiles.clear();
            return;
        }
        let step = seconds / TRANSITION_SECONDS;
        for (dam, mut next) in active.iter().map(|(dam, lift)| (*dam, *lift)) {
            if let Some(previous) = self
                .lifts
                .get(&dam)
                .filter(|previous| previous.stream == next.stream)
            {
                next.amount = previous.amount;
            }
            next.amount = (next.amount + step).min(1.0);
            self.lifts.insert(dam, next);
        }
        self.lifts.retain(|dam, lift| {
            if !active.contains_key(dam) {
                lift.amount -= step;
            }
            lift.amount > 0.0 && streams.contains_key(&lift.stream)
        });
        self.profiles.clear();
        for lift in self.lifts.values() {
            self.profiles
                .entry(lift.stream)
                .or_default()
                .lifts
                .push((lift.at, lift.amount));
        }
        let outgoing: HashMap<_, f32> = self
            .profiles
            .iter()
            .map(|(&stream, profile)| {
                (stream, profile.lifts.iter().map(|(_, amount)| amount).sum())
            })
            .collect();
        for (&entity, &upstream) in streams {
            let mut upstream = upstream;
            let mut incoming = 0.0;
            for _ in 0..streams.len() {
                let Some(PortOwner::Stream(parent)) = upstream else {
                    break;
                };
                incoming += outgoing.get(&parent).copied().unwrap_or_default();
                upstream = streams.get(&parent).copied().flatten();
            }
            if incoming > 0.0 {
                self.profiles.entry(entity).or_default().incoming = incoming;
            }
        }
    }
}

fn update_clarity(
    time: Res<Time<Virtual>>,
    mut clarity: ResMut<WaterClarity>,
    streams: Query<(Entity, &LaidStream)>,
    voices: Query<&MachineVoice>,
    dams: Query<
        (Entity, &PlacedAsset, &GlobalTransform, &RackTargets),
        (With<AssetClock>, Without<PrefabError>),
    >,
) {
    let flowing: HashMap<_, _> = streams
        .iter()
        .filter_map(|(entity, stream)| {
            let voice = voices.get(stream.source?).ok()?;
            voice.sounding.then_some((entity, stream.upstream))
        })
        .collect();
    let active = dams
        .iter()
        .filter_map(|(dam, asset, transform, targets)| {
            if asset.asset != PURIFIER || !targets.sounding || !targets.met {
                return None;
            }
            let at = transform.translation();
            streams
                .iter()
                .filter(|(entity, _)| flowing.contains_key(entity))
                .filter_map(|(entity, stream)| {
                    let (along, point) = stream.nearest_along(at);
                    let distance = Vec2::new(at.x - point.x, at.z - point.z).length();
                    (distance <= CHANNEL_REACH_M).then_some((
                        distance,
                        Lift {
                            stream: entity,
                            at: along,
                            amount: 0.0,
                        },
                    ))
                })
                .min_by(|(a, left), (b, right)| {
                    a.total_cmp(b).then_with(|| left.stream.cmp(&right.stream))
                })
                .map(|(_, lift)| (dam, lift))
        })
        .collect();
    clarity.advance(&flowing, active, time.delta_secs());
}

/// Use the authored water palette. Vertex colour multiplies the muddy material,
/// so the original palette is the neutral tint before a dam starts working.
fn tint_water(
    clarity: Res<WaterClarity>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    surfaces: Query<(Entity, &ChildOf, &Mesh3d, &stubby_streams::WaterSurface)>,
    mut previous: Local<HashMap<Entity, Vec<[f32; 4]>>>,
) {
    let Some(meshes) = meshes.as_mut() else {
        return;
    };
    previous.retain(|entity, _| surfaces.contains(*entity));
    for (entity, parent, mesh, surface) in &surfaces {
        let parent = parent.parent();
        if !clarity.profiles.contains_key(&parent) && !previous.contains_key(&entity) {
            continue;
        }
        let colours: Vec<_> = surface
            .0
            .iter()
            .map(|&position| water_tint(clarity.along(parent, position).unwrap_or_default()))
            .collect();
        if previous.get(&entity) != Some(&colours)
            && let Some(mut mesh) = meshes.get_mut(&mesh.0)
        {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours.clone());
            previous.insert(entity, colours);
        }
        if !clarity.profiles.contains_key(&parent) {
            previous.remove(&entity);
        }
    }
}

fn water_tint(clarity: f32) -> [f32; 4] {
    use serde::Deserialize;
    use std::sync::LazyLock;
    #[derive(Deserialize)]
    struct Profile {
        clarity: Palette,
    }
    #[derive(Deserialize)]
    struct Palette {
        stages: Vec<Stage>,
    }
    #[derive(Deserialize)]
    struct Stage {
        base_color: String,
        alpha: f32,
    }
    static TINTS: LazyLock<Vec<Vec4>> = LazyLock::new(|| {
        let profile: Profile = serde_json::from_str(include_str!(
            "../../../assets/streams/stream_segment/asset.json"
        ))
        .expect("stream clarity palette");
        let colours: Vec<_> = profile
            .clarity
            .stages
            .iter()
            .map(|stage| {
                let colour =
                    LinearRgba::from(Srgba::hex(&stage.base_color).expect("stream clarity colour"));
                Vec4::new(colour.red, colour.green, colour.blue, stage.alpha)
            })
            .collect();
        colours.iter().map(|colour| *colour / colours[0]).collect()
    });
    let stage = clarity * (TINTS.len() - 1) as f32;
    let lower = stage.floor() as usize;
    let higher = (lower + 1).min(TINTS.len() - 1);
    TINTS[lower].lerp(TINTS[higher], stage.fract()).to_array()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn only_clocked_tuned_patches_clean_downstream_and_removed_dams_ease_out() {
        let mut app = App::new();
        app.init_resource::<Time<Virtual>>()
            .add_plugins(PurifierPlugin);
        let world = app.world_mut();
        let burbler = world
            .spawn(MachineVoice {
                output: None,
                sounding: true,
                hearing_radius_metres: 20.0,
            })
            .id();
        let first = world
            .spawn(LaidStream::straight(
                Some(burbler),
                Vec3::ZERO,
                Vec3::X * 8.0,
            ))
            .id();
        let mut extension = LaidStream::straight(Some(burbler), Vec3::X * 8.0, Vec3::X * 16.0);
        extension.upstream = Some(PortOwner::Stream(first));
        extension.start = 8.0;
        let second = world.spawn(extension).id();
        let sibling = world
            .spawn(LaidStream::straight(
                Some(burbler),
                Vec3::Z * 8.0,
                Vec3::new(8.0, 0.0, 8.0),
            ))
            .id();

        let clock = AssetClock {
            engine: stubby_audio::ModuleId(1),
            clock: 1,
            bpm: 120.0,
        };
        let mut dam = |x: f32, sounding, met| {
            world
                .spawn((
                    PlacedAsset {
                        asset: PURIFIER.into(),
                        model: String::new(),
                        instance: x.to_string(),
                    },
                    GlobalTransform::from_translation(Vec3::X * x),
                    RackTargets {
                        sounding,
                        met,
                        missed: vec![],
                    },
                ))
                .id()
        };
        let low = dam(2.0, false, true);
        let middle = dam(5.0, true, false);
        let high = dam(12.0, true, true);
        world.entity_mut(low).insert(clock);
        world.entity_mut(middle).insert(clock);

        fn read(app: &App, stream: Entity, point: Vec3) -> f32 {
            let world = app.world();
            world.resource::<WaterClarity>().at(
                stream,
                world.get::<LaidStream>(stream).unwrap(),
                point,
            )
        }
        fn advance(app: &mut App, seconds: f32) {
            app.world_mut()
                .resource_mut::<Time<Virtual>>()
                .advance_by(Duration::from_secs_f32(seconds));
            app.update();
        }

        advance(&mut app, 1.0);
        assert_eq!(read(&app, first, Vec3::X * 7.0), 0.0);
        assert_eq!(read(&app, second, Vec3::X * 15.0), 0.0);

        let world = app.world_mut();
        world.get_mut::<RackTargets>(low).unwrap().sounding = true;
        world.get_mut::<RackTargets>(middle).unwrap().met = true;
        world.entity_mut(high).insert(clock);
        advance(&mut app, 1.0);
        assert_eq!(read(&app, first, Vec3::X), 0.0);
        assert_eq!(read(&app, first, Vec3::X * 3.0), 1.0 / 3.0);
        assert_eq!(read(&app, first, Vec3::X * 7.0), 2.0 / 3.0);
        assert_eq!(read(&app, second, Vec3::X * 10.0), 2.0 / 3.0);
        assert_eq!(read(&app, second, Vec3::X * 15.0), 1.0);
        assert_eq!(read(&app, sibling, Vec3::new(7.0, 0.0, 8.0)), 0.0);

        app.world_mut().despawn(middle);
        advance(&mut app, 0.5);
        assert_eq!(read(&app, second, Vec3::X * 15.0), 2.5 / 3.0);
        advance(&mut app, 0.5);
        assert_eq!(read(&app, first, Vec3::X * 3.0), 1.0 / 3.0);
        assert_eq!(read(&app, second, Vec3::X * 15.0), 2.0 / 3.0);
    }
}
