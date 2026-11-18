//! Building, as in Satisfactory: Q opens the build menu, a tile per buildable (from the
//! rules' `buildables.json`, with its icon and cost), and picking one starts placing it.
//! Streams are laid by `stubby_streams`; a campfire (add-campfire-craft-bench) is placed
//! here: its hologram follows the aim on the ground, pale amber when you hold its cost
//! and red when you don't, and a click builds an unlit campfire there for that cost. Q or
//! Escape leaves building. Campfire sticks will be their own item later; for now a
//! campfire costs reeds.
use bevy::light::NotShadowCaster;
use bevy::picking::events::{PointerClick, PointerOut, PointerOver};
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use stubby_core::{GroundHeight, HudCamera, LookCapture, PlacedAsset, Screen, ScreenMode};
use stubby_streams::{StreamBuild, StreamSystems};
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL, TILE};

use super::recipes_plugin::{PlayerInventory, RecipeCatalog};

const CAMPFIRE: &str = "campfire";
const CAMPFIRE_MODEL: &str = "structures/campfire/campfire.glb";
/// Where a campfire can be put down: this near and this far along the aim, in metres.
const NEAREST: f32 = 2.0;
const REACH: f32 = 10.0;

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CampfireBuild>()
            .add_systems(Startup, spawn_menu)
            .add_systems(
                Update,
                (
                    open_menu.before(StreamSystems::Input),
                    show_menu,
                    place_campfire.after(StreamSystems::Input),
                    tint_hologram,
                ),
            );
    }
}

/// Placing a campfire: on while the player has picked it from the build menu.
#[derive(Resource, Default)]
pub struct CampfireBuild {
    pub active: bool,
    ghost: Option<Entity>,
    valid: bool,
    built: u32,
}

#[derive(Component)]
struct MenuRoot;
#[derive(Component, Clone)]
struct BuildTile(String);
#[derive(Component)]
struct Ghost;
#[derive(Resource)]
struct Hologram(Handle<StandardMaterial>);

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    )
}

fn spawn_menu(
    mut commands: Commands,
    catalog: Option<Res<RecipeCatalog>>,
    names: Option<Res<stubby_core::ItemNames>>,
    server: Option<Res<AssetServer>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    if let Some(materials) = materials.as_mut() {
        commands.insert_resource(Hologram(materials.add(StandardMaterial {
            base_color: Color::srgba(1., 0.83, 0.6, 0.35),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })));
    }
    let Some(catalog) = catalog else { return };
    let buildables: Vec<_> = catalog.0.buildable_definitions().cloned().collect();
    let name = |item: &str| {
        names
            .as_ref()
            .map_or_else(|| item.replace('_', " "), |names| names.name(item))
    };
    commands
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BACKDROP),
            GlobalZIndex(140),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    padding: UiRect::all(px(20)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
            ))
            .with_children(|panel| {
                panel.spawn(text("BUILD", 26., INK));
                panel.spawn(text("Pick what to build, then place it.", 14., MUTED));
                panel
                    .spawn((
                        Node {
                            column_gap: px(12),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        for buildable in &buildables {
                            let cost = buildable
                                .cost
                                .iter()
                                .map(|stack| format!("{} {}", stack.quantity, name(&stack.item)))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let mut tile = row.spawn((
                                BuildTile(buildable.id.clone()),
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: px(6),
                                    width: px(150),
                                    padding: UiRect::all(px(10)),
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BackgroundColor(TILE),
                                BorderColor::all(EDGE),
                            ));
                            tile.with_children(|tile| {
                                let mut icon = tile.spawn((
                                    Node {
                                        width: px(96),
                                        height: px(96),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                                if let Some(server) = server.as_ref() {
                                    icon.insert(stubby_ui::item_icon(server, &buildable.id));
                                }
                                tile.spawn(text(buildable.name.clone(), 17., INK));
                                tile.spawn(text(cost, 13., MUTED));
                            });
                            tile.observe(hover).observe(unhover).observe(choose);
                        }
                    });
                panel.spawn(text("Q or Esc  close", 13., MUTED));
            });
        });
}

/// Q opens or closes the build menu; while building, Q leaves building instead.
fn open_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut screens: ResMut<ScreenMode>,
    stream: Option<Res<StreamBuild>>,
    mut camp: ResMut<CampfireBuild>,
) {
    if !keys.just_pressed(stubby_core::keys::BUILD) {
        return;
    }
    if camp.active {
        camp.active = false;
        return;
    }
    if stream.is_some_and(|stream| stream.active) {
        return; // stubby_streams leaves stream building on this Q
    }
    match screens.open {
        None => screens.open = Some(Screen::Build),
        Some(Screen::Build) => screens.close(),
        Some(_) => {}
    }
}

