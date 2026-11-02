//! The rack every machine is configured through: what is mounted, which cables are
//! patched, the audio patch that follows, and the guidance on the rack screen
//! (add-rack-configurations, add-mudpit-burbler). Machines differ only in data: their
//! profile's `configure` block (places, what an open place accepts, spare cables, the
//! normalled output). Modules differ only in `parts` and `jack` below.
//!
//! Sound starts when every spare cable is patched: the engine patch is exactly the
//! cables, so a half-patched voice is naturally silent (no CV opens the VCA, or nothing
//! reaches its input). Completing the last patch sends `MachineConfigured`, once.
use super::{
    audio::Synth,
    controls::{Screen, ScreenMode},
    machine_config::{
        CombNode, JackHit, JackRef, PodMesh, PortNode, Profiles, RackActions, RackConfiguration,
    },
    module_ids::ModuleIds,
    prefabs::{AssetClock, PlacedAsset},
    resources::{PlayerInventory, RecipeCatalog},
};
use bevy::{
    asset::RenderAssetUsages, mesh::Indices, picking::events::PointerClick, prelude::*,
    render::render_resource::PrimitiveTopology,
};
use std::collections::HashMap;
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};
use stubby_recipes::Stack;

/// What the rack of one placed machine holds.
#[derive(Component, Default)]
pub struct RackState {
    /// Place -> module asset, for places the player filled (fixed places are in the profile).
    pub mounted: HashMap<String, String>,
    /// Per comb cable in the profile: the (output, input) it joins, if patched. A comb
    /// cable goes between any output and any input, as in VCV Rack; the profile's jacks
    /// for it are only the patch the screen suggests.
    pub patched: Vec<Option<(JackRef, JackRef)>>,
    /// The player's own cables (output jack, input jack), each one patch cable from the
    /// inventory; unplugging returns it.
    pub own: Vec<(JackRef, JackRef)>,
    /// Bumped whenever the rack's contents change, so the screen redraws.
    pub version: u32,
}
impl RackState {
    /// Every patched connection, comb cables and the player's own.
    pub fn connections(&self) -> impl Iterator<Item = &(JackRef, JackRef)> {
        self.patched.iter().flatten().chain(&self.own)
    }
    pub fn joins(&self, from: &JackRef, to: &JackRef) -> bool {
        self.connections().any(|(f, t)| f == from && t == to)
    }
    /// An input takes one cable; an output may feed several.
    pub fn input_taken(&self, input: &JackRef) -> bool {
        self.connections().any(|(_, t)| t == input)
    }
    /// Every patch the machine's voice needs is made, by whichever cables.
    pub fn complete(&self, config: &RackConfiguration) -> bool {
        !config.cables.is_empty() && config.cables.iter().all(|c| self.joins(&c.from, &c.to))
    }
}

/// Engine modules this rack owns (not the machine's clock), and whether it has sounded.
#[derive(Component, Default)]
struct RackEngine {
    owned: Vec<ModuleId>,
    configured: bool,
}

/// A machine's rack was completed: every spare cable patched, its voice sounds.
#[derive(Message, Clone, Copy)]
pub struct MachineConfigured {
    pub machine: Entity,
}

/// The jack clicked first, waiting for the jack that completes the cable.
#[derive(Resource, Default)]
struct PatchSelection(Option<JackRef>);

/// A cable taken from the comb: which spare cable, and the jack its first end went into.
#[derive(Resource, Default)]
pub(crate) struct HeldCable {
    pub(crate) cable: Option<usize>,
    first: Option<JackRef>,
}

/// A hanging spare cable's tube or plug on the comb (spare cable index), pickable.
#[derive(Component, Clone, Copy)]
pub(crate) struct CombCable(pub usize);

pub struct RackPlugin;
impl Plugin for RackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatchSelection>()
            .init_resource::<HeldCable>()
            .add_observer(take_cable)
            .add_observer(drop_cable)
            .add_observer(drag_cable)
            .add_observer(drag_from_jack)
            .add_observer(drop_on_jack)
            .init_resource::<DraggedJack>()
            .add_message::<MachineConfigured>()
            .add_observer(click_jack)
            .add_observer(press_mount)
            .add_observer(start_item_drag)
            .add_observer(move_item_drag)
            .add_observer(end_item_drag)
            .add_observer(drop_item)
            .add_systems(
                Update,
                (
                    attach,
                    mount_buttons,
                    number_keys,
                    comb_parts_pickable,
                    live_parts_unpickable,
                    tint_plugs,
                    load_snaps,
                    snap_on_plug,
                    rack_ghost,
                    clear_feedback_on_open,
                    patch_engine,
                    pods,
                    draw_cables,
                    live_cable,
                    snap_release,
                )
                    .chain()
                    .after(super::machine_config::MachineInteractionSet),
            );
    }
}

// ------------------------------------------------------------------ modules → engine

/// The engine modules a rack module is built from (the burbler's own clock is the
/// machine's, `AssetClock`).
fn parts(module: &str) -> Vec<(&'static str, ModuleKind)> {
    match module {
        // A Square VCO is made from a bug's square wave, so it sounds at the bug's pitch.
        "square_vco" => vec![(
            "vco",
            ModuleKind::SquareVco {
                freq_hz: super::waveform_audio::bug_click().freq_hz,
            },
        )],
        // The volume is a plain VCA: TRIG opens it while the gate is high, so the
        // voice stays square (no envelope shaping it).
        "volume" => vec![("vca", ModuleKind::Vca { gain: 0.25 })],
        _ => vec![],
    }
}
/// Connections inside one module (none yet: the volume is a single VCA).
fn internal(_module: &str) -> Vec<((&'static str, &'static str), (&'static str, &'static str))> {
    vec![]
}
/// A panel jack's engine port: (part, port).
fn jack(module: &str, port: &str) -> Option<(&'static str, &'static str)> {
    Some(match (module, port) {
        ("burbler_clock", "gate") => ("clock", "gate"),
        ("burbler_clock", "phase") => ("clock", "phase"),
        ("square_vco", "out") => ("vco", "out"),
        ("square_vco", "voct") => ("vco", "voct"),
        ("volume", "trig") => ("vca", "cv"),
        ("volume", "in") => ("vca", "in"),
        ("volume", "out") => ("vca", "out"),
        _ => return None,
    })
}

/// Whether a panel jack is an output (true) or an input (false).
fn is_output(module: &str, port: &str) -> Option<bool> {
    Some(match (module, port) {
        ("burbler_clock", "gate" | "phase") | ("square_vco", "out") | ("volume", "out") => true,
        ("square_vco", "voct") | ("volume", "trig" | "in") => false,
        _ => return None,
    })
}

/// The module in each place: fixed ones from the profile, the rest as mounted.
fn modules(config: &RackConfiguration, state: &RackState) -> HashMap<String, String> {
    config
        .places
        .iter()
        .filter_map(|p| {
            let m = if p.fixed {
                p.module.clone()
            } else {
                state.mounted.get(&p.place).cloned()
            };
            m.map(|m| (p.place.clone(), m))
        })
        .collect()
}

