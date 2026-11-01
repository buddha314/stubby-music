//! Proximity interaction and the shared, model-backed machine rack HUD.
use super::{
    controls::{Screen, ScreenMode},
    prefab_presentation::AssetUsed,
    prefabs::{AssetClock, PlacedAsset, PrefabBinding, PrefabError},
    resources::ResourceNode,
    synth_modules::{ModuleLights, ModuleParams, ModuleView, SynthModule},
};
use bevy::{
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
};
use serde::Deserialize;
use std::collections::HashMap;

const HUD_LAYER: usize = 31;
const HUD_Y: f32 = -1000.;
const HP: f32 = 0.0508;

#[derive(Clone, Deserialize)]
pub struct RackConfiguration {
    pub title: String,
    pub hud: String,
    pub interact: InteractionProfile,
    pub places: Vec<RackPlace>,
}
#[derive(Clone, Deserialize)]
pub struct InteractionProfile {
    pub node: String,
    pub range_m: f32,
    pub prompt: String,
}
#[derive(Clone, Deserialize)]
pub struct RackPlace {
    pub kind: String,
    pub hp: u32,
    pub fixed: bool,
    pub module: Option<String>,
}
#[derive(Resource)]
struct Profiles(HashMap<String, RackConfiguration>);
impl Default for Profiles {
    fn default() -> Self {
        #[derive(Deserialize)]
        struct Profile {
            id: String,
            configure: RackConfiguration,
        }
        let profile: Profile = serde_json::from_str(include_str!(
            "../../../assets/machines/mudpit_burbler/asset.json"
        ))
        .expect("machine configuration profile");
        Self([(profile.id, profile.configure)].into())
    }
}
#[derive(Resource, Default)]
pub struct MachineInteraction {
    pub target: Option<Entity>,
    pub prompt: String,
}
/// Hook for other machine/configuration systems; emitted once when E opens a rack.
#[derive(Message)]
pub struct MachineConfigurationOpened {
    pub machine: Entity,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MachineInteractionSet;
#[derive(Component)]
pub struct RackHudCamera;
#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Prompt;
#[derive(Component)]
struct Header;
#[derive(Component)]
struct HudPart;
#[derive(Component)]
struct HudScene {
    kind: SceneKind,
    width: f32,
}
enum SceneKind {
    Frame,
    Module,
    Label(String),
}
#[derive(Resource)]
struct View {
    camera: Entity,
    machine: Option<Entity>,
    scene: Option<Entity>,
}

pub struct MachineConfigurationPlugin;
impl Plugin for MachineConfigurationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Profiles>()
            .init_resource::<MachineInteraction>()
            .init_resource::<ScreenMode>()
            .add_message::<MachineConfigurationOpened>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                interact
                    .in_set(MachineInteractionSet)
                    .after(PrefabBinding)
                    .after(super::controls::toggle_screens)
                    .before(super::controls::capture_cursor),
            )
            .add_systems(
                Update,
                (sync_view, prepare_parts)
                    .chain()
                    .after(MachineInteractionSet),
            );
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn interact(
    keys: Res<ButtonInput<KeyCode>>,
    profiles: Res<Profiles>,
    mut state: ResMut<MachineInteraction>,
    mut screens: ResMut<ScreenMode>,
    cameras: Query<Entity, (With<Camera3d>, Without<RackHudCamera>)>,
    machines: Query<(Entity, &PlacedAsset), Without<PrefabError>>,
    names: Query<(Entity, &Name)>,
    parents: Query<&ChildOf>,
    transforms: bevy::transform::helper::TransformHelper,
    pickups: Query<(Entity, &ResourceNode), Without<PrefabError>>,
    mut opened: MessageWriter<MachineConfigurationOpened>,
    mut used: MessageWriter<AssetUsed>,
) {
    state.target = None;
    state.prompt.clear();
    if let Some(Screen::MachineConfiguration(entity)) = screens.open {
        if !machines.contains(entity) {
            screens.close();
        }
        return;
    }
    if screens.open.is_some() {
        return;
    }
    let Some(camera) = cameras
        .single()
        .ok()
        .and_then(|e| transforms.compute_global_transform(e).ok())
    else {
        return;
    };
    let distance = |position: Vec3| {
        let d = position - camera.translation();
        Vec2::new(d.x, d.z).length_squared()
    };
    let nearest_pickup = pickups
        .iter()
        .filter(|(_, node)| !node.collected)
        .filter_map(|(e, _)| {
            transforms
                .compute_global_transform(e)
                .ok()
                .map(|t| distance(t.translation()))
        })
        .fold(f32::INFINITY, f32::min);
    let mut nearest = f32::INFINITY;
    for (point, name) in &names {
        let Some(machine) = parents
            .iter_ancestors(point)
            .find(|e| machines.contains(*e))
        else {
            continue;
        };
        let Ok((_, asset)) = machines.get(machine) else {
            continue;
        };
        let Some(config) = profiles.0.get(&asset.asset) else {
            continue;
        };
        if name.as_str() != config.interact.node {
            continue;
        }
        let Ok(at) = transforms.compute_global_transform(point) else {
            continue;
        };
        let d = distance(at.translation());
        if d <= config.interact.range_m.powi(2) && d < nearest && d < nearest_pickup {
            nearest = d;
            state.target = Some(machine);
            state.prompt.clone_from(&config.interact.prompt);
        }
    }
    if keys.just_pressed(super::controls::INTERACT)
        && let Some(machine) = state.target
    {
        screens.open = Some(Screen::MachineConfiguration(machine));
        opened.write(MachineConfigurationOpened { machine });
        if let Ok((_, asset)) = machines.get(machine) {
            used.write(AssetUsed {
                entity: machine,
                instance: asset.instance.clone(),
                asset: asset.asset.clone(),
            });
        }
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_target_texture(
        1024,
        768,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));
    let camera = commands
        .spawn((
            RackHudCamera,
            Camera3d::default(),
            Camera {
                order: -1,
                is_active: false,
                clear_color: Color::srgb(0.082, 0.086, 0.102).into(),
                ..default()
            },
            RenderTarget::Image(image.clone().into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 2.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_xyz(0., HUD_Y + 0.68, 4.)
                .looking_at(Vec3::new(0., HUD_Y + 0.68, 0.), Vec3::Y),
            RenderLayers::layer(HUD_LAYER),
            AmbientLight {
                brightness: 200.,
                ..default()
            },
            bevy::camera::Exposure { ev100: 6.5 },
        ))
        .id();
    commands.spawn((
        PointLight {
            intensity: 26000.,
            color: Color::srgb(1., 0.78, 0.6),
            range: 20.,
            ..default()
        },
        Transform::from_xyz(-1.2, HUD_Y + 1.9, 2.6),
        RenderLayers::layer(HUD_LAYER),
    ));
    commands.spawn((
        Prompt,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(20.),
            ..default()
        },
        TextColor(Color::srgb(1., 0.7, 0.31)),
        Node {
            position_type: PositionType::Absolute,
            bottom: percent(24),
            left: percent(35),
            ..default()
        },
    ));
    commands
        .spawn((
            Panel,
            Visibility::Hidden,
            GlobalZIndex(20),
            BackgroundColor(Color::srgba(0., 0., 0., 0.55)),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                BackgroundColor(Color::srgb(0.082, 0.086, 0.102)),
                BorderColor::all(Color::srgba(1., 0.7, 0.31, 0.4)),
                Node {
                    width: percent(88),
                    height: percent(90),
                    max_width: px(960),
                    padding: UiRect::all(px(18)),
                    border: UiRect::all(px(1)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Header,
                    Text::new(""),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(24.),
                        ..default()
                    },
                    TextColor(Color::srgb(1., 0.7, 0.31)),
                ));
                panel.spawn((
                    ImageNode::new(image),
                    Node {
                        width: percent(100),
                        flex_grow: 1.,
                        min_height: px(0),
                        ..default()
                    },
                ));
                panel.spawn(Text::new("Esc  close"));
            });
        });
    commands.insert_resource(View {
        camera,
        machine: None,
        scene: None,
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_view(
    mut commands: Commands,
    server: Res<AssetServer>,
    profiles: Res<Profiles>,
    screens: Res<ScreenMode>,
    interaction: Res<MachineInteraction>,
    mut view: ResMut<View>,
    machines: Query<(&PlacedAsset, Option<&AssetClock>)>,
    mut cameras: Query<&mut Camera, With<RackHudCamera>>,
    mut panels: Query<&mut Visibility, With<Panel>>,
    mut prompts: Query<&mut Text, (With<Prompt>, Without<Header>)>,
    mut headers: Query<&mut Text, (With<Header>, Without<Prompt>)>,
) {
    let machine = match screens.open {
        Some(Screen::MachineConfiguration(e)) if machines.contains(e) => Some(e),
        _ => None,
    };
    for mut panel in &mut panels {
        *panel = if machine.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut camera) = cameras.get_mut(view.camera) {
        camera.is_active = machine.is_some();
    }
    for mut prompt in &mut prompts {
        prompt.0 = if screens.open.is_none() {
            interaction.prompt.clone()
        } else {
            String::new()
        };
    }
    if let Some(entity) = machine
        && let Ok((asset, clock)) = machines.get(entity)
        && let Some(config) = profiles.0.get(&asset.asset)
    {
        for mut header in &mut headers {
            header.0 = clock.map_or_else(
                || config.title.clone(),
                |clock| format!("{}  ·  {:.0} BPM", config.title, clock.bpm),
            );
        }
    }
    if view.machine == machine {
        return;
    }
    if let Some(scene) = view.scene.take() {
        commands.entity(scene).despawn();
    }
    view.machine = machine;
    let Some(entity) = machine else {
        return;
    };
    let Ok((asset, clock)) = machines.get(entity) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    let width: f32 = config.places.iter().map(|place| place.hp as f32 * HP).sum();
    let root = commands
        .spawn((Transform::from_xyz(0., HUD_Y, 0.), Visibility::Visible))
        .id();
    view.scene = Some(root);
    commands.spawn((
        WorldAssetRoot(server.load(if config.hud.contains('#') {
            config.hud.clone()
        } else {
            format!("{}#Scene0", config.hud)
        })),
        Transform::default(),
        Visibility::Inherited,
        HudScene {
            kind: SceneKind::Frame,
            width,
        },
        ChildOf(root),
    ));
    let mut x = -width / 2.;
    for place in &config.places {
        let span = place.hp as f32 * HP;
        let center = x + span / 2.;
        x += span;
        if place.fixed
            && let Some(module) = &place.module
        {
            let mut part = commands.spawn((
                WorldAssetRoot(server.load(format!("models/modules/{module}.glb#Scene0"))),
                Transform::from_xyz(center, 0., 0.),
                Visibility::Inherited,
                HudScene {
                    kind: SceneKind::Module,
                    width,
                },
                ChildOf(root),
            ));
            if let Some(clock) = clock {
                part.insert((
                    ModuleView,
                    SynthModule {
                        id: clock.engine,
                        asset: module.clone(),
                    },
                    ModuleParams([("bpm".into(), clock.bpm)].into()),
                    ModuleLights::default(),
                ));
            }
        } else {
            commands.spawn((
                WorldAssetRoot(
                    server.load(format!("models/hud/slot_outline_{}hp.glb#Scene0", place.hp)),
                ),
                Transform::from_xyz(center, 0., 0.),
                Visibility::Inherited,
                HudScene {
                    kind: SceneKind::Module,
                    width,
                },
                ChildOf(root),
            ));
            commands.spawn((
                WorldAssetRoot(server.load("models/hud/slot_labels.glb#Scene0")),
                Transform::from_xyz(center, 1.375 * 0.56, 0.18),
                Visibility::Inherited,
                HudScene {
                    kind: SceneKind::Label(format!("Label_{}", place.kind)),
                    width,
                },
                ChildOf(root),
            ));
        }
    }
}

/// Assign native render layers to imported descendants; never rebuild model geometry.
fn prepare_parts(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    scenes: Query<&HudScene>,
    mut parts: Query<(Entity, Option<&Name>, Option<&mut Transform>), Without<HudPart>>,
) {
    for (entity, name, transform) in &mut parts {
        let Some(scene) = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| scenes.get(ancestor).ok())
        else {
            continue;
        };
        commands
            .entity(entity)
            .insert((HudPart, RenderLayers::layer(HUD_LAYER)));
        let Some(name) = name else {
            continue;
        };
        match &scene.kind {
            SceneKind::Frame => {
                if let Some(mut transform) = transform {
                    match name.as_str() {
                        "RailTop" | "RailBottom" => transform.scale.x = scene.width + 0.12,
                        "PostL" => transform.translation.x = -(scene.width / 2. + 0.1),
                        "PostR" => transform.translation.x = scene.width / 2. + 0.1,
                        _ => (),
                    }
                }
            }
            SceneKind::Label(selected)
                if name.as_str().starts_with("Label_") && name.as_str() != selected =>
            {
                commands.entity(entity).insert(Visibility::Hidden);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interact_point_opens_rack_escape_closes_and_nearer_pickup_wins() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Profiles>()
            .init_resource::<MachineInteraction>()
            .init_resource::<ScreenMode>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<AssetUsed>()
            .add_message::<MachineConfigurationOpened>()
            .add_systems(
                Update,
                (super::super::controls::toggle_screens, interact).chain(),
            );
        let machine = app
            .world_mut()
            .spawn((
                PlacedAsset {
                    asset: "mudpit_burbler".into(),
                    model: "burbler.glb".into(),
                    instance: "test-pit".into(),
                },
                Transform::from_xyz(100., 0., 0.),
            ))
            .id();
        app.world_mut().spawn((
            Name::new("Interact"),
            Transform::from_xyz(-10., 20., 0.),
            ChildOf(machine),
        ));
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(88., 1.65, 0.)))
            .id();
        app.update();
        assert_eq!(
            app.world().resource::<MachineInteraction>().target,
            Some(machine)
        );
        assert_eq!(
            app.world().resource::<MachineInteraction>().prompt,
            "E | Configure burbler"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world().resource::<ScreenMode>().open,
            Some(Screen::MachineConfiguration(machine))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(app.world().resource::<ScreenMode>().open, None);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let pickup = app
            .world_mut()
            .spawn((
                ResourceNode {
                    resource: "lightning_bug".into(),
                    instance: "nearer".into(),
                    collected: false,
                },
                Transform::from_xyz(88.5, 5., 0.),
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(app.world().resource::<MachineInteraction>().target, None);
        assert_eq!(app.world().resource::<ScreenMode>().open, None);
        app.world_mut()
            .get_mut::<ResourceNode>(pickup)
            .unwrap()
            .collected = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation
            .x = 100.;
        app.update();
        assert_eq!(
            app.world().resource::<MachineInteraction>().target,
            None,
            "standing near the model root is not standing near Interact"
        );
    }
}
