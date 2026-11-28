//! Building, as in Satisfactory: Q opens the build menu, a tile per buildable (from the
//! rules' `buildables.json`, with its icon and cost), and picking one starts placing it.
//! Streams are laid by `stubby_streams`; everything else with a `model` is placed here,
//! the same way for every buildable: its hologram finds its place by the row's
//! `placement` (on the ground; beside a stream, as a campfire; across a laid segment, as a
//! purifier's dam), pale amber when it can be built there with the cost in hand and red
//! when not, and a click builds it there for that cost. Q or Escape leaves building.
use bevy::light::NotShadowCaster;
use bevy::picking::events::{PointerClick, PointerOut, PointerOver};
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use stubby_core::{GroundHeight, HudCamera, LookCapture, PlacedAsset, Screen, ScreenMode};
use stubby_streams::{StreamBuild, StreamSystems};
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL, TILE};

use super::recipes_plugin::{PlayerInventory, RecipeCatalog};

use crate::stream_features::OnSegment;
use stubby_recipes::Placement;

/// Where a buildable can be put down: this near and this far along the aim, in metres.
const NEAREST: f32 = 2.0;
const REACH: f32 = 10.0;

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Placing>()
            .init_resource::<BuiltCount>()
            .add_message::<BuildRequest>()
            .add_systems(Startup, spawn_menu)
            .add_systems(
                Update,
                (
                    open_menu.before(StreamSystems::Input),
                    show_menu,
                    place.after(StreamSystems::Input),
                    apply_builds.after(place).run_if(not(resource_exists::<
                        bevy_stubby_recipes::DecidedElsewhere,
                    >)),
                    tint_hologram,
                ),
            );
    }
}