fn show_menu(screens: Res<ScreenMode>, mut roots: Query<&mut Visibility, With<MenuRoot>>) {
    let open = screens.open == Some(Screen::Build);
    for mut visibility in &mut roots {
        visibility.set_if_neq(if open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn hover(over: On<PointerOver>, mut tiles: Query<&mut BorderColor, With<BuildTile>>) {
    if let Ok(mut border) = tiles.get_mut(over.entity) {
        *border = BorderColor::all(AMBER);
    }
}

fn unhover(out: On<PointerOut>, mut tiles: Query<&mut BorderColor, With<BuildTile>>) {
    if let Ok(mut border) = tiles.get_mut(out.entity) {
        *border = BorderColor::all(EDGE);
    }
}

/// A tile picks what to build and closes the menu.
fn choose(
    click: On<PointerClick>,
    tiles: Query<&BuildTile>,
    mut screens: ResMut<ScreenMode>,
    stream: Option<ResMut<StreamBuild>>,
    mut camp: ResMut<CampfireBuild>,
) {
    let Ok(BuildTile(id)) = tiles.get(click.entity) else {
        return;
    };
    if screens.open != Some(Screen::Build) {
        return;
    }
    screens.close();
    match id.as_str() {
        "stream_segment" => {
            if let Some(mut stream) = stream {
                stream.active = true;
            }
        }
        CAMPFIRE => camp.active = true,
        _ => {}
    }
}

/// Where the aim meets the ground, between `NEAREST` and `REACH` along it; looking up,
/// the farthest reach on the ground.
fn aim_on_ground(eye: Vec3, dir: Vec3, ground: &GroundHeight) -> Vec3 {
    let height = |p: Vec3| (ground.0)(Vec2::new(p.x, p.z));
    let mut t = NEAREST;
    while t <= REACH {
        let p = eye + dir * t;
        if p.y <= height(p) {
            return Vec3::new(p.x, height(p), p.z);
        }
        t += 0.25;
    }
    let mut flat = Vec3::new(dir.x, 0., dir.z).normalize_or(Vec3::NEG_Z) * REACH;
    flat += Vec3::new(eye.x, 0., eye.z);
    Vec3::new(flat.x, height(flat), flat.z)
}

#[allow(clippy::too_many_arguments)] // Placing spans input, the world, the rules and stock.
fn place_campfire(
    mut commands: Commands,
    server: Option<Res<AssetServer>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    capture: Res<LookCapture>,
    screens: Res<ScreenMode>,
    ground: Res<GroundHeight>,
    catalog: Option<Res<RecipeCatalog>>,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    mut player: ResMut<PlayerInventory>,
    mut camp: ResMut<CampfireBuild>,
    mut ghosts: Query<&mut Transform, With<Ghost>>,
) {
    if screens.open.is_some() || keys.just_pressed(KeyCode::Escape) {
        camp.active = false;
    }
    if !camp.active {
        if let Some(ghost) = camp.ghost.take() {
            commands.entity(ghost).despawn();
        }
        return;
    }
    let (Some(server), Ok(camera)) = (server, camera.single()) else {
        return;
    };
    let ghost = *camp.ghost.get_or_insert_with(|| {
        commands
            .spawn((
                Ghost,
                Name::new("Campfire hologram"),
                WorldAssetRoot(server.load(format!("{CAMPFIRE_MODEL}#Scene0"))),
                Transform::default(),
                Visibility::default(),
            ))
            .id()
    });
    let eye = camera.translation();
    let at = aim_on_ground(eye, camera.forward().as_vec3(), &ground);
    let facing = Transform::from_translation(at).looking_at(Vec3::new(eye.x, at.y, eye.z), Vec3::Y);
    if let Ok(mut transform) = ghosts.get_mut(ghost) {
        *transform = facing;
    }
    let cost = catalog
        .and_then(|catalog| catalog.0.build_cost(CAMPFIRE, 1).ok())
        .unwrap_or_default();
    camp.valid = cost
        .iter()
        .all(|stack| player.items.quantity(&stack.item) >= stack.quantity);
    // The click that recaptures the cursor is not a build click.
    if !(capture.captured() && buttons.just_pressed(MouseButton::Left)) {
        return;
    }
    if !camp.valid {
        player.feedback = cost
            .iter()
            .map(|stack| format!("{} {}", stack.quantity, stack.item.replace('_', " ")))
            .collect::<Vec<_>>()
            .join(", ");
        player.feedback = format!("A campfire needs {}.", player.feedback);
        return;
    }
    if player.items.take_stacks(&cost).is_err() {
        return;
    }
    camp.built += 1;
    commands.spawn((
        Name::new("Campfire"),
        facing,
        Visibility::default(),
        WorldAssetRoot(server.load(format!("{CAMPFIRE_MODEL}#Scene0"))),
        PlacedAsset {
            asset: CAMPFIRE.into(),
            model: CAMPFIRE_MODEL.into(),
            instance: format!("campfire-built-{}", camp.built),
        },
    ));
    player.feedback = "Built a campfire. Light it with a backbeat: a spark on 2 and 4.".into();
}

/// The hologram's parts take the hologram material, amber when it can be built and red
/// when it can't, and cast no shadow.
fn tint_hologram(
    mut commands: Commands,
    camp: Res<CampfireBuild>,
    hologram: Option<Res<Hologram>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    parts: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    parents: Query<&ChildOf>,
    ghosts: Query<(), With<Ghost>>,
) {
    let (Some(hologram), Some(materials)) = (hologram, materials.as_mut()) else {
        return;
    };
    for (entity, material) in &parts {
        let in_ghost = parents
            .iter_ancestors(entity)
            .any(|ancestor| ghosts.contains(ancestor));
        if in_ghost && material.0 != hologram.0 {
            commands
                .entity(entity)
                .insert((MeshMaterial3d(hologram.0.clone()), NotShadowCaster));
        }
    }
    if camp.is_changed()
        && let Some(mut material) = materials.get_mut(&hologram.0)
    {
        material.base_color = if camp.valid {
            Color::srgba(1., 0.83, 0.6, 0.35)
        } else {
            Color::srgba(1., 0.35, 0.27, 0.35)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_aim_lands_on_the_ground_within_reach() {
        let flat = GroundHeight(|_| 0.);
        let eye = Vec3::new(0., 1.7, 0.);
        // Looking down at 45 degrees: where the aim meets the ground, about 1.7 m ahead.
        let down = Vec3::new(0., -1., -1.).normalize();
        let at = aim_on_ground(eye, down, &flat);
        assert!(at.y.abs() < 1e-6 && (1.6..1.9).contains(&-at.z), "{at}");
        // Looking at the horizon: the farthest reach, on the ground.
        let level = aim_on_ground(eye, Vec3::NEG_Z, &flat);
        assert!((level.z + REACH).abs() < 1e-4 && level.y == 0., "{level}");
    }
}
