//! Proximity interaction and the shared, model-backed machine rack HUD.
use super::{
    controls::{Screen, ScreenMode},
    prefab_presentation::AssetUsed,
    prefabs::{AssetClock, PlacedAsset, PrefabBinding, PrefabError},
    synth_modules::{ModuleLights, ModuleParams, ModuleView, SynthModule},
};
use bevy::{
    app::{HierarchyPropagatePlugin, Propagate},
    camera::{ScalingMode, Viewport, visibility::RenderLayers},
    picking::{
        Pickable,
        mesh_picking::{MeshPickingCamera, MeshPickingPlugin, MeshPickingSettings},
    },
    prelude::*,
    window::PrimaryWindow,
};
use serde::Deserialize;
use std::collections::HashMap;

pub(crate) const HUD_LAYER: usize = 31;
const HUD_Y: f32 = -1000.;
const HP: f32 = 0.0508;

#[derive(Clone, Deserialize)]
pub struct RackConfiguration {
    pub title: String,
    pub hud: String,
    pub interact: InteractionProfile,
    pub places: Vec<RackPlace>,
    /// The voice's output, normalled to the machine's outlet.
    #[serde(default)]
    pub normalled_output: Option<JackRef>,
    /// The spare cables on the comb, each for one patch the voice needs.
    #[serde(default)]
    pub cables: Vec<RackCable>,
    #[serde(default)]
    pub patched_hint: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct JackRef {
    pub place: String,
    pub port: String,
}
#[derive(Clone, Deserialize)]
pub struct RackCable {
    pub color: String,
    pub from: JackRef,
    pub to: JackRef,
    pub hint: String,
}
#[derive(Clone, Deserialize)]
pub struct Accept {
    pub item: String,
    pub module: String,
}
#[derive(Clone, Deserialize)]
pub struct InteractionProfile {
    pub node: String,
    pub range_m: f32,
    pub prompt: String,
}
#[derive(Clone, Deserialize)]
pub struct RackPlace {
    pub place: String,
    pub kind: String,
    pub hp: u32,
    pub fixed: bool,
    pub module: Option<String>,
    /// Items the player can mount here, each becoming a module.
    #[serde(default)]
    pub accepts: Vec<Accept>,
}
#[derive(Resource)]
pub(crate) struct Profiles(pub HashMap<String, RackConfiguration>);
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
struct Header;
#[derive(Component)]
struct HudPart;
/// The panel's guidance line (filled by `rack`).
#[derive(Component)]
pub(crate) struct RackHint;
/// Where `rack` lists the relevant items (the window's left column).
#[derive(Component)]
pub(crate) struct RackActions;
/// Where `rack` lays out the player's inventory boxes (the window's right column).
#[derive(Component)]
pub(crate) struct RackInventory;

/// The configuration window, Satisfactory's craft-bench layout, in fractions of the
/// screen: a framed window with a title bar; the relevant items on the left, the rack in
/// the middle panel, the inventory boxes on the right. `rack_viewport` fits the rack
/// camera to `CENTRE`.
mod layout {
    /// (left, top, width, height)
    pub const WINDOW: (f32, f32, f32, f32) = (0.06, 0.08, 0.88, 0.80);
    pub const TITLE_H: f32 = 0.055;
    pub const LEFT: (f32, f32, f32, f32) = (0.07, 0.15, 0.19, 0.71);
    pub const CENTRE: (f32, f32, f32, f32) = (0.27, 0.15, 0.40, 0.71);
    pub const RIGHT: (f32, f32, f32, f32) = (0.68, 0.15, 0.25, 0.71);
}
#[derive(Component)]
struct HudScene {
    kind: SceneKind,
    width: f32,
    place: Option<String>,
}
/// A jack's mesh on the rack screen, pickable for patching.
#[derive(Component, Clone)]
pub(crate) struct JackHit(pub JackRef);
/// A jack's halo mesh (its patch pod).
#[derive(Component, Clone)]
pub(crate) struct PodMesh(pub JackRef);
/// A jack's cable socket (`Port_<port>`) on the rack screen.
#[derive(Component, Clone)]
pub(crate) struct PortNode(pub JackRef);
/// A node of the rack frame's cable comb (`Comb_<n>`, `Comb_<n>_PlugA/B`).
#[derive(Component, Clone)]
pub(crate) struct CombNode(pub String);
/// An empty place's slot outline mesh on the rack screen: a drop target for modules.
#[derive(Component, Clone)]
pub(crate) struct SlotHit(pub String);
enum SceneKind {
    Frame,
    Module,
    /// An empty place's outline (its place).
    Slot(String),
    Label(String),
}
#[derive(Resource)]
struct View {
    camera: Entity,
    machine: Option<Entity>,
    version: u32,
    scene: Option<Entity>,
}

pub struct MachineConfigurationPlugin;
impl Plugin for MachineConfigurationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MeshPickingPlugin>() {
            app.add_plugins(MeshPickingPlugin)
                .insert_resource(MeshPickingSettings {
                    require_markers: true,
                    ..default()
                });
        }
        app.add_plugins(HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate))
            .init_resource::<Profiles>()
            .init_resource::<MachineInteraction>()
            .init_resource::<ScreenMode>()
            .init_resource::<super::interaction::InteractTarget>()
            .add_message::<MachineConfigurationOpened>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                interact
                    .in_set(MachineInteractionSet)
                    .after(PrefabBinding)
                    .after(super::interaction::InteractionTargeting)
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