/// The full engine patch for a rack: modules to add and connections, given the
/// machine's clock and a source of fresh IDs. Pure, so it is easy to check.
fn plan(
    config: &RackConfiguration,
    state: &RackState,
    clock: Option<ModuleId>,
    mut fresh: impl FnMut() -> Option<ModuleId>,
) -> (Vec<ModuleId>, Vec<Command>) {
    let mut owned = Vec::new();
    let mut commands = Vec::new();
    let mut ids: HashMap<(String, &'static str), ModuleId> = HashMap::new();
    let modules = modules(config, state);
    for (place, module) in &modules {
        if let Some(clock) = clock.filter(|_| module == "burbler_clock") {
            ids.insert((place.clone(), "clock"), clock);
        }
        for (part, kind) in parts(module) {
            let Some(id) = fresh() else { continue };
            owned.push(id);
            ids.insert((place.clone(), part), id);
            commands.push(Command::AddModule { id, kind });
        }
        for ((a, ap), (b, bp)) in internal(module) {
            if let (Some(&from), Some(&to)) =
                (ids.get(&(place.clone(), a)), ids.get(&(place.clone(), b)))
            {
                commands.push(Command::Connect {
                    from: PortRef::new(from, ap),
                    to: PortRef::new(to, bp),
                });
            }
        }
    }
    let port = |j: &JackRef| {
        let module = modules.get(&j.place)?;
        let (part, port) = jack(module, &j.port)?;
        Some(PortRef::new(*ids.get(&(j.place.clone(), part))?, port))
    };
    for (from, to) in state.connections() {
        if let (Some(from), Some(to)) = (port(from), port(to)) {
            commands.push(Command::Connect { from, to });
        }
    }
    if let Some(out) = config.normalled_output.as_ref().and_then(port) {
        commands.push(Command::Connect {
            from: out,
            to: PortRef::output(),
        });
    }
    (owned, commands)
}

// ------------------------------------------------------------------ state

fn attach(
    mut commands: Commands,
    profiles: Res<Profiles>,
    machines: Query<(Entity, &PlacedAsset), Without<RackState>>,
) {
    for (entity, asset) in &machines {
        if let Some(config) = profiles.0.get(&asset.asset) {
            commands.entity(entity).insert((
                RackState {
                    patched: vec![None; config.cables.len()],
                    ..default()
                },
                RackEngine::default(),
            ));
        }
    }
}

/// Rebuild a rack's engine modules when it changes; report completion once.
fn patch_engine(
    profiles: Res<Profiles>,
    synth: Option<Res<Synth>>,
    mut ids: ResMut<ModuleIds>,
    mut racks: Query<
        (
            Entity,
            &PlacedAsset,
            &RackState,
            &mut RackEngine,
            Option<&AssetClock>,
        ),
        Changed<RackState>,
    >,
    mut configured: MessageWriter<MachineConfigured>,
) {
    for (machine, asset, state, mut engine, clock) in &mut racks {
        let Some(config) = profiles.0.get(&asset.asset) else {
            continue;
        };
        let (owned, plan) = plan(config, state, clock.map(|c| c.engine), || ids.allocate());
        if let Some(synth) = &synth {
            for id in engine.owned.drain(..) {
                synth.send(Command::RemoveModule { id });
            }
            for command in plan {
                synth.send(command);
            }
        }
        engine.owned = owned;
        if state.complete(config) && !engine.configured {
            engine.configured = true;
            configured.write(MachineConfigured { machine });
        }
    }
}

fn open_rack(screens: &ScreenMode) -> Option<Entity> {
    match screens.open {
        Some(Screen::MachineConfiguration(e)) => Some(e),
        _ => None,
    }
}

// ------------------------------------------------------------------ mounting

#[derive(Component)]
pub(crate) struct MountButton {
    machine: Entity,
    place: String,
    item: String,
    module: String,
}

/// One button per open place the player can fill from the inventory.
fn mount_buttons(
    mut commands: Commands,
    server: Res<AssetServer>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    racks: Query<(&PlacedAsset, &RackState)>,
    actions: Query<Entity, With<RackActions>>,
    boxes: Query<Entity, With<super::machine_config::RackInventory>>,
    mut last: Local<Option<(Entity, u32, u32)>>,
) {
    let (Ok(area), Ok(grid)) = (actions.single(), boxes.single()) else {
        return;
    };
    let Some((machine, (asset, state))) =
        open_rack(&screens).and_then(|e| racks.get(e).ok().map(|r| (e, r)))
    else {
        if last.take().is_some() {
            commands.entity(area).despawn_related::<Children>();
            commands.entity(grid).despawn_related::<Children>();
        }
        return;
    };
    let held: u32 = player.items.items().map(|(_, n)| n).sum();
    if *last == Some((machine, state.version, held)) {
        return;
    }
    *last = Some((machine, state.version, held));
    commands.entity(area).despawn_related::<Children>();
    commands.entity(grid).despawn_related::<Children>();
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    // The player's inventory, Satisfactory-style: a "Relevant" strip of what this rack's
    // open places accept (number keys 1-0 insert them, as does a click), then the rest
    // of what you carry, dimmed.
    let relevant: Vec<_> = relevant(config, state, &player.items);
    let amber = Color::srgb(1., 0.7, 0.31);
    let muted = Color::srgb(0.55, 0.53, 0.49);
    let text = |value: String, size: f32, color: Color| {
        (
            Text::new(value),
            TextFont {
                font_size: bevy::text::FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Pickable::IGNORE,
        )
    };
    let name = |id: &str| {
        catalog
            .as_ref()
            .and_then(|c| c.0.item(id))
            .map_or(id.to_string(), |i| i.name.clone())
    };
    let mount = |place: &super::machine_config::RackPlace,
                 accept: &super::machine_config::Accept| {
        MountButton {
            machine,
            place: place.place.clone(),
            item: accept.item.clone(),
            module: accept.module.clone(),
        }
    };
    // Left: what fits an open slot, as Satisfactory's list (icon, name, count); drag a
    // row into its slot, or click it, or press its number.
    commands.entity(area).with_children(|list| {
        for (place, accept) in &relevant {
            let count = player.items.quantity(&accept.item);
            list.spawn((
                Node {
                    column_gap: px(10),
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(8), px(4)),
                    ..default()
                },
                BackgroundColor(amber.with_alpha(0.9)),
                mount(place, accept),
            ))
            .with_children(|row| {
                row.spawn((
                    item_icon(&server, &accept.item),
                    super::inventory_screen::ItemName(name(&accept.item)),
                    Node {
                        width: px(36),
                        height: px(36),
                        ..default()
                    },
                ));
                row.spawn((
                    text(name(&accept.item), 18., Color::srgb(0.1, 0.08, 0.05)),
                    Node {
                        flex_grow: 1.,
                        ..default()
                    },
                ));
                row.spawn(text(
                    format!("[{count}]"),
                    15.,
                    Color::srgb(0.1, 0.08, 0.05),
                ));
            });
        }
    });
    // Right: the inventory, 16 boxes; an item that fits an open slot can be dragged
    // from its box into the rack.
    commands.entity(grid).with_children(|grid| {
        let held: Vec<_> = player.items.boxes().collect();
        for index in 0..stubby_recipes::INVENTORY_BOXES as usize {
            let mut cell = grid.spawn((
                Node {
                    width: px(64),
                    height: px(64),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.11, 0.1, 0.09)),
                BorderColor::all(muted.with_alpha(0.5)),
                Pickable::IGNORE,
            ));
            let Some((id, count)) = held.get(index) else {
                continue;
            };
            if let Some((place, accept)) = relevant.iter().find(|(_, a)| a.item == *id) {
                cell.insert((mount(place, accept), Pickable::default()));
            }
            cell.with_children(|cell| {
                icon_slot(cell, Some(&server), id, &name(id), *count, 56.);
            });
        }
    });
}

