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
use stubby_recipes::Stack;

/// What the rack of one placed machine holds.
#[derive(Component, Default)]
pub struct RackState {
    /// Place -> module asset, for places the player filled (fixed places are in the profile).
    pub mounted: HashMap<String, String>,
    /// Per spare cable in the profile: patched yet?
    pub patched: Vec<bool>,
    /// The player's own cables (output jack, input jack), each one patch cable from the
    /// inventory; unplugging returns it.
    pub own: Vec<(JackRef, JackRef)>,
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
            .add_message::<MachineConfigured>()
            .add_observer(click_jack)
            .add_observer(press_mount)
            .add_systems(
                Update,
                (
                    attach,
                    mount_buttons,
                    number_keys,
                    comb_parts_pickable,
                    clear_feedback_on_open,
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
    for (cable, patched) in config.cables.iter().zip(&state.patched) {
        if *patched && let (Some(from), Some(to)) = (port(&cable.from), port(&cable.to)) {
            commands.push(Command::Connect { from, to });
        }
    }
    for (from, to) in &state.own {
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
    // The player's inventory, Satisfactory-style: a "Relevant" strip of what this rack's
    // open places accept (number keys 1-0 insert them, as does a click), then the rest
    // of what you carry, dimmed.
    let relevant: Vec<_> = relevant(config, state);
    let amber = Color::srgb(1., 0.7, 0.31);
    let muted = Color::srgb(0.55, 0.53, 0.49);
    let ink = Color::srgb(0.95, 0.94, 0.89);
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
    let row = || Node {
        column_gap: px(10),
        align_items: AlignItems::Center,
        ..default()
    };
    commands.entity(area).with_children(|area| {
        if !relevant.is_empty() {
            area.spawn(text("Relevant".into(), 15., muted));
            area.spawn((row(), Pickable::IGNORE))
                .with_children(|strip| {
                    for (index, (place, accept)) in relevant.iter().enumerate() {
                        let count = player.items.quantity(&accept.item);
                        let have = count > 0;
                        let action = if have {
                            format!(
                                "insert a {} in the {} slot",
                                module_name(&accept.module),
                                place.kind.to_uppercase()
                            )
                        } else {
                            format!(
                                "collect a {} to make a {}",
                                name(&accept.item).to_lowercase(),
                                module_name(&accept.module)
                            )
                        };
                        let mut tile = strip.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::axes(px(14), px(10)),
                                row_gap: px(4),
                                border: UiRect::all(px(1)),
                                min_width: px(260),
                                ..default()
                            },
                            BorderColor::all(if have {
                                amber.with_alpha(0.8)
                            } else {
                                muted.with_alpha(0.4)
                            }),
                            BackgroundColor(Color::srgba(0.09, 0.085, 0.075, 0.95)),
                        ));
                        tile.with_children(|tile| {
                            tile.spawn(text(
                                format!("[{}]  {}", (index + 1) % 10, name(&accept.item)),
                                19.,
                                if have { ink } else { muted },
                            ));
                            tile.spawn(text(
                                format!("x {count}"),
                                24.,
                                if have { amber } else { muted },
                            ));
                            tile.spawn(text(action, 14., if have { amber } else { muted }));
                        });
                        if have {
                            tile.insert(MountButton {
                                machine,
                                place: place.place.clone(),
                                item: accept.item.clone(),
                                module: accept.module.clone(),
                            });
                        }
                    }
                });
        }
        let others: Vec<_> = player
            .items
            .items()
            .filter(|(id, _)| !relevant.iter().any(|(_, a)| a.item == *id))
            .map(|(id, n)| format!("{} x {n}", name(id)))
            .collect();
        if !others.is_empty() {
            area.spawn(text(
                format!("Inventory:  {}", others.join("   ")),
                15.,
                muted,
            ));
        }
    });
}