/// Placing a buildable: the one picked from the build menu, while it is being placed.
#[derive(Resource, Default)]
pub struct Placing {
    pub active: Option<String>,
    /// The hologram, and which buildable it shows.
    ghost: Option<(Entity, String)>,
    valid: bool,
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
    mut placing: ResMut<Placing>,
) {
    if !keys.just_pressed(stubby_core::keys::BUILD) {
        return;
    }
    if placing.active.is_some() {
        placing.active = None;
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
    catalog: Option<Res<RecipeCatalog>>,
    mut placing: ResMut<Placing>,
) {
    let Ok(BuildTile(id)) = tiles.get(click.entity) else {
        return;
    };
    if screens.open != Some(Screen::Build) {
        return;
    }
    screens.close();
    let Some(buildable) = catalog.as_ref().and_then(|c| c.0.buildable(id)) else {
        return;
    };
    match (buildable.placement, &buildable.model) {
        (Placement::Laid, _) => {
            if let Some(mut stream) = stream {
                stream.active = true;
            }
        }
        (_, Some(_)) => placing.active = Some(id.clone()),
        (_, None) => {}
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

/// A buildable placed beside a stream, aimed within its reach of one, snaps beside the stream's bank
/// (add-stream-layers): this far from the stream's centre line, its half width with banks
/// (1.58 m) and the fire's ash bed.
const BESIDE_STREAM_M: f32 = 2.2;

/// Where a campfire aimed at `at` goes: beside the nearest stream within `reach_m`, on the
/// aim's side, or at the aim.
fn beside_stream(at: Vec3, eye: Vec3, streams: &[(f32, Vec3)], reach_m: f32) -> Option<Vec3> {
    let (_, point) = streams
        .iter()
        .map(|(d, p)| (*d, *p))
        .filter(|(d, _)| *d <= reach_m)
        .min_by(|a, b| a.0.total_cmp(&b.0))?;
    // The aim's side of the stream; aimed at the water itself, the player's side.
    let away = Vec3::new(at.x - point.x, 0., at.z - point.z);
    let side = if away.length() > 0.3 {
        away
    } else {
        Vec3::new(eye.x - point.x, 0., eye.z - point.z)
    }
    .normalize_or(Vec3::X);
    Some(point + side * BESIDE_STREAM_M)
}

/// Where a buildable placed across a segment goes: the aimed laid stream within this
/// many metres of the aim, at the point nearest the aim, turned to the flow there.
const SEGMENT_AIM_M: f32 = 2.5;

/// The place across the nearest laid stream: its centre line at the aim, the model's +X
/// along the flow (the stream segment's frame), and the segment it stands on.
fn across_segment(
    at: Vec3,
    streams: &[(Entity, &stubby_streams::LaidStream)],
) -> Option<(Transform, OnSegment)> {
    let (entity, stream, point) = streams
        .iter()
        .map(|(entity, stream)| (*entity, *stream, stream.nearest_point(at)))
        .filter(|(.., p)| Vec2::new(p.x - at.x, p.z - at.z).length() <= SEGMENT_AIM_M)
        .min_by(|a, b| {
            let d = |p: Vec3| Vec2::new(p.x - at.x, p.z - at.z).length_squared();
            d(a.2).total_cmp(&d(b.2))
        })?;
    let on = OnSegment::of(entity, stream, point);
    let (along, _) = stream.nearest_along(point);
    let (_, flow) = stream.path.at(along - stream.start + stream.start_s());
    let yaw = (-flow.z).atan2(flow.x);
    Some((
        Transform::from_translation(point).with_rotation(Quat::from_rotation_y(yaw)),
        on,
    ))
}

#[allow(clippy::too_many_arguments)] // Placing spans input, the world, the rules and stock.
fn place(
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
    mut placing: ResMut<Placing>,
    mut ghosts: Query<&mut Transform, With<Ghost>>,
    streams: Query<(Entity, &stubby_streams::LaidStream)>,
    rules: Option<Res<bevy_stubby_recipes::MachineRules>>,
    mut requests: MessageWriter<BuildRequest>,
) {
    if screens.open.is_some() || keys.just_pressed(KeyCode::Escape) {
        placing.active = None;
    }
    let buildable = placing
        .active
        .as_ref()
        .and_then(|id| catalog.as_ref()?.0.buildable(id).cloned());
    // A hologram of something else (or nothing to place) goes.
    if let Some((ghost, shown)) = placing.ghost.clone()
        && buildable.as_ref().is_none_or(|b| b.id != shown)
    {
        commands.entity(ghost).despawn();
        placing.ghost = None;
    }
    let (Some(buildable), Some(server), Ok(camera)) = (buildable, server, camera.single()) else {
        return;
    };
    let Some(model) = buildable.model.clone() else {
        return;
    };
    let ghost = placing
        .ghost
        .get_or_insert_with(|| {
            let ghost = commands
                .spawn((
                    Ghost,
                    Name::new(format!("{} hologram", buildable.name)),
                    WorldAssetRoot(server.load(format!("{model}#Scene0"))),
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();
            (ghost, buildable.id.clone())
        })
        .0;
    let eye = camera.translation();
    let mut at = aim_on_ground(eye, camera.forward().as_vec3(), &ground);
    let facing = |at: Vec3| {
        Transform::from_translation(at).looking_at(Vec3::new(eye.x, at.y, eye.z), Vec3::Y)
    };
    let laid: Vec<(Entity, &stubby_streams::LaidStream)> = streams.iter().collect();
    // Where it goes, and whether this place will do.
    let (place, on, placeable) = match buildable.placement {
        Placement::BesideStream => {
            let reach_m = rules
                .as_ref()
                .and_then(|rules| rules.of(&buildable.id)?.clock_reach_m)
                .unwrap_or(0.);
            let near: Vec<(f32, Vec3)> = laid
                .iter()
                .filter(|(_, s)| s.source.is_some())
                .map(|(_, s)| {
                    let point = s.nearest_point(at);
                    (Vec2::new(point.x - at.x, point.z - at.z).length(), point)
                })
                .collect();
            if let Some(snapped) = beside_stream(at, eye, &near, reach_m) {
                at = Vec3::new(snapped.x, ground.at(snapped.x, snapped.z), snapped.z);
            }
            (facing(at), None, true)
        }
        Placement::AcrossSegment => match across_segment(at, &laid) {
            Some((place, on)) => (place, Some(on), true),
            None => (facing(at), None, false),
        },
        Placement::Ground | Placement::Laid => (facing(at), None, true),
    };
    if let Ok(mut transform) = ghosts.get_mut(ghost) {
        *transform = place;
    }
    let cost = catalog
        .as_ref()
        .and_then(|catalog| catalog.0.build_cost(&buildable.id, 1).ok())
        .unwrap_or_default();
    let affordable = cost
        .iter()
        .all(|stack| player.items.quantity(&stack.item) >= stack.quantity);
    let valid = affordable && placeable;
    if placing.valid != valid {
        placing.valid = valid;
    }
    // The click that recaptures the cursor is not a build click.
    if !(capture.captured() && buttons.just_pressed(MouseButton::Left)) {
        return;
    }
    if !affordable {
        player.feedback =
            shortfall_notice(&buildable.name, &cost, &player.items, catalog.as_deref());
        return;
    }
    if !placeable {
        return;
    }
    requests.write(BuildRequest {
        buildable: buildable.id.clone(),
        place,
        segment: on.map(|on| on.segment),
    });
}

/// Ask to build `buildable` at `place` (across segment `segment` of a laid stream, for a
/// buildable placed so). Built where its decider is: here, or a shared world's host.
#[derive(Message, Clone, Debug, PartialEq)]
pub struct BuildRequest {
    pub buildable: String,
    pub place: Transform,
    pub segment: Option<u64>,
}

/// What was built, and where: every built thing carries it, so a peer that did not build
/// it (a shared world's client) can show it. Its model, name and place follow from it.
#[derive(Component, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Built {
    pub buildable: String,
    pub instance: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    /// The stream segment it sits across, by key.
    pub segment: Option<u64>,
}

impl Built {
    pub fn place(&self) -> Transform {
        Transform::from_translation(self.translation.into())
            .with_rotation(Quat::from_array(self.rotation))
    }
}

/// How many things have been built here: each built thing's instance is
/// `<buildable>-built-<n>`, unique in the world because one peer counts.
#[derive(Resource, Default)]
pub struct BuiltCount(pub u32);

/// Take `buildable`'s cost from `items`: its rule, or what it is short of.
pub fn pay_for(
    catalog: &RecipeCatalog,
    buildable: &str,
    items: &mut stubby_recipes::Inventory,
) -> Result<stubby_recipes::BuildableDefinition, String> {
    let Some(definition) = catalog.0.buildable(buildable).cloned() else {
        return Err(format!("Nothing called {buildable} can be built."));
    };
    if definition.model.is_none() {
        return Err(format!("{} is laid, not placed.", definition.name));
    }
    let cost = catalog.0.build_cost(buildable, 1).unwrap_or_default();
    if items.take_stacks(&cost).is_err() {
        return Err(shortfall_notice(
            &definition.name,
            &cost,
            items,
            Some(catalog),
        ));
    }
    Ok(definition)
}

/// Build what was paid for: the next instance, its record, and what shows it.
pub fn spawn_built(
    commands: &mut Commands,
    server: &AssetServer,
    count: &mut BuiltCount,
    definition: &stubby_recipes::BuildableDefinition,
    place: Transform,
    on: Option<OnSegment>,
) -> Entity {
    count.0 += 1;
    let built = Built {
        buildable: definition.id.clone(),
        instance: format!("{}-built-{}", definition.id, count.0),
        translation: place.translation.into(),
        rotation: place.rotation.to_array(),
        segment: on.map(|on| on.segment),
    };
    let mut entity = commands.spawn((built_looks(server, definition, &built), built));
    if let Some(on) = on {
        entity.insert(on);
    }
    entity.id()
}

/// What shows a built thing and makes it what it is (its model and placed asset).
pub fn built_looks(
    server: &AssetServer,
    definition: &stubby_recipes::BuildableDefinition,
    built: &Built,
) -> impl Bundle + use<> {
    let model = definition.model.clone().unwrap_or_default();
    (
        Name::new(definition.name.clone()),
        built.place(),
        Visibility::default(),
        WorldAssetRoot(server.load(format!("{model}#Scene0"))),
        PlacedAsset {
            asset: definition.id.clone(),
            model,
            instance: built.instance.clone(),
        },
    )
}

/// The stream segment with this key, as something across it sits on it.
pub fn segment_on(
    key: u64,
    at: Vec3,
    streams: &Query<(Entity, &stubby_streams::LaidStream)>,
) -> Option<OnSegment> {
    streams
        .iter()
        .find(|(_, laid)| (laid.first..=laid.last).any(|k| laid.segment_key(k) == key))
        .map(|(stream, laid)| OnSegment::of(stream, laid, at))
}

/// Builds decided here: pay, build, and meet the `built` goal.
#[allow(clippy::too_many_arguments)]
fn apply_builds(
    mut requests: MessageReader<BuildRequest>,
    mut commands: Commands,
    server: Option<Res<AssetServer>>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: ResMut<PlayerInventory>,
    mut count: ResMut<BuiltCount>,
    streams: Query<(Entity, &stubby_streams::LaidStream)>,
    mut goals: MessageWriter<bevy_stubby_recipes::GoalRequest>,
) {
    let (Some(server), Some(catalog)) = (server, catalog) else {
        requests.clear();
        return;
    };
    for request in requests.read() {
        let on = request
            .segment
            .and_then(|key| segment_on(key, request.place.translation, &streams));
        if request.segment.is_some() && on.is_none() {
            continue;
        }
        let definition = match pay_for(&catalog, &request.buildable, &mut player.items) {
            Ok(definition) => definition,
            Err(said) => {
                player.feedback = said;
                continue;
            }
        };
        spawn_built(
            &mut commands,
            &server,
            &mut count,
            &definition,
            request.place,
            on,
        );
        // Every buildable built meets its `built` goal condition (the opening's campfire).
        goals.write(built_goal(&definition.id));
        if let Some(hint) = &definition.built_hint {
            player.feedback = hint.clone();
        }
    }
}

/// The goal event a build meets.
pub fn built_goal(buildable: &str) -> bevy_stubby_recipes::GoalRequest {
    bevy_stubby_recipes::GoalRequest(stubby_recipes::GoalEvent::ConditionMet(
        stubby_recipes::Condition::Built {
            buildable: buildable.into(),
        },
    ))
}

/// The hologram's parts take the hologram material, amber when it can be built and red
/// when it can't, and cast no shadow.
/// What a build is short of, and for anything that is crafted, where it is made, so the
/// player is pointed at the campfire rather than left guessing (user, 2026-11-20).
fn shortfall_notice(
    name: &str,
    cost: &[stubby_recipes::Stack],
    held: &stubby_recipes::Inventory,
    catalog: Option<&RecipeCatalog>,
) -> String {
    let item_name = |id: &str| {
        catalog
            .and_then(|c| c.0.item(id))
            .map_or_else(|| id.replace('_', " "), |item| item.name.to_lowercase())
    };
    let crafted = |id: &str| {
        catalog.is_some_and(|c| {
            c.0.recipes()
                .any(|r| r.outputs.iter().any(|s| s.item == id))
        })
    };
    let short: Vec<_> = cost
        .iter()
        .filter(|stack| held.quantity(&stack.item) < stack.quantity)
        .collect();
    if let Some(stack) = short.iter().find(|stack| crafted(&stack.item)) {
        return format!(
            "A {} needs a {} kit: craft it at a lit campfire (E), then build it here.",
            name.to_lowercase(),
            item_name(&stack.item).trim_end_matches(" kit"),
        );
    }
    let needs = short
        .iter()
        .map(|stack| format!("{} {}", stack.quantity, item_name(&stack.item)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("A {} needs {needs}.", name.to_lowercase())
}

fn tint_hologram(
    mut commands: Commands,
    placing: Res<Placing>,
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
    if placing.is_changed()
        && let Some(mut material) = materials.get_mut(&hologram.0)
    {
        material.base_color = if placing.valid {
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
    fn a_campfire_snaps_beside_a_stream_within_reach() {
        let stream = [(3., Vec3::ZERO)];
        let eye = Vec3::new(0., 1.7, 10.);
        // Aimed 3 m off the stream: beside its bank, on that side.
        let at = beside_stream(Vec3::new(0., 0., 3.), eye, &stream, 5.).expect("snaps");
        assert!((at.z - BESIDE_STREAM_M).abs() < 1e-4 && at.x.abs() < 1e-4);
        // Aimed at the water: the player's side.
        let at = beside_stream(Vec3::ZERO, eye, &[(0., Vec3::ZERO)], 5.).expect("snaps");
        assert!(at.z > 0.);
        // Out of reach: no snap.
        assert!(beside_stream(Vec3::new(0., 0., 9.), eye, &[(9., Vec3::ZERO)], 5.).is_none());
    }

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

    /// A purifier's dam snaps across the aimed laid stream, square to its flow, and has no
    /// place away from one (add-reed-water-purifier D6).
    #[test]
    fn a_dam_snaps_across_the_aimed_segment() {
        let laid = stubby_streams::LaidStream::straight_segments(
            None,
            Vec3::ZERO,
            Vec3::new(0., 0., -12.),
            3,
        );
        let stream = Entity::from_raw_u32(7).unwrap();
        let streams = [(stream, &laid)];
        let (place, on) = across_segment(Vec3::new(1.2, 0., -6.), &streams).expect("a place");
        assert!(place.translation.x.abs() < 0.05 && (place.translation.z + 6.).abs() < 0.3);
        // The model's +X runs downstream (-Z here).
        assert!(place.rotation.mul_vec3(Vec3::X).distance(Vec3::NEG_Z) < 0.01);
        assert_eq!(on.stream, stream);
        assert_eq!(on.segment, laid.segment_key(1));
        assert!(across_segment(Vec3::new(5., 0., -6.), &streams).is_none());
    }
}