/// The items this rack's open places accept, in place order (at most ten: keys 1-0).
fn relevant<'a>(
    config: &'a RackConfiguration,
    state: &RackState,
    held: &stubby_recipes::Inventory,
) -> Vec<(
    &'a super::machine_config::RackPlace,
    &'a super::machine_config::Accept,
)> {
    config
        .places
        .iter()
        .filter(|p| !p.fixed && !state.mounted.contains_key(&p.place))
        .flat_map(|p| p.accepts.iter().map(move |a| (p, a)))
        // Only what the player holds: an item you have none of is not shown.
        .filter(|(_, a)| held.quantity(&a.item) > 0)
        .take(10)
        .collect()
}

/// Number keys 1-0 insert the matching relevant item, as in Satisfactory.
fn number_keys(
    keys: Res<ButtonInput<KeyCode>>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut player: ResMut<PlayerInventory>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
) {
    const DIGITS: [KeyCode; 10] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];
    let Some(index) = DIGITS.iter().position(|k| keys.just_pressed(*k)) else {
        return;
    };
    let Some(machine) = open_rack(&screens) else {
        return;
    };
    let Ok((asset, mut state)) = racks.get_mut(machine) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    let Some((place, accept)) = relevant(config, &state, &player.items)
        .get(index)
        .map(|(p, a)| (p.place.clone(), (*a).clone()))
    else {
        return;
    };
    if player.items.take(&accept.item, 1).is_ok() {
        state.mounted.insert(place, accept.module.clone());
        state.version += 1;
        player.feedback = format!("Mounted a {}.", module_name(&accept.module));
    }
}

fn module_name(module: &str) -> String {
    match module {
        "square_vco" => "Square VCO".into(),
        other => other.replace('_', " "),
    }
}

/// Clicking a mount button (or its label) takes the item and mounts the module.
fn press_mount(
    click: On<PointerClick>,
    buttons: Query<&MountButton>,
    parents: Query<&ChildOf>,
    mut player: ResMut<PlayerInventory>,
    mut racks: Query<&mut RackState>,
) {
    let Some(mount) = mount_of(click.entity, &buttons, &parents) else {
        return;
    };
    let Ok(mut state) = racks.get_mut(mount.machine) else {
        return;
    };
    if state.mounted.contains_key(&mount.place) {
        return;
    }
    if player.items.take(&mount.item, 1).is_ok() {
        state
            .mounted
            .insert(mount.place.clone(), mount.module.clone());
        state.version += 1;
        player.feedback = format!("Mounted a {}.", module_name(&mount.module));
    }
}

use super::inventory_screen::{icon_slot, item_icon};

/// The icon following the pointer while an item is dragged from the Relevant strip.
#[derive(Component)]
struct DragGhost(String);
/// The same icon inside the rack view: the rack camera draws over the screen's UI
/// there, so over the rack the dragged item is a card in the rack's own scene.
#[derive(Component)]
struct RackGhost;

/// The mount button a pointer event landed on (the tile or one of its children).
fn mount_of<'a>(
    entity: Entity,
    buttons: &'a Query<&MountButton>,
    parents: &Query<&ChildOf>,
) -> Option<&'a MountButton> {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find_map(|e| buttons.get(e).ok())
}

const GHOST_PX: f32 = 56.;

/// Dragging a Relevant tile picks up its item: its icon follows the pointer.
fn start_item_drag(
    drag: On<bevy::picking::events::PointerDragStart>,
    mut commands: Commands,
    server: Res<AssetServer>,
    buttons: Query<&MountButton>,
    ghosts: Query<Entity, With<DragGhost>>,
) {
    // The event bubbles from the icon up through the tile; handle it once, at the tile.
    let Ok(mount) = buttons.get(drag.entity) else {
        return;
    };
    ghosts.iter().for_each(|g| commands.entity(g).despawn());
    let at = drag.pointer.position - Vec2::splat(GHOST_PX / 2.);
    commands.spawn((
        DragGhost(mount.item.clone()),
        item_icon(&server, &mount.item),
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(GHOST_PX),
            height: px(GHOST_PX),
            ..default()
        },
        GlobalZIndex(100),
        Pickable::IGNORE,
    ));
}

fn move_item_drag(
    drag: On<bevy::picking::events::PointerDrag>,
    mut ghosts: Query<&mut Node, With<DragGhost>>,
) {
    let at = drag.pointer.position - Vec2::splat(GHOST_PX / 2.);
    for mut node in &mut ghosts {
        node.left = px(at.x);
        node.top = px(at.y);
    }
}

fn end_item_drag(
    _drag: On<bevy::picking::events::PointerDragEnd>,
    mut commands: Commands,
    ghosts: Query<Entity, Or<(With<DragGhost>, With<RackGhost>)>>,
) {
    ghosts.iter().for_each(|g| commands.entity(g).despawn());
}

/// While an item is dragged, its card follows the pointer inside the rack view, just in
/// front of the panels, the size of the screen ghost.
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // The drag, the rack camera and the card.
fn rack_ghost(
    mut commands: Commands,
    server: Res<AssetServer>,
    ghosts: Query<&DragGhost>,
    hud: Query<(&Camera, &GlobalTransform), With<super::machine_config::RackHudCamera>>,
    ports: Query<&GlobalTransform, With<PortNode>>,
    pointers: Query<(
        &bevy::picking::pointer::PointerId,
        &bevy::picking::pointer::PointerLocation,
    )>,
    mut cards: Query<(Entity, &mut Transform), With<RackGhost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (Some(DragGhost(item)), Ok((camera, eye))) = (ghosts.iter().next(), hud.single()) else {
        return;
    };
    let Some(position) = pointers
        .iter()
        .find(|(id, _)| **id == bevy::picking::pointer::PointerId::Mouse)
        .and_then(|(_, l)| l.location.as_ref())
        .map(|l| l.position)
    else {
        return;
    };
    let Some(panel) = ports.iter().next().map(GlobalTransform::translation) else {
        return;
    };
    // On a plane a little in front of the panels, facing the camera.
    let normal = eye.forward().as_vec3();
    let plane = panel - normal * 0.15;
    let on_plane = |p: Vec2| {
        let ray = camera.viewport_to_world(eye, p).ok()?;
        let d = ray.direction.dot(normal);
        (d.abs() > 1e-4)
            .then(|| ray.origin + ray.direction * ((plane - ray.origin).dot(normal) / d))
    };
    let (Some(at), Some(edge)) = (on_plane(position), on_plane(position + Vec2::X * GHOST_PX))
    else {
        return;
    };
    let size = at.distance(edge);
    let transform = Transform::from_translation(at)
        .looking_to(normal, eye.up().as_vec3())
        .with_scale(Vec3::splat(size));
    if let Ok((_, mut card)) = cards.single_mut() {
        *card = transform;
        return;
    }
    commands.spawn((
        RackGhost,
        Mesh3d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(server.load(format!("ui/icons/{item}.png"))),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        transform,
        bevy::camera::visibility::RenderLayers::layer(super::machine_config::HUD_LAYER),
        Pickable::IGNORE,
    ));
}

/// Dropping a Relevant item on its empty slot mounts the module, Satisfactory-style.
fn drop_item(
    drop: On<bevy::picking::events::PointerDragDrop>,
    slots: Query<&super::machine_config::SlotHit>,
    buttons: Query<&MountButton>,
    parents: Query<&ChildOf>,
    mut player: ResMut<PlayerInventory>,
    mut racks: Query<&mut RackState>,
) {
    let (Ok(slot), Some(mount)) = (
        slots.get(drop.entity),
        mount_of(drop.dropped, &buttons, &parents),
    ) else {
        return;
    };
    if slot.0 != mount.place {
        player.feedback = format!("A {} does not fit there.", module_name(&mount.module));
        return;
    }
    let Ok(mut state) = racks.get_mut(mount.machine) else {
        return;
    };
    if !state.mounted.contains_key(&mount.place) && player.items.take(&mount.item, 1).is_ok() {
        state
            .mounted
            .insert(mount.place.clone(), mount.module.clone());
        state.version += 1;
        player.feedback = format!("Mounted a {}.", module_name(&mount.module));
    }
}

