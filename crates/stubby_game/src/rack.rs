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
        RackHint,
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

/// What the rack of one placed machine holds.
#[derive(Component, Default)]
pub struct RackState {
    /// Place -> module asset, for places the player filled (fixed places are in the profile).
    pub mounted: HashMap<String, String>,
    /// Per spare cable in the profile: patched yet?
    pub patched: Vec<bool>,
    /// Bumped whenever the rack's contents change, so the screen redraws.
    pub version: u32,
}
impl RackState {
    pub fn complete(&self) -> bool {
        !self.patched.is_empty() && self.patched.iter().all(|p| *p)
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

pub struct RackPlugin;
impl Plugin for RackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatchSelection>()
            .add_message::<MachineConfigured>()
            .add_observer(click_jack)
            .add_observer(press_mount)
            .add_systems(
                Update,
                (
                    attach,
                    mount_buttons,
                    patch_engine,
                    guidance,
                    pods,
                    draw_cables,
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
        "square_vco" => vec![("vco", ModuleKind::SquareVco { freq_hz: 110. })],
        // The volume is a VCA with a built-in envelope fired by TRIG.
        "volume" => vec![
            (
                "env",
                ModuleKind::Envelope {
                    attack_s: 0.005,
                    decay_s: 0.3,
                },
            ),
            ("vca", ModuleKind::Vca { gain: 0.25 }),
        ],
        _ => vec![],
    }
}
/// Connections inside one module (the volume's envelope opens its VCA).
fn internal(module: &str) -> Vec<((&'static str, &'static str), (&'static str, &'static str))> {
    match module {
        "volume" => vec![(("env", "out"), ("vca", "cv"))],
        _ => vec![],
    }
}
/// A panel jack's engine port: (part, port).
fn jack(module: &str, port: &str) -> Option<(&'static str, &'static str)> {
    Some(match (module, port) {
        ("burbler_clock", "gate") => ("clock", "gate"),
        ("burbler_clock", "phase") => ("clock", "phase"),
        ("square_vco", "out") => ("vco", "out"),
        ("square_vco", "voct") => ("vco", "voct"),
        ("volume", "trig") => ("env", "gate"),
        ("volume", "in") => ("vca", "in"),
        ("volume", "out") => ("vca", "out"),
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
    for (cable, patched) in config.cables.iter().zip(&state.patched) {
        if *patched && let (Some(from), Some(to)) = (port(&cable.from), port(&cable.to)) {
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
                    patched: vec![false; config.cables.len()],
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
        if state.complete() && !engine.configured {
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
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    racks: Query<(&PlacedAsset, &RackState)>,
    actions: Query<Entity, With<RackActions>>,
    mut last: Local<Option<(Entity, u32, u32)>>,
) {
    let Ok(area) = actions.single() else { return };
    let Some((machine, (asset, state))) =
        open_rack(&screens).and_then(|e| racks.get(e).ok().map(|r| (e, r)))
    else {
        if last.take().is_some() {
            commands.entity(area).despawn_related::<Children>();
        }
        return;
    };
    let held: u32 = player.items.items().map(|(_, n)| n).sum();
    if *last == Some((machine, state.version, held)) {
        return;
    }
    *last = Some((machine, state.version, held));
    commands.entity(area).despawn_related::<Children>();
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    for place in config
        .places
        .iter()
        .filter(|p| !p.fixed && !state.mounted.contains_key(&p.place))
    {
        for accept in &place.accepts {
            let name = |id: &str| {
                catalog
                    .as_ref()
                    .and_then(|c| c.0.item(id))
                    .map_or(id.to_string(), |i| i.name.clone())
            };
            let have = player.items.quantity(&accept.item) > 0;
            let label = if have {
                format!(
                    "Insert {}  (uses 1 {})",
                    module_name(&accept.module),
                    name(&accept.item)
                )
            } else {
                format!(
                    "Collect a {} to make a {}",
                    name(&accept.item).to_lowercase(),
                    module_name(&accept.module)
                )
            };
            commands.entity(area).with_children(|area| {
                let mut button = area.spawn((
                    Node {
                        padding: UiRect::axes(px(14), px(8)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(1., 0.7, 0.31, if have { 0.8 } else { 0.25 })),
                    BackgroundColor(Color::srgba(1., 0.7, 0.31, if have { 0.12 } else { 0.0 })),
                    children![(
                        Text::new(label),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(17.),
                            ..default()
                        },
                        TextColor(if have {
                            Color::srgb(1., 0.7, 0.31)
                        } else {
                            Color::srgb(0.55, 0.53, 0.49)
                        }),
                    )],
                ));
                if have {
                    button.insert((MountButton {
                        machine,
                        place: place.place.clone(),
                        item: accept.item.clone(),
                        module: accept.module.clone(),
                    },));
                }
            });
        }
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
    let target = click.entity;
    let Some(mount) = buttons.get(target).ok().or_else(|| {
        parents
            .get(target)
            .ok()
            .and_then(|p| buttons.get(p.parent()).ok())
    }) else {
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

// ------------------------------------------------------------------ patching

/// Click one jack, then the other: if a spare cable joins those two, it is patched.
/// Clicking a patched jack unplugs its cable back onto the comb.
fn click_jack(
    click: On<PointerClick>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<PlayerInventory>,
) {
    let Ok(JackHit(jack)) = jacks.get(click.entity) else {
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
    let cable_at = |j: &JackRef| {
        config
            .cables
            .iter()
            .position(|c| &c.from == j || &c.to == j)
    };
    if let Some(i) = cable_at(jack).filter(|&i| state.patched[i]) {
        state.patched[i] = false;
        state.version += 1;
        selection.0 = None;
        player.feedback = "Cable back on the comb.".into();
        return;
    }
    let Some(first) = selection.0.take() else {
        selection.0 = Some(jack.clone());
        return;
    };
    let pair = config
        .cables
        .iter()
        .position(|c| (c.from == first && &c.to == jack) || (&c.from == jack && c.to == first));
    match pair {
        Some(i)
            if modules(config, &state).contains_key(&config.cables[i].from.place)
                && modules(config, &state).contains_key(&config.cables[i].to.place) =>
        {
            state.patched[i] = true;
            state.version += 1;
        }
        _ => player.feedback = "Those two jacks don't take a cable here; follow the glow.".into(),
    }
}

// ------------------------------------------------------------------ guidance

fn guidance(
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    racks: Query<(&PlacedAsset, &RackState)>,
    mut hints: Query<&mut Text, With<RackHint>>,
) {
    let Some((asset, state)) = open_rack(&screens).and_then(|e| racks.get(e).ok()) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    let missing = config
        .places
        .iter()
        .find(|p| !p.fixed && !state.mounted.contains_key(&p.place));
    let n = config.cables.len();
    let text = if let Some(place) = missing {
        format!("insert a module in the {} slot", place.kind.to_uppercase())
    } else if let Some(i) = state.patched.iter().position(|p| !p) {
        format!(
            "{} of {n}  |  {}",
            i + 1,
            config.cables[i].hint.replace('·', "|")
        )
    } else {
        config.patched_hint.replace('·', "|")
    };
    for mut hint in &mut hints {
        if hint.0 != text {
            hint.0 = text.clone();
        }
    }
}

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
            .zip(&state.patched)
            .find(|(c, p)| ready && !**p && (c.from == pod.0 || c.to == pod.0));
        let Some((cable, _)) = waiting else {
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
    comb: Query<(&CombNode, &GlobalTransform)>,
    old: Query<Entity, With<CableVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawn: Local<Option<(Entity, u32, usize)>>,
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
        .chain(comb.iter().map(|(_, t)| t.translation()))
        .fold(Vec3::ZERO, |sum, p| sum + p.abs());
    let key = (
        machine,
        state.version,
        (layout.element_sum() * 1000.) as usize,
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
    let node = |name: String| comb.iter().find(|(c, _)| c.0 == name).map(|(_, t)| *t);
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
    for (cable, patched) in config.cables.iter().zip(&state.patched) {
        let mat = materials.add(StandardMaterial {
            base_color: cable_color(&cable.color),
            perceptual_roughness: 0.6,
            ..default()
        });
        let ends = (port(&cable.from), port(&cable.to));
        if *patched {
            if let (Some(a), Some(b)) = ends {
                commands.spawn((
                    CableVisual,
                    Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS))),
                    MeshMaterial3d(mat),
                    layer.clone(),
                ));
            }
            continue;
        }
        if ready && let (Some(a), Some(b)) = ends {
            commands.spawn((
                CableVisual,
                Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS * 0.7))),
                MeshMaterial3d(ghost.clone()),
                layer.clone(),
            ));
        }
        // Spare: folded over a comb slot, both plugs hanging tip down.
        if let (Some(top), Some(pa), Some(pb)) = (
            node(format!("Comb_{slot}")),
            node(format!("Comb_{slot}_PlugA")),
            node(format!("Comb_{slot}_PlugB")),
        ) {
            let (a, b) = (pa.translation(), pb.translation());
            let c = 2. * top.translation() - (a + b) / 2. + Vec3::Y * 0.03;
            commands.spawn((
                CableVisual,
                Mesh3d(meshes.add(tube(&bezier(a, c, b), CABLE_RADIUS))),
                MeshMaterial3d(mat),
                layer.clone(),
            ));
            for plug in [pa, pb] {
                commands.spawn((
                    CableVisual,
                    WorldAssetRoot(server.load("models/cables/patch_plug.glb#Scene0")),
                    plug.compute_transform(),
                    bevy::app::Propagate(layer.clone()),
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
    fn the_burbler_sounds_only_with_both_cables_patched() {
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut state = RackState {
            patched: vec![false; config.cables.len()],
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
        // Internal env -> vca and the normalled output only: no tone.
        assert_eq!(connections(&state), 2);
        state.patched = vec![true, false];
        assert!(!state.complete());
        state.patched = vec![true, true];
        assert!(state.complete());
        assert_eq!(connections(&state), 4, "GATE->TRIG and OUT->IN added");
    }
}