/// The items this rack's open places accept, in place order (at most ten: keys 1-0).
fn relevant<'a>(
    config: &'a RackConfiguration,
    state: &RackState,
) -> Vec<(
    &'a super::machine_config::RackPlace,
    &'a super::machine_config::Accept,
)> {
    config
        .places
        .iter()
        .filter(|p| !p.fixed && !state.mounted.contains_key(&p.place))
        .flat_map(|p| p.accepts.iter().map(move |a| (p, a)))
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
    let Some((place, accept)) = relevant(config, &state)
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
    mut held: ResMut<HeldCable>,
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
    // A cable taken from the comb: its first end, then its second.
    if let Some(i) = held.cable {
        let cable = &config.cables[i];
        if &cable.from != jack && &cable.to != jack {
            player.feedback = format!(
                "The {} cable goes from {} to {}.",
                colour_name(&cable.color),
                cable.from.port.to_uppercase(),
                cable.to.port.to_uppercase()
            );
            return;
        }
        match held.first.take() {
            None => held.first = Some(jack.clone()),
            Some(first) if &first != jack => {
                state.patched[i] = true;
                state.version += 1;
                held.cable = None;
                player.feedback.clear();
            }
            Some(first) => held.first = Some(first),
        }
        return;
    }
    // A patched comb cable at this jack: back onto the comb.
    if let Some(i) = cable_at(jack).filter(|&i| state.patched[i]) {
        state.patched[i] = false;
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
    let mounted = modules(config, &state);
    // An input takes one cable; an output may feed several.
    let input_taken = |j: &JackRef| {
        state.own.iter().any(|(_, to)| to == j)
            || config
                .cables
                .iter()
                .zip(&state.patched)
                .any(|(c, p)| *p && &c.to == j)
    };
    // The comb cable these two jacks are for, if any.
    if let Some(i) = config
        .cables
        .iter()
        .position(|c| (c.from == first && &c.to == jack) || (&c.from == jack && c.to == first))
        && mounted.contains_key(&config.cables[i].from.place)
        && mounted.contains_key(&config.cables[i].to.place)
        && !input_taken(&config.cables[i].to)
    {
        state.patched[i] = true;
        state.version += 1;
        return;
    }
    // Otherwise, a cable of the player's own: an output to a free input.
    let (from, to) = match (direction(&mounted, &first), direction(&mounted, jack)) {
        (Some(true), Some(false)) => (first, jack.clone()),
        (Some(false), Some(true)) => (jack.clone(), first),
        _ => {
            player.feedback = "A cable joins an output to an input; follow the glow.".into();
            return;
        }
    };
    if input_taken(&to) {
        player.feedback = "That input already has a cable.".into();
    } else if player.items.take(PATCH_CABLE, 1).is_ok() {
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
    if !state.patched.get(*i).copied().unwrap_or(true) {
        held.cable = Some(*i);
        held.first = None;
        selection.0 = None;
    }
}

/// Dragging a hanging cable onto either of its jacks patches it.
fn drop_cable(
    drop: On<bevy::picking::events::PointerDragDrop>,
    parts: Query<&CombCable>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut held: ResMut<HeldCable>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
) {
    let (Ok(CombCable(i)), Ok(JackHit(jack))) = (parts.get(drop.dropped), jacks.get(drop.entity))
    else {
        return;
    };
    let Some(machine) = open_rack(&screens) else {
        return;
    };
    let Ok((asset, mut state)) = racks.get_mut(machine) else {
        return;
    };
    let Some(cable) = profiles.0.get(&asset.asset).and_then(|c| c.cables.get(*i)) else {
        return;
    };
    if (&cable.from == jack || &cable.to == jack) && !state.patched[*i] {
        state.patched[*i] = true;
        state.version += 1;
        held.cable = None;
        held.first = None;
    }
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

fn colour_name(colour: &str) -> &str {
    match colour {
        "verdigris" => "green",
        "rust" => "red",
        other => other,
    }
}

/// Your own cables are this item; the comb's come with the rack.
const PATCH_CABLE: &str = "patch_cable";

fn direction(mounted: &HashMap<String, String>, j: &JackRef) -> Option<bool> {
    is_output(mounted.get(&j.place)?, &j.port)
}

// ------------------------------------------------------------------ guidance

fn guidance(
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    held: Res<HeldCable>,
    player: Res<PlayerInventory>,
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
    let text = if let Some(cable) = held.cable.and_then(|i| config.cables.get(i)) {
        let other = match &held.first {
            None => format!(
                "click {}, then {}",
                cable.from.port.to_uppercase(),
                cable.to.port.to_uppercase()
            ),
            Some(first) => {
                let rest = if first == &cable.from {
                    &cable.to
                } else {
                    &cable.from
                };
                format!("now click {}", rest.port.to_uppercase())
            }
        };
        format!("{} cable in hand  |  {other}", colour_name(&cable.color))
    } else if let Some(place) = missing {
        format!("insert a module in the {} slot", place.kind.to_uppercase())
    } else if let Some(i) = state.patched.iter().position(|p| !p) {
        format!(
            "{} of {n}  |  {}",
            i + 1,
            config.cables[i].hint.replace('·', "|")
        )
    } else {
        format!(
            "{}  |  patch more with your own cables: an output, then an input",
            config.patched_hint.replace('·', "|")
        )
    };
    let text = if player.feedback.is_empty() {
        text
    } else {
        format!("{text}\n{}", player.feedback)
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
    comb: Query<(Entity, &CombNode, &GlobalTransform, &Transform)>,
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
        .chain(comb.iter().map(|(_, _, t, _)| t.translation()))
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
            // Drawn in the slot's own space, as its children, so the cable and plugs
            // follow the frame however it is fitted (its posts move after it spawns).
            let (slot_entity, (a, b)) = (top.0, (pa.1.translation, pb.1.translation));
            let c = -(a + b) / 2. + Vec3::Y * 0.03;
            let index = config
                .cables
                .iter()
                .position(|c| std::ptr::eq(c, cable))
                .unwrap_or(0);
            commands.spawn((
                CableVisual,
                CombCable(index),
                Pickable::default(),
                Mesh3d(meshes.add(tube(&bezier(a, c, b), CABLE_RADIUS * 1.6))),
                MeshMaterial3d(mat),
                layer.clone(),
                ChildOf(slot_entity),
            ));
            for plug in [pa, pb] {
                commands.spawn((
                    CableVisual,
                    CombCable(index),
                    WorldAssetRoot(server.load("models/cables/patch_plug.glb#Scene0")),
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
        // The normalled output only: no tone.
        assert_eq!(connections(&state), 1);
        state.patched = vec![true, false];
        assert!(!state.complete());
        state.patched = vec![true, true];
        assert!(state.complete());
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
            patched: vec![true; config.cables.len()],
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