// ------------------------------------------------------------------ patching

/// Click one jack, then the other: if a spare cable joins those two, it is patched.
/// Clicking a patched jack unplugs its cable back onto the comb.
#[allow(clippy::too_many_arguments)] // A click reads the rack, its cables and the inventory.
fn click_jack(
    click: On<PointerClick>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut held: ResMut<HeldCable>,
    mut dragged: ResMut<DraggedJack>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<PlayerInventory>,
) {
    // A drag that started here already pressed this jack; its click is the same gesture.
    if dragged.0.take() == Some(click.entity) {
        return;
    }
    let Ok(JackHit(jack)) = jacks.get(click.entity) else {
        return;
    };
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut held, &mut player)
    });
}

/// VCV Rack's gesture: drag from a jack (or a hanging cable) and drop on another jack.
/// Starting a drag presses the jack it starts on; dropping presses the jack under it.
#[allow(clippy::too_many_arguments)] // As `click_jack`.
fn drag_from_jack(
    drag: On<bevy::picking::events::PointerDragStart>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut held: ResMut<HeldCable>,
    mut dragged: ResMut<DraggedJack>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<PlayerInventory>,
) {
    let Ok(JackHit(jack)) = jacks.get(drag.entity) else {
        return;
    };
    dragged.0 = Some(drag.entity);
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut held, &mut player)
    });
}

#[allow(clippy::too_many_arguments)] // As `click_jack`.
fn drop_on_jack(
    drop: On<bevy::picking::events::PointerDragDrop>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut held: ResMut<HeldCable>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<PlayerInventory>,
) {
    let (Ok(JackHit(jack)), true) = (jacks.get(drop.entity), jacks.contains(drop.dropped)) else {
        return;
    };
    if drop.entity == drop.dropped {
        return;
    }
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut held, &mut player)
    });
}

/// The drag gesture's jack, so the click that ends it is not counted twice.
#[derive(Resource, Default)]
struct DraggedJack(Option<Entity>);

/// Run `f` on the open rack's configuration and state.
fn on_rack(
    screens: &ScreenMode,
    profiles: &Profiles,
    racks: &mut Query<(&PlacedAsset, &mut RackState)>,
    f: impl FnOnce(&RackConfiguration, &mut RackState),
) {
    let Some(machine) = open_rack(screens) else {
        return;
    };
    let Ok((asset, mut state)) = racks.get_mut(machine) else {
        return;
    };
    if let Some(config) = profiles.0.get(&asset.asset) {
        f(config, &mut state);
    }
}

/// One press of a jack, by click or by the drag gesture.
fn press_jack(
    jack: &JackRef,
    config: &RackConfiguration,
    state: &mut RackState,
    selection: &mut PatchSelection,
    held: &mut HeldCable,
    player: &mut PlayerInventory,
) {
    let mounted = modules(config, state);
    // A cable joins an output to a free input, whichever end goes in first.
    let orient = |a: &JackRef, b: &JackRef, state: &RackState| {
        let (from, to) = match (direction(&mounted, a), direction(&mounted, b)) {
            (Some(true), Some(false)) => (a.clone(), b.clone()),
            (Some(false), Some(true)) => (b.clone(), a.clone()),
            _ => return None,
        };
        (!state.input_taken(&to)).then_some((from, to))
    };
    // A cable taken from the comb: its first end, then its second, any output to any input.
    if let Some(i) = held.cable {
        match held.first.take() {
            None => held.first = Some(jack.clone()),
            Some(first) if &first != jack => match orient(&first, jack, state) {
                Some(link) => {
                    state.patched[i] = Some(link);
                    state.version += 1;
                    held.cable = None;
                }
                // Not an output and a free input: keep it in hand.
                None => held.first = Some(first),
            },
            Some(first) => held.first = Some(first),
        }
        return;
    }
    // A patched comb cable at this jack: back onto the comb.
    if let Some(i) = state.patched.iter().position(|p| {
        p.as_ref()
            .is_some_and(|(from, to)| from == jack || to == jack)
    }) {
        state.patched[i] = None;
        state.version += 1;
        selection.0 = None;
        player.feedback = "Cable back on the comb.".into();
        return;
    }
    // One of the player's own cables at this jack: back into the inventory.
    if let Some(i) = state.own.iter().position(|(a, b)| a == jack || b == jack) {
        state.own.remove(i);
        state.version += 1;
        selection.0 = None;
        let back = player.items.grant(&[Stack {
            item: PATCH_CABLE.into(),
            quantity: 1,
        }]);
        player.feedback = match back {
            Ok(()) => format!(
                "Patch cable back in your inventory ({}).",
                player.items.quantity(PATCH_CABLE)
            ),
            Err(error) => error.to_string(),
        };
        return;
    }
    let Some(first) = selection.0.take() else {
        selection.0 = Some(jack.clone());
        return;
    };
    if &first == jack {
        selection.0 = Some(first);
        return;
    }
    let Some((from, to)) = orient(&first, jack, state) else {
        return;
    };
    // Jack to jack: the comb's free cable for this patch if it has one, else your own.
    if let Some(i) = config
        .cables
        .iter()
        .zip(&state.patched)
        .position(|(c, p)| p.is_none() && c.from == from && c.to == to)
    {
        state.patched[i] = Some((from, to));
        state.version += 1;
        return;
    }
    if player.items.take(PATCH_CABLE, 1).is_ok() {
        state.own.push((from, to));
        state.version += 1;
        player.feedback = format!(
            "Patched one of your cables ({} left).",
            player.items.quantity(PATCH_CABLE)
        );
    } else {
        player.feedback = "You have no patch cables.".into();
    }
}

/// Clicking a hanging cable takes it from the comb; then click its two jacks.
fn take_cable(
    click: On<PointerClick>,
    parts: Query<&CombCable>,
    screens: Res<ScreenMode>,
    racks: Query<&RackState>,
    mut held: ResMut<HeldCable>,
    mut selection: ResMut<PatchSelection>,
) {
    let Ok(CombCable(i)) = parts.get(click.entity) else {
        return;
    };
    let Some(state) = open_rack(&screens).and_then(|e| racks.get(e).ok()) else {
        return;
    };
    if state.patched.get(*i).is_some_and(Option::is_none) {
        held.cable = Some(*i);
        held.first = None;
        selection.0 = None;
    }
}

/// Dragging a hanging cable off the comb takes it, as clicking it does.
fn drag_cable(
    drag: On<bevy::picking::events::PointerDragStart>,
    parts: Query<&CombCable>,
    screens: Res<ScreenMode>,
    racks: Query<&RackState>,
    mut held: ResMut<HeldCable>,
    mut selection: ResMut<PatchSelection>,
) {
    let Ok(CombCable(i)) = parts.get(drag.entity) else {
        return;
    };
    let Some(state) = open_rack(&screens).and_then(|e| racks.get(e).ok()) else {
        return;
    };
    if state.patched.get(*i).is_some_and(Option::is_none) {
        held.cable = Some(*i);
        held.first = None;
        selection.0 = None;
    }
}