/// E opens the rack of the machine `interaction::InteractTarget` points at.
fn interact(
    keys: Res<ButtonInput<KeyCode>>,
    profiles: Res<Profiles>,
    target: Res<super::interaction::InteractTarget>,
    mut state: ResMut<MachineInteraction>,
    mut screens: ResMut<ScreenMode>,
    machines: Query<&PlacedAsset, Without<PrefabError>>,
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
    let Some((machine, asset)) = target
        .machine()
        .and_then(|m| machines.get(m).ok().map(|a| (m, a)))
    else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    state.target = Some(machine);
    state.prompt.clone_from(&config.interact.prompt);
    if keys.just_pressed(super::controls::INTERACT) {
        screens.open = Some(Screen::MachineConfiguration(machine));
        opened.write(MachineConfigurationOpened { machine });
        used.write(AssetUsed {
            entity: machine,
            instance: asset.instance.clone(),
            asset: asset.asset.clone(),
        });
    }
}

fn setup(mut commands: Commands) {
    // The rack draws straight onto the window, over the panel, through its own camera
    // and layer; its viewport follows the panel (`sync_view`). Drawing to the window
    // rather than an image lets mesh picking reach the jacks.
    let camera = commands
        .spawn((
            RackHudCamera,
            Camera3d::default(),
            Camera {
                order: 1,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.082, 0.086, 0.102)),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 2.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_xyz(0., HUD_Y + 0.68, 4.)
                .looking_at(Vec3::new(0., HUD_Y + 0.68, 0.), Vec3::Y),
            RenderLayers::layer(HUD_LAYER),
            MeshPickingCamera,
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
    let text = |size: f32, color: Color| {
        (
            TextFont {
                font_size: bevy::text::FontSize::Px(size),
                ..default()
            },
            TextColor(color),
        )
    };
    commands
        .spawn((
            Panel,
            Visibility::Hidden,
            GlobalZIndex(20),
            // The backdrop must not catch the pointer: the jacks drawn by the rack camera
            // sit under it. Only the buttons in `RackActions` take clicks.
            Pickable::IGNORE,
            BackgroundColor(Color::srgba(0., 0., 0., 0.45)),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|panel| {
            let at = |(l, t, w, h): (f32, f32, f32, f32)| Node {
                position_type: PositionType::Absolute,
                left: percent(l * 100.),
                top: percent(t * 100.),
                width: percent(w * 100.),
                height: percent(h * 100.),
                ..default()
            };
            let frame = Color::srgb(0.42, 0.29, 0.13);
            // The window and its title bar.
            panel.spawn((
                Node {
                    border: UiRect::all(px(1)),
                    ..at(layout::WINDOW)
                },
                BackgroundColor(Color::srgba(0.07, 0.065, 0.06, 0.97)),
                BorderColor::all(frame),
                Pickable::IGNORE,
            ));
            let (l, t, w, _) = layout::WINDOW;
            panel
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..at((l, t, w, layout::TITLE_H))
                    },
                    BackgroundColor(Color::srgba(0.11, 0.1, 0.09, 1.)),
                    Pickable::IGNORE,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Header,
                        Text::new(""),
                        text(20., Color::srgb(0.92, 0.9, 0.85)),
                        Pickable::IGNORE,
                    ));
                    bar.spawn((
                        Text::new("X"),
                        text(22., Color::srgb(0.8, 0.78, 0.74)),
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(16),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
            // The middle panel, behind the rack camera's view.
            panel.spawn((
                Node {
                    border: UiRect::all(px(2)),
                    ..at(layout::CENTRE)
                },
                BackgroundColor(Color::srgb(0.13, 0.12, 0.11)),
                BorderColor::all(Color::srgb(0.3, 0.29, 0.27)),
                Pickable::IGNORE,
            ));
            panel.spawn((
                RackActions,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    overflow: Overflow::scroll_y(),
                    ..at(layout::LEFT)
                },
                Pickable::IGNORE,
            ));
            panel.spawn((
                RackInventory,
                Node {
                    // Four boxes a row: the 16-box inventory as 4 x 4.
                    max_width: px(4. * 64. + 3. * 6. + 2.),
                    flex_wrap: FlexWrap::Wrap,
                    align_content: AlignContent::FlexStart,
                    column_gap: px(6),
                    row_gap: px(6),
                    ..at(layout::RIGHT)
                },
                Pickable::IGNORE,
            ));
            // The guidance line stays (empty: the screen shows, it does not tell).
            panel.spawn((
                RackHint,
                Text::new(""),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
            // Key prompts under the window, as Satisfactory shows them.
            panel
                .spawn((
                    Node {
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..at((l, t + layout::WINDOW.3 + 0.012, w, 0.04))
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|keys| {
                    keys.spawn((
                        Node {
                            padding: UiRect::axes(px(8), px(3)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.85, 0.84, 0.8)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|key| {
                        key.spawn((
                            Text::new("Esc"),
                            text(15., Color::srgb(0.1, 0.1, 0.1)),
                            Pickable::IGNORE,
                        ));
                    });
                    keys.spawn((
                        Text::new("Close"),
                        text(15., Color::srgb(0.85, 0.84, 0.8)),
                        Pickable::IGNORE,
                    ));
                });
        });
    commands.insert_resource(View {
        camera,
        machine: None,
        version: 0,
        scene: None,
    });
}

/// The rack's area on screen: the window's middle panel, in what the game draws into
/// (the window when run alone, an image when Jackdaw's Play hosts it), at the rack's
/// aspect, centred in the panel.
fn rack_viewport(target: UVec2) -> Viewport {
    let (w, h) = (target.x as f32, target.y as f32);
    let (l, t, cw, ch) = layout::CENTRE;
    let (pw, ph) = (cw * w - 8., ch * h - 8.);
    let (vw, vh) = if pw / ph > 1.25 {
        (ph * 1.25, ph)
    } else {
        (pw, pw / 1.25)
    };
    Viewport {
        physical_position: UVec2::new(
            (l * w + (cw * w - vw) / 2.) as u32,
            (t * h + (ch * h - vh) / 2.) as u32,
        ),
        physical_size: UVec2::new(vw.max(1.) as u32, vh.max(1.) as u32),
        ..default()
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_view(
    mut commands: Commands,
    server: Res<AssetServer>,
    profiles: Res<Profiles>,
    screens: Res<ScreenMode>,
    mut view: ResMut<View>,
    machines: Query<(
        &PlacedAsset,
        Option<&AssetClock>,
        Option<&super::rack::RackState>,
    )>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<RackHudCamera>>,
    mut panels: Query<&mut Visibility, With<Panel>>,
    mut headers: Query<&mut Text, With<Header>>,
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
        if let Some(size) = camera.physical_target_size().or_else(|| {
            windows
                .single()
                .ok()
                .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
        }) {
            camera.viewport = Some(rack_viewport(size));
        }
    }
    if let Some(entity) = machine
        && let Ok((asset, clock, _)) = machines.get(entity)
        && let Some(config) = profiles.0.get(&asset.asset)
    {
        for mut header in &mut headers {
            header.0 = clock.map_or_else(
                || config.title.clone(),
                |clock| format!("{}  |  {:.0} BPM", config.title, clock.bpm),
            );
        }
    }
    let version = machine
        .and_then(|e| machines.get(e).ok())
        .and_then(|(_, _, rack)| rack.map(|r| r.version))
        .unwrap_or(0);
    if view.machine == machine && view.version == version {
        return;
    }
    if let Some(scene) = view.scene.take() {
        commands.entity(scene).despawn();
    }
    view.machine = machine;
    view.version = version;
    let Some(entity) = machine else {
        return;
    };
    let Ok((asset, clock, rack)) = machines.get(entity) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    let width: f32 = config.places.iter().map(|place| place.hp as f32 * HP).sum();
    let root = commands
        .spawn((
            Transform::from_xyz(0., HUD_Y, 0.),
            Visibility::Visible,
            Propagate(RenderLayers::layer(HUD_LAYER)),
        ))
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
            place: None,
        },
        ChildOf(root),
    ));
    let mut x = -width / 2.;
    for place in &config.places {
        let span = place.hp as f32 * HP;
        let center = x + span / 2.;
        x += span;
        let mounted = place
            .module
            .clone()
            .filter(|_| place.fixed)
            .or_else(|| rack.and_then(|r| r.mounted.get(&place.place).cloned()));
        if let Some(module) = &mounted {
            let mut part = commands.spawn((
                WorldAssetRoot(server.load(format!("models/modules/{module}.glb#Scene0"))),
                Transform::from_xyz(center, 0., 0.),
                Visibility::Inherited,
                HudScene {
                    kind: SceneKind::Module,
                    width,
                    place: Some(place.place.clone()),
                },
                ChildOf(root),
            ));
            if let Some(clock) = clock.filter(|_| module.as_str() == "burbler_clock") {
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
                    kind: SceneKind::Slot(place.place.clone()),
                    width,
                    place: None,
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
                    place: None,
                },
                ChildOf(root),
            ));
        }
    }
}

/// Tag imported parts the rack needs (jacks, pods, ports, the comb) and fit the frame;
/// render layers come from the root's `Propagate`. Never rebuilds model geometry.
fn prepare_parts(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    scenes: Query<&HudScene>,
    names: Query<&Name>,
    mut parts: Query<
        (Entity, Option<&Name>, Option<&mut Transform>, Has<Mesh3d>),
        Without<HudPart>,
    >,
) {
    for (entity, name, transform, is_mesh) in &mut parts {
        let Some(scene) = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| scenes.get(ancestor).ok())
        else {
            continue;
        };
        commands.entity(entity).insert(HudPart);
        // A mesh's own name, or its glTF node's (the primitive's parent).
        let own = name.map(Name::as_str);
        let node = own
            .filter(|_| !is_mesh)
            .or_else(|| {
                parents
                    .get(entity)
                    .ok()
                    .and_then(|p| names.get(p.parent()).ok())
                    .map(Name::as_str)
            })
            .or(own);
        if let (Some(place), Some(node)) = (&scene.place, node) {
            let jack = |port: &str| JackRef {
                place: place.clone(),
                port: port.into(),
            };
            let port = node.rsplit('_').next().unwrap_or_default();
            if is_mesh
                && ["_Nut_", "_Hole_", "_Bushing_"]
                    .iter()
                    .any(|k| node.contains(k))
            {
                commands
                    .entity(entity)
                    .insert((JackHit(jack(port)), Pickable::default()));
            } else if is_mesh && node.contains("_JackHalo") {
                commands.entity(entity).insert(PodMesh(jack(port)));
            } else if !is_mesh && let Some(port) = node.strip_prefix("Port_") {
                commands.entity(entity).insert(PortNode(jack(port)));
            }
        }
        let Some(name) = name else {
            continue;
        };
        match &scene.kind {
            SceneKind::Frame => {
                if name.as_str().starts_with("Comb_") {
                    commands
                        .entity(entity)
                        .insert(CombNode(name.as_str().into()));
                }
                if let Some(mut transform) = transform {
                    match name.as_str() {
                        "RailTop" | "RailBottom" => transform.scale.x = scene.width + 0.12,
                        "PostL" => transform.translation.x = -(scene.width / 2. + 0.1),
                        "PostR" => transform.translation.x = scene.width / 2. + 0.1,
                        _ => (),
                    }
                }
            }
            SceneKind::Slot(place) if is_mesh => {
                commands
                    .entity(entity)
                    .insert((SlotHit(place.clone()), Pickable::default()));
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
    fn e_on_a_targeted_machine_opens_its_rack_and_escape_closes_it() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Profiles>()
            .init_resource::<MachineInteraction>()
            .init_resource::<ScreenMode>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<super::super::interaction::InteractTarget>()
            .add_message::<AssetUsed>()
            .add_message::<MachineConfigurationOpened>()
            .add_systems(
                Update,
                (super::super::controls::toggle_screens, interact).chain(),
            );
        let machine = app
            .world_mut()
            .spawn(PlacedAsset {
                asset: "mudpit_burbler".into(),
                model: "burbler.glb".into(),
                instance: "test-pit".into(),
            })
            .id();
        *app.world_mut()
            .resource_mut::<super::super::interaction::InteractTarget>() =
            super::super::interaction::InteractTarget::Machine(machine);
        app.update();
        assert_eq!(
            app.world().resource::<MachineInteraction>().prompt,
            "Configure burbler"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world().resource::<ScreenMode>().open,
            Some(Screen::MachineConfiguration(machine))
        );
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.clear();
        keys.press(KeyCode::Escape);
        app.update();
        assert_eq!(app.world().resource::<ScreenMode>().open, None);
    }
}
