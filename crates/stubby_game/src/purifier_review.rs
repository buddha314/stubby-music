//! An opt-in standalone setup that leaves the purifier's rack for the player to patch.
use crate::purifier::PURIFIER;
use bevy::{prelude::*, world_serialization::WorldAssetRoot};
use bevy_stubby_rack::{machine_config::Profiles, rack::RackState};
use bevy_stubby_recipes::PlayerInventory;
use stubby_core::{AssetClock, AssetPart, AssetState, HudCamera, PlacedAsset};
use stubby_streams::{LaidStream, PORT_HEIGHT, Port, PortOwner, StreamBuild, StreamSystems};

const MODEL: &str = "machines/reed_water_purifier/reed_water_purifier.glb";

pub struct PurifierReviewPlugin;
impl Plugin for PurifierReviewPlugin {
    fn build(&self, app: &mut App) {
        if std::env::args().any(|arg| arg == "--purifier-review") {
            app.init_resource::<Setup>().add_systems(
                Update,
                prepare
                    .after(StreamSystems::Input)
                    .before(StreamSystems::Plan),
            );
        }
    }
}

#[derive(Default, Resource)]
enum Setup {
    #[default]
    FindOutlet,
    Lay(Port),
    WaitForDam(Entity),
    Ready,
}

fn prepare(
    mut commands: Commands,
    mut setup: ResMut<Setup>,
    mut build: ResMut<StreamBuild>,
    mut inventory: ResMut<PlayerInventory>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<HudCamera>)>,
    profiles: Res<Profiles>,
    mut burblers: Query<(&PlacedAsset, &AssetState, &mut RackState)>,
    outlets: Query<(&AssetPart, &GlobalTransform)>,
    streams: Query<&LaidStream>,
    clocks: Query<&AssetClock>,
    server: Res<AssetServer>,
    mut calendar: ResMut<stubby_world::calendar::Calendar>,
) {
    if matches!(*setup, Setup::Ready) {
        return;
    }
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    match *setup {
        Setup::FindOutlet => {
            let candidate = outlets
                .iter()
                .filter(|(part, _)| part.name == "Outlet_0")
                .filter_map(|(part, transform)| {
                    let (asset, state, _) = burblers.get(part.root).ok()?;
                    let at = transform.translation();
                    let distance = Vec2::new(at.x, at.z).distance(stubby_core::OPENING_SPAWN_XZ);
                    // Use the outdoor opening burbler. The grotto is farther from spawn.
                    (asset.asset == "mudpit_burbler" && !state.stream_connected && distance < 60.0)
                        .then_some((distance, asset.instance.as_str(), part.root, transform))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)));
            let Some((_, _, source, outlet)) = candidate else {
                return;
            };
            let Ok((_, _, mut rack)) = burblers.get_mut(source) else {
                return;
            };
            let config = &profiles.0["mudpit_burbler"];
            rack.mounted.insert("vco".into(), "square_vco".into());
            rack.own = config
                .cables
                .iter()
                .map(|cable| (cable.from.clone(), cable.to.clone()))
                .collect();
            rack.version += 1;
            let _ = inventory.items.grant(&[stubby_recipes::Stack {
                item: "reed".into(),
                quantity: 18,
            }]);
            let needed = 6_u32.saturating_sub(inventory.items.quantity("patch_cable"));
            if needed > 0 {
                let _ = inventory.items.grant(&[stubby_recipes::Stack {
                    item: "patch_cable".into(),
                    quantity: needed,
                }]);
            }
            calendar.elapsed = std::time::Duration::from_secs(1350);
            let right = outlet.right().as_vec3();
            let port = Port {
                at: outlet.translation() - Vec3::Y * PORT_HEIGHT,
                dir: Vec3::new(right.x, 0.0, right.z).normalize_or(Vec3::X),
                owner: PortOwner::Burbler(source),
            };
            *setup = Setup::Lay(port);
        }
        Setup::Lay(port) => {
            if let Some(stream) = streams
                .iter()
                .find(|stream| stream.upstream == Some(port.owner))
            {
                build.active = false;
                build.anchor = None;
                let (point, flow) = stream.centre_at(stream.span.min(stream.length() * 0.5));
                let side = flow.cross(Vec3::Y);
                let dam = commands
                    .spawn((
                        Name::new("Purifier review dam"),
                        Transform::from_translation(point)
                            .with_rotation(Quat::from_mat3(&Mat3::from_cols(flow, Vec3::Y, side))),
                        Visibility::default(),
                        WorldAssetRoot(server.load(format!("{MODEL}#Scene0"))),
                        PlacedAsset {
                            asset: PURIFIER.into(),
                            model: MODEL.into(),
                            instance: "purifier-review".into(),
                        },
                    ))
                    .id();
                *camera = Transform::from_translation(point + side * 2.4 + Vec3::Y * 1.65)
                    .looking_at(point, Vec3::Y);
                *setup = Setup::WaitForDam(dam);
            } else {
                let side = port.dir.cross(Vec3::Y);
                *camera = Transform::from_translation(port.at + side * 3.0 + Vec3::Y * 1.65)
                    .looking_at(port.at + port.dir * 10.0, Vec3::Y);
                build.active = true;
                build.anchor = Some(port);
                build.click = build.plan.as_ref().is_some_and(|plan| plan.valid);
            }
        }
        Setup::WaitForDam(dam) => {
            if let Ok(clock) = clocks.get(dam) {
                info!(
                    "Purifier review: unpatched rack with a {:.0} BPM clock. Press E to configure.",
                    clock.bpm
                );
                *setup = Setup::Ready;
            }
        }
        Setup::Ready => {}
    }
}