/// Dropping a hanging cable on one of its jacks plugs that end in; the other end stays
/// in hand, following the pointer, until it goes into its other jack (VCV Rack).
#[allow(clippy::too_many_arguments)] // As `click_jack`.
fn drop_cable(
    drop: On<bevy::picking::events::PointerDragDrop>,
    parts: Query<&CombCable>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut held: ResMut<HeldCable>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<PlayerInventory>,
) {
    let (Ok(CombCable(_)), Ok(JackHit(jack))) = (parts.get(drop.dropped), jacks.get(drop.entity))
    else {
        return;
    };
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut held, &mut player)
    });
}

/// The comb's hanging cables load as models; make their meshes pickable.
fn comb_parts_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Mesh3d>, Without<Pickable>, Without<CombCable>)>,
    parents: Query<&ChildOf>,
    combs: Query<&CombCable>,
) {
    for entity in &meshes {
        if let Some(cable) = parents
            .iter_ancestors(entity)
            .find_map(|a| combs.get(a).ok())
        {
            commands
                .entity(entity)
                .insert((*cable, Pickable::default()));
        }
    }
}

/// On a plug model's root: its cable's material. The plug glb ships one sleeve colour;
/// its sleeve and cable stub take the cable's colour instead, as in VCV Rack.
#[derive(Component)]
struct PlugColour(Handle<StandardMaterial>);

fn tint_plugs(
    mut commands: Commands,
    meshes: Query<(Entity, &ChildOf), (With<Mesh3d>, Without<PlugTinted>)>,
    names: Query<&Name>,
    parents: Query<&ChildOf>,
    colours: Query<&PlugColour>,
) {
    for (entity, parent) in &meshes {
        let Ok(name) = names.get(parent.parent()) else {
            continue;
        };
        if !matches!(
            name.as_str(),
            "PatchCable_Sleeve" | "PatchCable_Stub" | "PatchCable_Relief"
        ) {
            continue;
        }
        if let Some(PlugColour(material)) = parents
            .iter_ancestors(entity)
            .find_map(|a| colours.get(a).ok())
        {
            commands
                .entity(entity)
                .insert((MeshMaterial3d(material.clone()), PlugTinted));
        }
    }
}
#[derive(Component)]
struct PlugTinted;

// ------------------------------------------------------------------ sound

/// The plug-in snaps: CC0 clicks from Kenney's Interface Sounds
/// (`assets/audio/sfx/patch_snap/`, licence beside them), played in turn so repeated
/// patching does not sound mechanical.
const SNAPS: [&[u8]; 3] = [
    include_bytes!("../../../assets/audio/sfx/patch_snap/click_002.wav"),
    include_bytes!("../../../assets/audio/sfx/patch_snap/click_004.wav"),
    include_bytes!("../../../assets/audio/sfx/patch_snap/switch_001.wav"),
];
const SNAP_GAIN: f32 = 0.6;

/// A 16-bit PCM WAV as (sample rate, one buffer per channel).
fn decode_wav(bytes: &[u8]) -> Option<(u32, Vec<Vec<f32>>)> {
    let u16_at = |i: usize| Some(u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?));
    let u32_at = |i: usize| Some(u32::from_le_bytes(bytes.get(i..i + 4)?.try_into().ok()?));
    if bytes.get(0..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }
    let (mut at, mut format, mut data) = (12, None, None);
    while at + 8 <= bytes.len() {
        let (id, len) = (bytes.get(at..at + 4)?, u32_at(at + 4)? as usize);
        let body = at + 8;
        match id {
            b"fmt " => {
                format = Some((
                    u16_at(body)?,
                    u16_at(body + 2)?,
                    u32_at(body + 4)?,
                    u16_at(body + 14)?,
                ))
            }
            b"data" => data = Some(bytes.get(body..(body + len).min(bytes.len()))?),
            _ => {}
        }
        at = body + len + (len & 1);
    }
    let ((1, channels, rate, 16), data) = (format?, data?) else {
        return None;
    };
    let channels = channels.max(1) as usize;
    let mut out = vec![Vec::new(); channels];
    for (i, frame) in data.chunks_exact(2).enumerate() {
        out[i % channels].push(i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.);
    }
    Some((rate, out))
}

/// Load the snaps into the engine once its thread is up.
fn load_snaps(synth: Option<Res<super::audio::Synth>>, mut done: Local<bool>) {
    let Some(thread) = synth.as_ref().and_then(|s| s.thread()) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    for (k, bytes) in SNAPS.iter().enumerate() {
        if let Some((rate, channels)) = decode_wav(bytes) {
            thread.load_sample(stubby_audio::SampleId(k as u32 + 1), rate, channels);
        }
    }
}

/// A cable going into a jack (a comb cable patched, or one of your own) snaps.
fn snap_on_plug(
    synth: Option<Res<super::audio::Synth>>,
    racks: Query<(Entity, &RackState), Changed<RackState>>,
    mut plugged: Local<HashMap<Entity, usize>>,
    mut next: Local<u32>,
) {
    for (machine, state) in &racks {
        let now = state.connections().count();
        let before = plugged.insert(machine, now).unwrap_or(now);
        if now > before
            && let Some(synth) = &synth
        {
            synth.send(stubby_audio::Command::PlaySample {
                sample: stubby_audio::SampleId(*next % SNAPS.len() as u32 + 1),
                gain: SNAP_GAIN,
            });
            *next += 1;
        }
    }
}

/// The live cable's plug model must never take a click meant for the jack under it.
fn live_parts_unpickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Mesh3d>, Without<Pickable>)>,
    parents: Query<&ChildOf>,
    live: Query<(), With<LiveCable>>,
) {
    for entity in &meshes {
        if parents.iter_ancestors(entity).any(|a| live.contains(a)) {
            commands.entity(entity).insert(Pickable::IGNORE);
        }
    }
}

/// Feedback on the rack screen starts fresh each time it opens.
fn clear_feedback_on_open(
    mut opened: MessageReader<super::machine_config::MachineConfigurationOpened>,
    mut player: ResMut<PlayerInventory>,
    mut held: ResMut<HeldCable>,
) {
    if opened.read().next().is_some() {
        player.feedback.clear();
        *held = HeldCable::default();
    }
}

/// Your own cables are this item; the comb's come with the rack.
const PATCH_CABLE: &str = "patch_cable";

fn direction(mounted: &HashMap<String, String>, j: &JackRef) -> Option<bool> {
    is_output(mounted.get(&j.place)?, &j.port)
}

// ------------------------------------------------------------------ guidance

fn cable_color(name: &str) -> Color {
    match name {
        "verdigris" => Color::srgb_u8(0x3f, 0x7a, 0x6e),
        "rust" => Color::srgb_u8(0x9c, 0x3b, 0x2a),
        "ochre" => Color::srgb_u8(0xb8, 0x89, 0x2f),
        "slate" => Color::srgb_u8(0x4b, 0x5a, 0x78),
        _ => Color::srgb_u8(0xcf, 0xc6, 0xb2),
    }
}

/// Pods pulse in the colour of the cable still waiting for their jack; others rest.
fn pods(
    time: Res<Time>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    racks: Query<(&PlacedAsset, &RackState)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: Query<(
        &PodMesh,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    mut owned: Local<HashMap<Entity, Handle<StandardMaterial>>>,
    entities: Query<Entity, With<PodMesh>>,
) {
    let Some((asset, state)) = open_rack(&screens).and_then(|e| racks.get(e).ok()) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    let ready = config
        .places
        .iter()
        .all(|p| p.fixed || state.mounted.contains_key(&p.place));
    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * std::f32::consts::TAU / 1.2).sin();
    for (entity, (pod, mut material, mut visibility)) in entities.iter().zip(meshes.iter_mut()) {
        let waiting = config
            .cables
            .iter()
            .find(|c| ready && !state.joins(&c.from, &c.to) && (c.from == pod.0 || c.to == pod.0));
        let Some(cable) = waiting else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        let handle = owned.entry(entity).or_insert_with(|| {
            materials.add(StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })
        });
        if material.0 != *handle {
            material.0 = handle.clone();
        }
        if let Some(mut m) = materials.get_mut(&*handle) {
            let c = cable_color(&cable.color).to_linear();
            m.base_color = Color::LinearRgba(c.with_alpha(0.35 + 0.4 * pulse));
            m.emissive = c * (1.0 + 4.0 * pulse);
        }
    }
}

// ------------------------------------------------------------------ cables

#[derive(Component)]
struct CableVisual;

const CABLE_RADIUS: f32 = 0.016;

/// How near (in screen pixels) the pointer must come to a jack for the cable in hand
/// to snap to it, as in VCV Rack.
const SNAP_PX: f32 = 40.;

/// The jack the cable in hand would go into from `pointer` (target coordinates): the
/// nearest within `SNAP_PX` that makes a valid patch from what is already plugged in
/// (its first end, or your selected jack), or any jack while no end is in yet.
#[allow(clippy::too_many_arguments)] // The hand, the rack and the screen.
fn snap_target(
    pointer: Vec2,
    camera: &Camera,
    eye: &GlobalTransform,
    config: &RackConfiguration,
    state: &RackState,
    held: &HeldCable,
    selection: &PatchSelection,
    ports: &Query<(&PortNode, &GlobalTransform)>,
) -> Option<(JackRef, Vec3)> {
    let anchor = held.first.as_ref().or(selection.0.as_ref());
    if held.cable.is_none() && anchor.is_none() {
        return None;
    }
    let mounted = modules(config, state);
    let valid = |j: &JackRef| match anchor {
        None => direction(&mounted, j).is_some(),
        Some(a) if a == j => false,
        Some(a) => match (direction(&mounted, a), direction(&mounted, j)) {
            (Some(true), Some(false)) => !state.input_taken(j),
            (Some(false), Some(true)) => !state.input_taken(a),
            _ => false,
        },
    };
    ports
        .iter()
        .filter(|(p, _)| valid(&p.0))
        .filter_map(|(p, t)| {
            let at = camera.world_to_viewport(eye, t.translation()).ok()?;
            let d = at.distance(pointer);
            (d <= SNAP_PX).then(|| (d, p.0.clone(), t.translation()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, j, at)| (j, at))
}

/// Letting go near a valid jack plugs the cable in there, even off the jack itself
/// (a direct hit on a jack is already handled by picking).
#[allow(clippy::too_many_arguments)] // The release, the hand, the rack and the screen.
fn snap_release(
    mut inputs: MessageReader<bevy::picking::pointer::PointerInput>,
    hovers: Res<bevy::picking::hover::HoverMap>,
    jacks: Query<(), With<JackHit>>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    hud: Query<(&Camera, &GlobalTransform), With<super::machine_config::RackHudCamera>>,
    ports: Query<(&PortNode, &GlobalTransform)>,
    mut hand: (
        ResMut<HeldCable>,
        ResMut<PatchSelection>,
        ResMut<PlayerInventory>,
    ),
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
) {
    use bevy::picking::pointer::{PointerAction, PointerButton, PointerId};
    let Some(at) = inputs
        .read()
        .filter(|i| {
            i.pointer_id == PointerId::Mouse
                && matches!(i.action, PointerAction::Release(PointerButton::Primary))
        })
        .last()
        .map(|i| i.location.position)
    else {
        return;
    };
    let on_jack = hovers
        .get(&PointerId::Mouse)
        .is_some_and(|h| h.keys().any(|e| jacks.contains(*e)));
    let (Ok((camera, eye)), false) = (hud.single(), on_jack) else {
        return;
    };
    let (held, selection, player) = (&mut *hand.0, &mut *hand.1, &mut *hand.2);
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        if let Some((jack, _)) =
            snap_target(at, camera, eye, config, state, held, selection, &ports)
        {
            press_jack(&jack, config, state, selection, held, player);
        }
    });
}

/// The cable in hand, drawn as VCV Rack draws a cable being patched, and alive: it is
/// a small spring simulation, so it swings after the pointer instead of snapping.
/// Taken from the comb, it hangs from the pointer (one plug at the pointer, the cable
/// looping down to the other, which swings); once one end is in a jack, it runs from
/// that jack to the pointer with the same elastic sag as a patched cable, the sag
/// springing after every move. The plugs are the cable's own patch_plug models.
#[derive(Component)]
struct LiveCable;

/// The live cable's moving parts: the curve's middle and (while hanging from the
/// pointer) its swinging far end, each a damped spring toward where gravity rests it.
#[derive(Default)]
struct CableSim {
    colours: HashMap<String, Handle<StandardMaterial>>,
    plugs: [Option<Entity>; 2],
    started: bool,
    mid: (Vec3, Vec3),
    end: (Vec3, Vec3),
}
impl CableSim {
    fn spring(point: &mut (Vec3, Vec3), target: Vec3, dt: f32) {
        const STIFFNESS: f32 = 70.;
        const DAMPING: f32 = 7.;
        let (x, v) = point;
        *v += ((target - *x) * STIFFNESS - *v * DAMPING) * dt;
        *x += *v * dt;
    }
}

/// How far the far plug hangs below the pointer while a cable is carried.
const HANG_M: f32 = 0.32;
/// The patch plug model's cable attaches this far above its origin (`patch_plug.glb`'s
/// `Cable_attach`).
const PLUG_ATTACH_M: f32 = 0.33;

#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Reads the hand, the rack's sockets, the camera and pointer.
fn live_cable(
    mut commands: Commands,
    time: Res<Time>,
    server: Res<AssetServer>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    held: Res<HeldCable>,
    selection: Res<PatchSelection>,
    racks: Query<(&PlacedAsset, &RackState)>,
    ports: Query<(&PortNode, &GlobalTransform)>,
    hud: Query<(&Camera, &GlobalTransform), With<super::machine_config::RackHudCamera>>,
    pointers: Query<(
        &bevy::picking::pointer::PointerId,
        &bevy::picking::pointer::PointerLocation,
    )>,
    old: Query<Entity, (With<LiveCable>, With<Mesh3d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sim: Local<CableSim>,
) {
    old.iter().for_each(|e| commands.entity(e).despawn());
    let clear = |commands: &mut Commands, sim: &mut CableSim| {
        for plug in sim.plugs.iter_mut() {
            if let Some(e) = plug.take() {
                commands.entity(e).despawn();
            }
        }
        sim.started = false;
    };
    let rack = open_rack(&screens).and_then(|m| racks.get(m).ok());
    let config = rack.and_then(|(a, _)| profiles.0.get(&a.asset));
    let port = |j: &JackRef| {
        ports
            .iter()
            .find(|(p, _)| &p.0 == j)
            .map(|(_, t)| t.translation())
    };
    // The anchored end: a comb cable's first jack, or the first jack of one of your own.
    let anchor = held.first.as_ref().or(selection.0.as_ref()).and_then(port);
    let carrying = held.cable.is_some() || anchor.is_some();
    let (Some(config), true, Ok((camera, eye))) = (config, carrying, hud.single()) else {
        clear(&mut commands, &mut sim);
        return;
    };
    // The pointer, on the plane of the jacks, facing the camera.
    let Some(plane) = anchor.or_else(|| ports.iter().next().map(|(_, t)| t.translation())) else {
        clear(&mut commands, &mut sim);
        return;
    };
    let screen = pointers
        .iter()
        .find(|(id, _)| **id == bevy::picking::pointer::PointerId::Mouse)
        .and_then(|(_, l)| l.location.as_ref())
        .map(|l| l.position);
    let Some(mut pointer) = screen
        .and_then(|p| camera.viewport_to_world(eye, p).ok())
        .and_then(|ray| {
            let normal = eye.forward().as_vec3();
            let d = ray.direction.dot(normal);
            (d.abs() > 1e-4)
                .then(|| ray.origin + ray.direction * ((plane - ray.origin).dot(normal) / d))
        })
    else {
        clear(&mut commands, &mut sim);
        return;
    };
    // Near a jack it can go into, the plug snaps onto it (VCV Rack): what you see is
    // where it will go when you let go.
    if let (Some((_, state)), Some(screen)) = (rack, screen)
        && let Some((_, at)) = snap_target(
            screen, camera, eye, config, state, &held, &selection, &ports,
        )
    {
        pointer = at;
    }
    let dt = time.delta_secs().min(1. / 30.);
    // The plug at the pointer points into the panel, as VCV Rack draws a plug: seen
    // head-on, its round end sits exactly where it will go in, and the cable leaves its
    // back toward the viewer and bends away at an elbow.
    let facing = Quat::from_rotation_arc(Vec3::NEG_Y, eye.forward().as_vec3());
    let held_plug = Transform::from_translation(pointer).with_rotation(facing);
    let held_attach = held_plug.translation + facing * Vec3::Y * PLUG_ATTACH_M;
    // Ends: anchored (jack -> held plug) or carried (held plug -> swinging far plug).
    let (a, end_target) = match anchor {
        Some(a) => (a, held_attach),
        None => (held_attach, held_attach + Vec3::NEG_Y * HANG_M),
    };
    if !sim.started {
        sim.started = true;
        sim.mid = ((a + end_target) / 2., Vec3::ZERO);
        sim.end = (end_target, Vec3::ZERO);
    }
    let b = if anchor.is_some() {
        sim.end = (held_attach, Vec3::ZERO);
        held_attach
    } else {
        CableSim::spring(&mut sim.end, end_target, dt);
        sim.end.0
    };
    // Where gravity rests the middle: VCV Rack's sag for this span.
    let rest = (a + b) / 2.
        + Vec3::NEG_Y * 0.5 * (0.508 + a.distance(b)) * if anchor.is_some() { 1. } else { 0.25 };
    CableSim::spring(&mut sim.mid, rest, dt);
    let colour = held
        .cable
        .and_then(|i| config.cables.get(i))
        .map_or("bone".to_string(), |c| c.color.clone());
    let layer = bevy::camera::visibility::RenderLayers::layer(super::machine_config::HUD_LAYER);
    let material = sim
        .colours
        .entry(colour.clone())
        .or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: cable_color(&colour),
                perceptual_roughness: 0.6,
                ..default()
            })
        })
        .clone();
    commands.spawn((
        LiveCable,
        Mesh3d(meshes.add(tube(&bezier(a, sim.mid.0, b), CABLE_RADIUS))),
        MeshMaterial3d(material.clone()),
        layer.clone(),
        Pickable::IGNORE,
    ));
    // Plugs: the held one; while carried, the far one hangs tip down from the cable's end.
    let ends: Vec<Transform> = if anchor.is_some() {
        vec![held_plug]
    } else {
        vec![
            held_plug,
            Transform::from_translation(b - Vec3::Y * PLUG_ATTACH_M),
        ]
    };
    for (k, plug) in sim.plugs.iter_mut().enumerate() {
        match (ends.get(k), *plug) {
            (Some(at), Some(e)) => {
                commands.entity(e).insert(*at);
            }
            (Some(at), None) => {
                *plug = Some(
                    commands
                        .spawn((
                            LiveCable,
                            WorldAssetRoot(server.load("models/cables/patch_plug.glb#Scene0")),
                            PlugColour(material.clone()),
                            *at,
                            bevy::app::Propagate(layer.clone()),
                            Pickable::IGNORE,
                        ))
                        .id(),
                );
            }
            (None, Some(e)) => {
                commands.entity(e).despawn();
                *plug = None;
            }
            (None, None) => {}
        }
    }
}

/// VCV Rack's elastic cable (`create_patch_cable.cable_curve`): a quadratic Bezier whose
/// middle sags by (1 - tension) * (slump_base + span).
fn cable_curve(a: Vec3, b: Vec3) -> Vec<Vec3> {
    let c = (a + b) / 2. + Vec3::NEG_Y * 0.5 * (0.508 + a.distance(b));
    bezier(a, c, b)
}
fn bezier(a: Vec3, c: Vec3, b: Vec3) -> Vec<Vec3> {
    (0..=32)
        .map(|i| i as f32 / 32.)
        .map(|t| a * (1. - t).powi(2) + c * 2. * (1. - t) * t + b * t * t)
        .collect()
}
fn tube(points: &[Vec3], radius: f32) -> Mesh {
    const SIDES: u32 = 10;
    let (mut pos, mut nor, mut idx) = (Vec::new(), Vec::new(), Vec::new());
    for (i, p) in points.iter().enumerate() {
        let t = (points[(i + 1).min(points.len() - 1)] - points[i.saturating_sub(1)])
            .normalize_or(Vec3::X);
        let n = t.any_orthonormal_vector();
        let s = t.cross(n);
        for k in 0..SIDES {
            let a = k as f32 / SIDES as f32 * std::f32::consts::TAU;
            let d = n * a.cos() + s * a.sin();
            pos.push((*p + d * radius).to_array());
            nor.push(d.to_array());
        }
    }
    for i in 0..points.len() as u32 - 1 {
        for k in 0..SIDES {
            let (a, b) = (i * SIDES + k, i * SIDES + (k + 1) % SIDES);
            let (c, d) = (a + SIDES, b + SIDES);
            idx.extend([a, c, b, b, c, d]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nor)
    .with_inserted_indices(Indices::U32(idx))
}

/// Patched cables between their jacks, spare ones hanging on the comb, and a ghost for
/// each patch still to make. Redrawn when the rack changes or its parts first appear.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw_cables(
    mut commands: Commands,
    server: Res<AssetServer>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    racks: Query<(&PlacedAsset, &RackState)>,
    ports: Query<(&PortNode, &GlobalTransform)>,
    comb: Query<(Entity, &CombNode, &GlobalTransform, &Transform)>,
    old: Query<Entity, With<CableVisual>>,
    held: Res<HeldCable>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawn: Local<Option<(Entity, u32, usize, Option<usize>)>>,
) {
    let Some(machine) = open_rack(&screens) else {
        if drawn.take().is_some() {
            old.iter().for_each(|e| commands.entity(e).despawn());
        }
        return;
    };
    let Ok((asset, state)) = racks.get(machine) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    // Redraw when the rack changes, or when its parts appear or move into place (the
    // frame's posts are fitted after the model spawns).
    let layout = ports
        .iter()
        .map(|(_, t)| t.translation())
        .chain(comb.iter().map(|(_, _, t, _)| t.translation()))
        .fold(Vec3::ZERO, |sum, p| sum + p.abs());
    let key = (
        machine,
        state.version,
        (layout.element_sum() * 1000.) as usize,
        // A cable in hand is drawn by `live_cable`, not on the comb.
        held.cable,
    );
    if *drawn == Some(key) {
        return;
    }
    *drawn = Some(key);
    old.iter().for_each(|e| commands.entity(e).despawn());
    let port = |j: &JackRef| {
        ports
            .iter()
            .find(|(p, _)| &p.0 == j)
            .map(|(_, t)| t.translation())
    };
    // A comb node: its entity and local transform (the plugs hang below their slot).
    let node = |name: String| {
        comb.iter()
            .find(|(_, c, ..)| c.0 == name)
            .map(|(e, _, _, t)| (e, *t))
    };
    let layer = bevy::camera::visibility::RenderLayers::layer(super::machine_config::HUD_LAYER);
    let ready = config
        .places
        .iter()
        .all(|p| p.fixed || state.mounted.contains_key(&p.place));
    let ghost = materials.add(StandardMaterial {
        base_color: Color::srgba(1., 0.83, 0.6, 0.22),
        emissive: LinearRgba::rgb(1.5, 1.2, 0.8),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let mut slot = 0;
    let own = materials.add(StandardMaterial {
        base_color: cable_color("bone"),
        perceptual_roughness: 0.6,
        ..default()
    });
    for (from, to) in &state.own {
        if let (Some(a), Some(b)) = (port(from), port(to)) {
            commands.spawn((
                CableVisual,
                Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS))),
                MeshMaterial3d(own.clone()),
                layer.clone(),
            ));
        }
    }
    for (cable, patched) in config.cables.iter().zip(&state.patched) {
        let mat = materials.add(StandardMaterial {
            base_color: cable_color(&cable.color),
            perceptual_roughness: 0.6,
            ..default()
        });
        let ends = (port(&cable.from), port(&cable.to));
        if let Some((from, to)) = patched {
            if let (Some(a), Some(b)) = (port(from), port(to)) {
                commands.spawn((
                    CableVisual,
                    Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS))),
                    MeshMaterial3d(mat.clone()),
                    layer.clone(),
                ));
            }
        }
        // The ghost of the suggested patch, until some cable makes it.
        if ready
            && !state.joins(&cable.from, &cable.to)
            && let (Some(a), Some(b)) = ends
        {
            commands.spawn((
                CableVisual,
                Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS * 0.7))),
                MeshMaterial3d(ghost.clone()),
                layer.clone(),
            ));
        }
        if patched.is_some() {
            continue;
        }
        let index = config
            .cables
            .iter()
            .position(|c| std::ptr::eq(c, cable))
            .unwrap_or(0);
        if key.3 == Some(index) {
            slot += 1;
            continue;
        }
        // Spare: folded over a comb slot, both plugs hanging tip down.
        if let (Some(top), Some(pa), Some(pb)) = (
            node(format!("Comb_{slot}")),
            node(format!("Comb_{slot}_PlugA")),
            node(format!("Comb_{slot}_PlugB")),
        ) {
            // Drawn in the slot's own space, as its children, so the cable and plugs
            // follow the frame however it is fitted (its posts move after it spawns).
            let (slot_entity, (a, b)) = (top.0, (pa.1.translation, pb.1.translation));
            let c = -(a + b) / 2. + Vec3::Y * 0.03;
            commands.spawn((
                CableVisual,
                CombCable(index),
                Pickable::default(),
                Mesh3d(meshes.add(tube(&bezier(a, c, b), CABLE_RADIUS * 1.6))),
                MeshMaterial3d(mat.clone()),
                layer.clone(),
                ChildOf(slot_entity),
            ));
            for plug in [pa, pb] {
                commands.spawn((
                    CableVisual,
                    CombCable(index),
                    WorldAssetRoot(server.load("models/cables/patch_plug.glb#Scene0")),
                    PlugColour(mat.clone()),
                    plug.1,
                    bevy::app::Propagate(layer.clone()),
                    ChildOf(slot_entity),
                ));
            }
        }
        slot += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_snaps_decode() {
        for bytes in SNAPS {
            let (rate, channels) = decode_wav(bytes).expect("16-bit PCM WAV");
            assert_eq!(rate, 44_100);
            assert!(channels[0].iter().any(|s| s.abs() > 0.05));
        }
    }

    #[test]
    fn the_burbler_sounds_only_with_both_cables_patched() {
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut state = RackState {
            patched: vec![None; config.cables.len()],
            ..default()
        };
        state.mounted.insert("vco".into(), "square_vco".into());
        let connections = |state: &RackState| {
            let mut next = 100;
            plan(config, state, Some(ModuleId(1)), || {
                next += 1;
                Some(ModuleId(next))
            })
            .1
            .into_iter()
            .filter(|c| matches!(c, Command::Connect { .. }))
            .count()
        };
        // The normalled output only: no tone.
        assert_eq!(connections(&state), 1);
        let link =
            |c: &super::super::machine_config::RackCable| Some((c.from.clone(), c.to.clone()));
        state.patched = vec![link(&config.cables[0]), None];
        assert!(!state.complete(config));
        state.patched = vec![link(&config.cables[0]), link(&config.cables[1])];
        assert!(state.complete(config));
        // A comb cable goes anywhere: the red one making the GATE -> TRIG patch and the
        // green one OUT -> IN still completes the voice.
        state.patched = vec![link(&config.cables[1]), link(&config.cables[0])];
        assert!(state.complete(config));
        assert_eq!(connections(&state), 3, "GATE->TRIG and OUT->IN added");
    }

    /// Development only: renders the burbler's patch to WAVs for listening (not run in
    /// normal builds). `STUBBY_RENDER_DIR=<dir> cargo test -p stubby_game --lib
    /// render_burbler -- --ignored` writes `burbler-patched.wav` (the two comb cables).
    #[test]
    #[ignore]
    fn render_burbler() {
        use stubby_audio::{Engine, EngineConfig};
        let dir = std::env::var("STUBBY_RENDER_DIR").unwrap_or_else(|_| ".".into());
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut patched = RackState {
            patched: config
                .cables
                .iter()
                .map(|c| Some((c.from.clone(), c.to.clone())))
                .collect(),
            ..default()
        };
        patched.mounted.insert("vco".into(), "square_vco".into());
        for (name, state) in [("burbler-patched", patched)] {
            const SR: u32 = 48_000;
            let (mut engine, mut renderer) = Engine::offline(EngineConfig {
                sample_rate: SR,
                max_block_frames: 512,
            })
            .unwrap();
            let clock = ModuleId(1);
            engine
                .send(Command::AddModule {
                    id: clock,
                    kind: ModuleKind::Clock { bpm: 90. },
                })
                .unwrap();
            let mut next = 100;
            for command in plan(config, &state, Some(clock), || {
                next += 1;
                Some(ModuleId(next))
            })
            .1
            {
                engine.send(command).unwrap();
            }
            engine.flush().unwrap();
            let mut samples = Vec::new();
            let mut buf = vec![0f32; 512 * 2];
            for _ in 0..(SR as usize * 6).div_ceil(512) {
                renderer.render(&mut buf);
                samples.extend_from_slice(&buf);
            }
            let mut wav = Vec::new();
            let data = (samples.len() * 2) as u32;
            for chunk in [&b"RIFF"[..], &(36 + data).to_le_bytes(), b"WAVEfmt "] {
                wav.extend_from_slice(chunk);
            }
            for v in [16u32, 1 | (2 << 16), SR, SR * 4, 4 | (16 << 16)] {
                wav.extend_from_slice(&v.to_le_bytes());
            }
            wav.extend_from_slice(b"data");
            wav.extend_from_slice(&data.to_le_bytes());
            for s in &samples {
                wav.extend_from_slice(&((s.clamp(-1., 1.) * 32767.) as i16).to_le_bytes());
            }
            std::fs::write(format!("{dir}/{name}.wav"), wav).unwrap();
        }
    }
}
