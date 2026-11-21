//! The rack every machine is configured through: what is mounted, which cables are
//! patched, the audio patch that follows, and the guidance on the rack screen
//! (add-rack-configurations, add-mudpit-burbler). Machines differ only in data: their
//! profile's `configure` block (places, what an open place accepts, the patches its voice
//! needs, the normalled output). Modules differ only in data: each module's `module.json`, read and
//! planned by `stubby_rack` (refactor-rack-modules-as-data). This file is the glue that
//! joins the rack to the rest of the game: the machine's clock, the inventory, the bug.
//!
//! Every cable is one of the player's patch cables, from the inventory, and unplugging
//! gives it back (simplify-rack-cables: no cable comb). Sound starts when every needed
//! patch is made: the engine patch is exactly the cables, so a half-patched voice is naturally silent (no CV opens the VCA, or nothing
//! reaches its input). Completing the last patch sends `MachineConfigured`, once.
use crate::{
    machine_config::{
        ButtonHit, ButtonLight, JackHit, JackRef, KnobHit, PodMesh, PortNode, Profiles,
        RackActions, RackCable, RackConfiguration, RackModuleView, StepLed,
    },
    synth_modules::{ModuleCatalog, ModuleParams, ParamProfile, SynthModule},
};
use bevy::{
    asset::RenderAssetUsages,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    mesh::Indices,
    picking::events::PointerClick,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use bevy_stubby_audio::{Synth, module_ids::ModuleIds};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::LazyLock;
use stubby_audio::{Command, ModuleId, PortRef};
use stubby_core::{AssetClock, ItemNames, PlacedAsset, Stock};
use stubby_core::{Screen, ScreenMode};
use stubby_rack::{Modules, Plan, RackPatch};

/// Every rack module's definition, validated once. The data is embedded, so a bad
/// definition fails the first rack that opens, with every error listed.
pub static RACK_MODULES: LazyLock<Modules> = LazyLock::new(|| {
    stubby_rack::shipped_modules().unwrap_or_else(|e| panic!("rack module definitions:\n{e}"))
});

/// What the rack of one placed machine holds.
#[derive(Component, Default)]
pub struct RackState {
    /// Place -> module asset, for places the player filled (fixed places are in the profile).
    pub mounted: HashMap<String, String>,
    /// Upstream layers' patches, made before the player opens the rack (add-stream-layers):
    /// free and never unplugged. Set from the profile's inherited cables.
    pub inherited: Vec<(JackRef, JackRef)>,
    /// The patched cables (output jack, input jack), each one of the player's patch
    /// cables from the inventory; unplugging returns it. Any cable joins any output to any
    /// free input, as in VCV Rack; the profile's patches are only what the screen suggests.
    pub own: Vec<(JackRef, JackRef)>,
    /// Bumped whenever the rack's contents change, so the screen redraws.
    pub version: u32,
}
impl RackState {
    /// Every patched connection: inherited and the player's.
    pub fn connections(&self) -> impl Iterator<Item = &(JackRef, JackRef)> {
        self.inherited.iter().chain(&self.own)
    }
    pub fn joins(&self, from: &JackRef, to: &JackRef) -> bool {
        self.connections().any(|(f, t)| f == from && t == to)
    }
    /// An input takes one cable; an output may feed several.
    pub fn input_taken(&self, input: &JackRef) -> bool {
        self.connections().any(|(_, t)| t == input)
    }
    /// Every patch the machine's voice needs is made, by whichever cables, directly or
    /// through patched modules (the clock's gate through a sequencer still reaches the
    /// volume's TRIG; sync-snail-shell-sequencer).
    pub fn complete(&self, config: &RackConfiguration) -> bool {
        let mut needed = config.cables.iter().filter(|c| !c.optional).peekable();
        needed.peek().is_some() && needed.all(|c| self.reaches(&c.from, &c.to))
    }
    /// Whether a place holds a module: a fixed one, or one the player mounted.
    pub fn filled(&self, config: &RackConfiguration, place: &str) -> bool {
        self.mounted.contains_key(place)
            || config
                .places
                .iter()
                .any(|p| p.place == place && p.fixed && p.module.is_some())
    }
    /// Whether the screen suggests a patch now: always, or an optional one while both its
    /// places hold a module.
    pub fn offers(&self, config: &RackConfiguration, cable: &RackCable) -> bool {
        !cable.optional
            || (self.filled(config, &cable.from.place) && self.filled(config, &cable.to.place))
    }
    /// Mount `item` as `module` in an open place, taking one from the player: all or
    /// nothing (an occupied place, or none held, takes nothing). Every way of mounting
    /// (dragging, clicking, number keys) comes here.
    pub fn mount<S: Stock>(
        &mut self,
        player: &mut S,
        place: &str,
        item: &str,
        module: &str,
    ) -> bool {
        if self.mounted.contains_key(place) || !player.take(item, 1) {
            return false;
        }
        self.mounted.insert(place.to_owned(), module.to_owned());
        self.version += 1;
        player.say(format!("Mounted a {}.", module_name(module)));
        true
    }
    /// Every place the voice needs is filled (an optional place need not be), so the
    /// screen shows the patches still to make.
    pub fn ready(&self, config: &RackConfiguration) -> bool {
        config
            .places
            .iter()
            .all(|p| p.fixed || p.optional || self.mounted.contains_key(&p.place))
    }
    /// Whether a signal from output `from` reaches input `to`: by a cable, or by a cable
    /// into a module whose output (any of it, as a module carries its inputs to its
    /// outputs) reaches `to`. A module visited once is not followed again, so loops end.
    pub fn reaches(&self, from: &JackRef, to: &JackRef) -> bool {
        let mut outputs = vec![from.clone()];
        let mut visited: Vec<&str> = Vec::new();
        while let Some(output) = outputs.pop() {
            for (out, input) in self.connections() {
                if *out != output {
                    continue;
                }
                if input == to {
                    return true;
                }
                if !visited.contains(&input.place.as_str()) {
                    visited.push(&input.place);
                    outputs.extend(
                        self.connections()
                            .filter(|(o, _)| o.place == input.place)
                            .map(|(o, _)| o.clone()),
                    );
                }
            }
        }
        false
    }
}

/// Engine modules this rack owns (not the machine's clock), and whether it has sounded.
#[derive(Component, Default)]
#[component(on_remove = retire_rack)]
pub struct RackEngine {
    owned: Vec<ModuleId>,
    /// The voice's normalled output, which the streams place in the world.
    pub voice: Option<PortRef>,
    /// Every needed patch is made, so the voice sounds and its stream carries it.
    pub sounding: bool,
    /// (place, part) -> engine module, so a knob reaches the module it turns.
    parts: HashMap<(String, String), ModuleId>,
    configured: bool,
}
impl RackEngine {
    /// The place whose module is this engine module, if the rack plays it.
    pub fn place_of(&self, module: ModuleId) -> Option<&str> {
        self.parts
            .iter()
            .find(|(_, id)| **id == module)
            .map(|((place, _), _)| place.as_str())
    }
}

fn retire_rack(world: DeferredWorld, context: HookContext) {
    let Some(rack) = world.get::<RackEngine>(context.entity) else {
        return;
    };
    let Some(synth) = world.get_resource::<Synth>() else {
        return;
    };
    for &id in &rack.owned {
        synth.send(Command::RemoveModule { id });
    }
}

/// Whether a rack's knobs meet every target setting its machine has
/// (`RackConfiguration::targets`, from the rules). With no targets it is met. The game's
/// rule for a machine reads it, beside `RackEngine::sounding`. Reflected, so the rules
/// designer's live panel shows it (add-machine-rules).
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq)]
#[reflect(Component, Default)]
pub struct RackTargets {
    pub met: bool,
    /// Each missed target and where its knob is: `seq.gate_4 at 0`.
    pub missed: Vec<String>,
    /// Whether the rack's voice sounds (every needed cable patched, a clock lent).
    pub sounding: bool,
}

/// Knob values the player has set on a rack, by (place, param). A knob not yet turned
/// is at its starting value (`initial_knob`); the clock's BPM lives in `AssetClock`.
#[derive(Component, Default)]
pub struct RackKnobs(pub HashMap<(String, String), f32>);

/// A machine's rack was completed: every needed patch made, its voice sounds.
#[derive(Message, Clone, Copy)]
pub struct MachineConfigured {
    pub machine: Entity,
}

/// The jack clicked first, waiting for the jack that completes the cable.
#[derive(Resource, Default)]
struct PatchSelection(Option<JackRef>);

/// A patched cable's tube on the rack screen and the patch it makes. Hovering it lights
/// it end to end, so you can see where it starts and where it goes (user, 2026-11-18).
#[derive(Component, Clone)]
pub struct PatchedCable {
    pub from: JackRef,
    pub to: JackRef,
    glow: LinearRgba,
}
/// The patched cable under the pointer.
#[derive(Resource, Default)]
struct HoveredCable(Option<Entity>);

/// The rack, taking items from the stock `S` (the player's inventory, in the game).
pub struct RackPlugin<S>(PhantomData<S>);
impl<S> Default for RackPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<S: Stock> Plugin for RackPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_plugins(stubby_ui::ItemTooltipPlugin)
            .init_resource::<ItemNames>()
            .init_resource::<KnobDefaults>()
            .register_type::<RackTargets>()
            .init_resource::<PatchSelection>()
            .init_resource::<HoveredCable>()
            .add_systems(
                Update,
                (
                    cancel_cable,
                    draw_scope.after(show_knobs).after(patch_engine),
                ),
            )
            .add_observer(cable_over)
            .add_observer(cable_out)
            .add_observer(drag_from_jack::<S>)
            .add_observer(drop_on_jack::<S>)
            .init_resource::<DraggedJack>()
            .add_message::<MachineConfigured>()
            .add_observer(click_jack::<S>)
            .add_observer(press_mount::<S>)
            .add_observer(start_item_drag)
            .add_observer(move_item_drag)
            .add_observer(end_item_drag)
            .add_observer(drop_item::<S>)
            .add_observer(turn_knob)
            .add_observer(scroll_knob)
            .init_resource::<KnobUnderPointer>()
            .add_observer(knob_over)
            .add_observer(knob_out)
            .add_systems(Startup, spawn_knob_readout)
            .add_systems(
                Update,
                show_knob_readout.after(stubby_core::MachineInteractionSet),
            )
            .add_observer(press_button)
            .add_systems(
                Update,
                (
                    attach,
                    apply_knobs,
                    check_targets,
                    show_knobs,
                    mount_buttons::<S>,
                    number_keys::<S>,
                    light_hovered_cable,
                    live_parts_unpickable,
                    tint_plugs,
                    load_snaps,
                    snap_on_plug,
                    rack_ghost,
                    clear_feedback_on_open::<S>,
                    patch_engine,
                    step_lights,
                    publish_voice,
                    pods,
                    draw_cables,
                    live_cable,
                    snap_release::<S>,
                )
                    .chain()
                    .after(stubby_core::MachineInteractionSet),
            );
    }
}

// ------------------------------------------------------------------ modules → engine

/// Starting values the game supplies for a module's knobs, joining the rack to the
/// world: a machine's clock starts at its seeded tempo, and a Square VCO, made from a
/// lightning bug's square wave, starts at the bug's pitch. Rack data never names these.
fn starting_value(
    module: &str,
    knob: &str,
    clock: Option<&AssetClock>,
    defaults: &KnobDefaults,
) -> Option<f32> {
    let k = RACK_MODULES.knob(module, knob)?;
    let part = RACK_MODULES
        .get(module)?
        .parts
        .iter()
        .find(|p| p.part == k.part)?;
    let value = match (part.kind, k.param) {
        ("Clock", "bpm") => clock.map(|c| c.bpm),
        (kind, param) => defaults
            .0
            .get(&(kind.to_owned(), param.to_owned()))
            .copied(),
    }?;
    // A clicking knob starts on its grid (a bug's pitch lands on its nearest note), so
    // the engine, the knob and its readout agree.
    Some(if k.clicks() { k.snap(value) } else { value })
}

/// Starting knob values the game supplies, by engine kind and parameter (a Square VCO
/// made from a lightning bug's square wave starts at the bug's pitch). Rack data never
/// names where they come from.
#[derive(Resource, Default, Clone)]
pub struct KnobDefaults(pub HashMap<(String, String), f32>);

/// A knob's value before the player turns it: the game's starting value, else the
/// module's default.
fn initial_knob(
    module: &str,
    knob: &str,
    clock: Option<&AssetClock>,
    defaults: &KnobDefaults,
) -> Option<f32> {
    starting_value(module, knob, clock, defaults)
        .or_else(|| RACK_MODULES.knob(module, knob).map(|k| k.default))
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

/// The engine patch for a rack, planned by `stubby_rack` from what the rack shows. The
/// machine's own clock stands in for every Clock part, so the rack, the bubbles, the
/// stream and the lamp keep one tempo.
fn plan(
    config: &RackConfiguration,
    state: &RackState,
    knobs: &RackKnobs,
    clock: Option<&AssetClock>,
    defaults: &KnobDefaults,
    fresh: impl FnMut() -> Option<ModuleId>,
) -> Plan {
    let mut modules = modules(config, state);
    // A borrowed clock with nothing lent is an empty place.
    if clock.is_none() {
        for place in config.places.iter().filter(|p| p.borrowed) {
            modules.remove(&place.place);
        }
    }
    let mut provided = HashMap::new();
    if let Some(clock) = clock {
        for (place, module) in &modules {
            for part in RACK_MODULES.get(module).into_iter().flat_map(|m| &m.parts) {
                if part.kind == "Clock" {
                    provided.insert((place.clone(), part.part.clone()), clock.engine);
                }
            }
        }
    }
    let cables: Vec<_> = state.connections().cloned().collect();
    RACK_MODULES.plan(
        &RackPatch {
            modules: &modules,
            cables: &cables,
            normalled_output: config.normalled_output.as_ref(),
            provided: &provided,
        },
        |place, knob| {
            knobs
                .0
                .get(&(place.to_owned(), knob.to_owned()))
                .copied()
                .or_else(|| {
                    let module = modules.get(place)?;
                    starting_value(module, knob, clock, defaults)
                })
        },
        fresh,
    )
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
                    inherited: config
                        .cables
                        .iter()
                        .filter(|c| c.inherited)
                        .map(|c| (c.from.clone(), c.to.clone()))
                        .collect(),
                    ..default()
                },
                RackEngine::default(),
                RackKnobs::default(),
                RackTargets::default(),
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
            &RackKnobs,
            &mut RackEngine,
            Option<&AssetClock>,
        ),
        Changed<RackState>,
    >,
    mut configured: MessageWriter<MachineConfigured>,
    defaults: Res<KnobDefaults>,
) {
    for (machine, asset, state, knobs, mut engine, clock) in &mut racks {
        let Some(config) = profiles.0.get(&asset.asset) else {
            continue;
        };
        let plan = plan(config, state, knobs, clock, &defaults, || ids.allocate());
        if let Some(synth) = &synth {
            for id in engine.owned.drain(..) {
                synth.send(Command::RemoveModule { id });
            }
            for command in plan.commands {
                synth.send(command);
            }
        }
        engine.owned = plan.owned;
        engine.parts = plan.parts;
        engine.voice = plan.voice;
        // A borrowed clock must be lent for the voice to sound.
        let lent = clock.is_some() || !config.places.iter().any(|p| p.borrowed);
        engine.sounding = lent && state.complete(config);
        if engine.sounding && !engine.configured {
            engine.configured = true;
            configured.write(MachineConfigured { machine });
        }
    }
}

/// Whether each rack's knobs meet its profile's targets. A target's place must hold a
/// module (a borrowed clock with nothing lent holds none), and an unturned knob is at its
/// starting value.
fn check_targets(
    profiles: Res<Profiles>,
    defaults: Res<KnobDefaults>,
    mut racks: Query<(
        &PlacedAsset,
        &RackState,
        &RackKnobs,
        &RackEngine,
        Option<&AssetClock>,
        &mut RackTargets,
    )>,
    mut reported: Local<std::collections::HashSet<String>>,
) {
    for (asset, state, knobs, engine, clock, mut targets) in &mut racks {
        let Some(config) = profiles.0.get(&asset.asset) else {
            continue;
        };
        let mut modules = modules(config, state);
        if clock.is_none() {
            for place in config.places.iter().filter(|p| p.borrowed) {
                modules.remove(&place.place);
            }
        }
        let missed: Vec<String> = config
            .targets
            .iter()
            .filter_map(|target| {
                let at = format!("{}.{}", target.place, target.knob);
                let Some(module) = modules.get(&target.place) else {
                    return Some(format!("{at} (empty place)"));
                };
                let value = knobs
                    .0
                    .get(&(target.place.clone(), target.knob.clone()))
                    .copied()
                    .or_else(|| initial_knob(module, &target.knob, clock, &defaults));
                match value {
                    Some(value) if target.met(value) => None,
                    Some(value) => Some(format!("{at} at {value}")),
                    None => Some(at),
                }
            })
            .collect();
        let next = RackTargets {
            met: missed.is_empty(),
            missed,
            sounding: engine.sounding,
        };
        let logged = next.met != targets.met || next.missed != targets.missed;
        if logged || reported.insert(asset.instance.clone()) {
            // For tuning machines: which targets a rack misses, each time that changes.
            if config.targets.is_empty() {
                // Nothing to report.
            } else if next.met {
                info!("{}: every target met", asset.instance);
            } else {
                info!(
                    "{}: targets missed: {}",
                    asset.instance,
                    next.missed.join(", ")
                );
            }
        }
        if *targets != next {
            *targets = next;
        }
    }
}

/// Each rack's voice, as other systems see it (`stubby_core::MachineVoice`).
fn publish_voice(
    mut commands: Commands,
    profiles: Res<Profiles>,
    racks: Query<
        (
            Entity,
            &PlacedAsset,
            &RackEngine,
            Option<&stubby_core::MachineVoice>,
        ),
        Changed<RackEngine>,
    >,
) {
    for (machine, asset, engine, current) in &racks {
        let voice = stubby_core::MachineVoice {
            output: engine.voice,
            sounding: engine.sounding,
            hearing_radius_metres: profiles
                .0
                .get(&asset.asset)
                .map_or(0., |c| c.hearing_radius_metres),
        };
        if current.is_none_or(|c| c.output != voice.output || c.sounding != voice.sounding) {
            commands.entity(machine).insert(voice);
        }
    }
}

fn open_rack(screens: &ScreenMode) -> Option<Entity> {
    match screens.open {
        Some(Screen::MachineConfiguration(e)) => Some(e),
        _ => None,
    }
}

// ------------------------------------------------------------------ knobs

/// Pixels of vertical drag that turn a knob through its whole range (VCV Rack's feel).
const KNOB_DRAG_PX: f32 = 240.;
/// One scroll-wheel notch turns a knob this fraction of its range.
const KNOB_SCROLL_STEP: f32 = 1. / 48.;

/// A value's position along its knob, 0..1, following the knob's taper.
fn knob_norm(value: f32, p: &ParamProfile) -> f32 {
    let v = value.clamp(p.min, p.max);
    if p.taper == "log" && p.min > 0.0 {
        (v / p.min).ln() / (p.max / p.min).ln()
    } else {
        (v - p.min) / (p.max - p.min)
    }
}
/// The value at a position along a knob, 0..1.
fn knob_value(norm: f32, p: &ParamProfile) -> f32 {
    let n = norm.clamp(0., 1.);
    if p.taper == "log" && p.min > 0.0 {
        p.min * (p.max / p.min).powf(n)
    } else {
        p.min + (p.max - p.min) * n
    }
}

/// How far a knob turns: smoothly by a fraction of its range, or by whole clicks on a
/// knob with a grid (a semitone, a 24 BPM step), so small moves never snap back.
#[derive(Clone, Copy)]
enum Turn {
    Smooth(f32),
    Clicks(i32),
}

/// Pixels of drag per click on a knob with a grid.
const CLICK_PX: f32 = 10.;

/// Turn the open rack's knob `hit` (up is positive).
fn turn_open_knob(
    hit: &KnobHit,
    turn: Turn,
    profiles: &Profiles,
    catalog: &ModuleCatalog,
    racks: &mut Query<(
        &PlacedAsset,
        &RackState,
        &mut RackKnobs,
        Option<&AssetClock>,
    )>,
    machine: Entity,
    defaults: &KnobDefaults,
) {
    let Ok((asset, state, mut knobs, clock)) = racks.get_mut(machine) else {
        return;
    };
    let Some(config) = profiles.0.get(&asset.asset) else {
        return;
    };
    // A borrowed clock is the neighbour's: it is shown, never turned here.
    if config
        .places
        .iter()
        .any(|p| p.place == hit.place && p.borrowed)
    {
        return;
    }
    let Some(module) = modules(config, state).remove(&hit.place) else {
        return;
    };
    let Some(param) = catalog
        .0
        .get(&module)
        .and_then(|m| m.params.iter().find(|p| p.param == hit.param))
    else {
        return;
    };
    let key = (hit.place.clone(), hit.param.clone());
    let Some(current) = knobs
        .0
        .get(&key)
        .copied()
        .or_else(|| initial_knob(&module, &hit.param, clock, defaults))
    else {
        return;
    };
    let grid = RACK_MODULES
        .knob(&module, &hit.param)
        .filter(|knob| knob.clicks());
    let mut value = match (turn, grid) {
        // A knob with a grid (the BPM's 24 steps, a VCO's semitones) clicks.
        (Turn::Clicks(steps), Some(knob)) => knob.nudge(current, steps),
        (Turn::Clicks(_), None) => current,
        (Turn::Smooth(turn), _) => knob_value(knob_norm(current, param) + turn, param),
    };
    // The knob steps as the engine does (whole BPM, semitones on a sequencer step), so
    // the panel and the sound agree.
    if let Some(spec) = engine_spec(&module, &hit.param) {
        value = spec.conform(value);
    }
    if value != current {
        knobs.0.insert(key, value);
    }
}

/// The engine parameter a module's control sets, with its range and quantum.
fn engine_spec(module: &str, control: &str) -> Option<&'static stubby_audio::ParamSpec> {
    let knob = RACK_MODULES.knob(module, control)?;
    let kind = RACK_MODULES
        .0
        .get(module)?
        .parts
        .iter()
        .find(|part| part.part == knob.part)?
        .kind;
    stubby_audio::ModuleKind::default_of(kind)?.param(knob.param)
}

/// Click a lit push button (a sequencer's step key) to light its step or clear it, as on
/// Intellijel's Steppy and KORG's SQ-1.
fn press_button(
    click: On<PointerClick>,
    hits: Query<&ButtonHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut racks: Query<(&PlacedAsset, &RackState, &mut RackKnobs)>,
) {
    let (Ok(hit), Some(machine)) = (hits.get(click.entity), open_rack(&screens)) else {
        return;
    };
    let Ok((asset, state, mut knobs)) = racks.get_mut(machine) else {
        return;
    };
    let Some(module) = profiles
        .0
        .get(&asset.asset)
        .and_then(|config| modules(config, state).remove(&hit.place))
    else {
        return;
    };
    let key = (hit.place.clone(), hit.param.clone());
    let current = knobs.0.get(&key).copied().unwrap_or_else(|| {
        RACK_MODULES
            .knob(&module, &hit.param)
            .map_or(0., |knob| knob.default)
    });
    knobs.0.insert(key, if current >= 0.5 { 0. } else { 1. });
}

/// The step keys' lights: a key is lit while its step is on (selected); the red light
/// beside it is lit while the sequencer's heard playhead is on its step, switched at the
/// instant each step is heard.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn step_lights(
    mut commands: Commands,
    mut steps: MessageReader<bevy_stubby_audio::StepTick>,
    screens: Res<ScreenMode>,
    racks: Query<(Entity, &RackEngine, &RackKnobs)>,
    lights: Query<
        (
            Entity,
            Option<&ButtonLight>,
            Option<&StepLed>,
            &MeshMaterial3d<StandardMaterial>,
            Option<&OwnLight>,
        ),
        Or<(With<ButtonLight>, With<StepLed>)>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pending: Local<Vec<(Entity, String, u8, Option<std::time::Instant>)>>,
    mut playheads: Local<HashMap<(Entity, String), u8>>,
) {
    // Which machine and place each step belongs to.
    for tick in steps.read() {
        for (machine, engine, _) in &racks {
            if let Some(((place, _), _)) = engine.parts.iter().find(|(_, id)| **id == tick.module) {
                pending.push((machine, place.clone(), tick.step, tick.at));
            }
        }
    }
    let now = std::time::Instant::now();
    pending.retain(|(machine, place, step, at)| {
        if at.is_some_and(|at| at > now) {
            return true;
        }
        playheads.insert((*machine, place.clone()), *step);
        false
    });
    let Some(machine) = open_rack(&screens) else {
        return;
    };
    let Ok((_, _, knobs)) = racks.get(machine) else {
        return;
    };
    for (entity, key, led, material, own) in &lights {
        // Each key gets its own copy of the cap's glowing material, so it can light alone.
        let Some(own) = own else {
            if let Some(cap) = materials.get(&material.0).cloned() {
                let on = cap.emissive;
                let handle = materials.add(cap);
                commands
                    .entity(entity)
                    .insert((MeshMaterial3d(handle), OwnLight { on }));
            }
            continue;
        };
        let lit = match (key, led) {
            (Some(key), _) => knobs
                .0
                .get(&(key.place.clone(), key.param.clone()))
                .is_some_and(|v| *v >= 0.5),
            (None, Some(led)) => playheads.get(&(machine, led.place.clone())) == Some(&led.step),
            (None, None) => false,
        };
        let glow = match (lit, led.is_some()) {
            (false, _) => LinearRgba::BLACK,
            // The playhead light is red, whatever its material held when copied.
            (true, true) => PLAYHEAD_RED,
            (true, false) => own.on,
        };
        if let Some(mut cap) = materials.get_mut(&material.0)
            && cap.emissive != glow
        {
            cap.emissive = glow;
        }
    }
}

/// The step playhead light's glow when its step plays.
const PLAYHEAD_RED: LinearRgba = LinearRgba::rgb(8., 0.9, 0.4);

/// A step key's own material, and its glow when lit.
#[derive(Component)]
struct OwnLight {
    on: LinearRgba,
}

/// The knob under the pointer, and whether the pointer is still over it (a drag keeps
/// its knob after the pointer leaves).
#[derive(Resource, Default)]
struct KnobUnderPointer(Option<(Entity, bool)>);

/// The knob readout (VCV Rack's knob tooltip): the knob's name and value beside the
/// pointer, with a note name on a semitone knob.
#[derive(Component)]
struct KnobReadout;

fn knob_over(
    over: On<bevy::picking::events::PointerOver>,
    hits: Query<(), With<KnobHit>>,
    mut under: ResMut<KnobUnderPointer>,
) {
    if hits.contains(over.entity) {
        under.0 = Some((over.entity, true));
    }
}

fn knob_out(out: On<bevy::picking::events::PointerOut>, mut under: ResMut<KnobUnderPointer>) {
    if let Some((entity, inside)) = under.0.as_mut()
        && *entity == out.entity
    {
        *inside = false;
    }
}

fn spawn_knob_readout(mut commands: Commands) {
    commands
        .spawn((
            KnobReadout,
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(stubby_ui::style::PANEL),
            BorderColor::all(stubby_ui::style::EDGE),
            GlobalZIndex(200),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_child(stubby_ui::style::label("", 14., stubby_ui::style::INK));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_knob_readout(
    screens: Res<ScreenMode>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    mut under: ResMut<KnobUnderPointer>,
    hits: Query<&KnobHit>,
    profiles: Res<Profiles>,
    defaults: Res<KnobDefaults>,
    racks: Query<(&PlacedAsset, &RackState, &RackKnobs, Option<&AssetClock>)>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut readouts: Query<(&mut Node, &mut Visibility, &Children), With<KnobReadout>>,
    mut texts: Query<&mut Text>,
) {
    // A knob let go of outside it is no longer shown.
    if let Some((_, false)) = under.0
        && !buttons.is_some_and(|b| b.pressed(MouseButton::Left))
    {
        under.0 = None;
    }
    let shown = (|| {
        let machine = open_rack(&screens)?;
        let (entity, _) = under.0?;
        let hit = hits.get(entity).ok()?;
        let (asset, state, knobs, clock) = racks.get(machine).ok()?;
        let module = modules(profiles.0.get(&asset.asset)?, state).remove(&hit.place)?;
        let knob = RACK_MODULES.knob(&module, &hit.param)?;
        let value = knobs
            .0
            .get(&(hit.place.clone(), hit.param.clone()))
            .copied()
            .or_else(|| initial_knob(&module, &hit.param, clock, &defaults))?;
        let cursor = windows.single().ok()?.cursor_position()?;
        Some((
            format!(
                "{}  {}",
                hit.param.replace('_', " ").to_uppercase(),
                knob.readout(value)
            ),
            cursor,
        ))
    })();
    for (mut node, mut visibility, children) in &mut readouts {
        let Some((line, cursor)) = &shown else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        node.left = px(cursor.x + 16.);
        node.top = px(cursor.y + 12.);
        for child in children {
            if let Ok(mut text) = texts.get_mut(*child)
                && text.0 != *line
            {
                text.0.clone_from(line);
            }
        }
    }
}

/// Drag a knob up to turn it up, down to turn it down (VCV Rack).
fn turn_knob(
    drag: On<bevy::picking::events::PointerDrag>,
    hits: Query<&KnobHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    catalog: Res<ModuleCatalog>,
    mut racks: Query<(
        &PlacedAsset,
        &RackState,
        &mut RackKnobs,
        Option<&AssetClock>,
    )>,
    defaults: Res<KnobDefaults>,
    mut dragged: Local<(Option<Entity>, f32)>,
) {
    let (Ok(hit), Some(machine)) = (hits.get(drag.entity), open_rack(&screens)) else {
        return;
    };
    let grid = racks
        .get(machine)
        .ok()
        .and_then(|(asset, state, _, _)| {
            modules(profiles.0.get(&asset.asset)?, state).remove(&hit.place)
        })
        .and_then(|module| RACK_MODULES.knob(&module, &hit.param).cloned())
        .is_some_and(|knob| knob.clicks());
    let turn = if grid {
        // Drag builds up pixels; each `CLICK_PX` of it is one click.
        if dragged.0 != Some(drag.entity) {
            *dragged = (Some(drag.entity), 0.);
        }
        dragged.1 += -drag.delta.y;
        let clicks = (dragged.1 / CLICK_PX).trunc();
        dragged.1 -= clicks * CLICK_PX;
        if clicks == 0. {
            return;
        }
        Turn::Clicks(clicks as i32)
    } else {
        Turn::Smooth(-drag.delta.y / KNOB_DRAG_PX)
    };
    turn_open_knob(
        hit, turn, &profiles, &catalog, &mut racks, machine, &defaults,
    );
}

/// The scroll wheel over a knob turns it in small steps.
fn scroll_knob(
    scroll: On<bevy::picking::events::PointerScroll>,
    hits: Query<&KnobHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    catalog: Res<ModuleCatalog>,
    mut racks: Query<(
        &PlacedAsset,
        &RackState,
        &mut RackKnobs,
        Option<&AssetClock>,
    )>,
    defaults: Res<KnobDefaults>,
) {
    let (Ok(hit), Some(machine)) = (hits.get(scroll.entity), open_rack(&screens)) else {
        return;
    };
    let notches = match scroll.unit {
        bevy::input::mouse::MouseScrollUnit::Line => scroll.y,
        bevy::input::mouse::MouseScrollUnit::Pixel => scroll.y / 40.,
    };
    let grid = racks
        .get(machine)
        .ok()
        .and_then(|(asset, state, _, _)| {
            modules(profiles.0.get(&asset.asset)?, state).remove(&hit.place)
        })
        .and_then(|module| RACK_MODULES.knob(&module, &hit.param).cloned())
        .is_some_and(|knob| knob.clicks());
    // One notch is one click on a knob with a grid, a small turn otherwise.
    let turn = if grid {
        Turn::Clicks(notches.signum() as i32 * notches.abs().round().max(1.) as i32)
    } else {
        Turn::Smooth(notches.signum() * notches.abs().max(1.) * KNOB_SCROLL_STEP)
    };
    if notches != 0. {
        turn_open_knob(
            hit, turn, &profiles, &catalog, &mut racks, machine, &defaults,
        );
    }
}

/// Send turned knobs to the engine. The clock's BPM is the machine's tempo
/// (`AssetClock`), so the bubbles, the stream and the lamp follow it too.
fn apply_knobs(
    profiles: Res<Profiles>,
    synth: Option<Res<Synth>>,
    mut racks: Query<
        (
            &PlacedAsset,
            &RackState,
            &RackKnobs,
            &RackEngine,
            Option<&mut AssetClock>,
        ),
        Changed<RackKnobs>,
    >,
) {
    for (asset, state, knobs, engine, mut clock) in &mut racks {
        let Some(config) = profiles.0.get(&asset.asset) else {
            continue;
        };
        let modules = modules(config, state);
        for ((place, param), &value) in &knobs.0 {
            let Some(module) = modules.get(place) else {
                continue;
            };
            let Some(knob) = RACK_MODULES.knob(module, param) else {
                continue;
            };
            let Some(&id) = engine.parts.get(&(place.clone(), knob.part.clone())) else {
                continue;
            };
            // The machine's clock is its tempo: retune it here and everything follows.
            if let Some(clock) = clock.as_mut()
                && clock.engine == id
                && clock.bpm != value
            {
                clock.bpm = value;
            }
            if let Some(synth) = &synth {
                synth.send(Command::SetParam {
                    module: id,
                    param: knob.param,
                    value,
                });
            }
        }
    }
}

/// The open rack's modules show their knobs' values (knob angle, the BPM display).
fn show_knobs(
    defaults: Res<KnobDefaults>,
    screens: Res<ScreenMode>,
    catalog: Res<ModuleCatalog>,
    racks: Query<(&RackKnobs, Option<&AssetClock>)>,
    mut views: Query<(&RackModuleView, &SynthModule, &mut ModuleParams)>,
) {
    let Some((knobs, clock)) = open_rack(&screens).and_then(|m| racks.get(m).ok()) else {
        return;
    };
    for (view, module, mut params) in &mut views {
        let Some(profile) = catalog.0.get(&module.asset) else {
            continue;
        };
        for p in &profile.params {
            let value = knobs
                .0
                .get(&(view.place.clone(), p.param.clone()))
                .copied()
                .or_else(|| initial_knob(&module.asset, &p.param, clock, &defaults));
            if let Some(value) = value
                && params.0.get(&p.param) != Some(&value)
            {
                params.0.insert(p.param.clone(), value);
            }
        }
    }
}

#[derive(Component)]
struct ScopeTrace(Handle<Mesh>);

/// The rack view owns the display mesh. The audio tap belongs to the engine part
/// playing in this place, which may be replaced when the rack is repatched.
///
/// The scope needs no adjusting (user, 2026-11-20): it is a triggered scope that shows the
/// incoming wave's shape, refreshed once a beat of the rack's clock. Each beat it finds the
/// wave's period in a 0.1 s capture, starts on a rising zero crossing and shows two cycles,
/// scaled to fit, holding them until the next beat (and through a closed gate). Its TIME and
/// SCALE knobs turn to show the capture and the fit, as the rack plays what it shows.
#[allow(clippy::type_complexity)] // The rack's engine parts, clock and knobs.
fn draw_scope(
    mut commands: Commands,
    synth: Option<Res<Synth>>,
    screens: Res<ScreenMode>,
    catalog: Res<ModuleCatalog>,
    mut racks: Query<(&RackEngine, Option<&AssetClock>, Option<&mut RackKnobs>)>,
    views: Query<(Entity, &RackModuleView, &SynthModule, &ModuleParams)>,
    children: Query<&Children>,
    panels: Query<(&Name, Option<&ScopeTrace>)>,
    assets: Option<(ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)>,
    mut samples: Local<Vec<f32>>,
    mut held: Local<HashMap<Entity, (f32, Vec<f32>, f32)>>,
    time: Res<Time<Real>>,
) {
    let Some(thread) = synth.as_ref().and_then(|synth| synth.thread()) else {
        return;
    };
    let Some((rack, clock, mut knobs)) =
        open_rack(&screens).and_then(|machine| racks.get_mut(machine).ok())
    else {
        return;
    };
    let Some((mut meshes, mut materials)) = assets else {
        return;
    };
    // One beat of the rack's clock: its own, else its clock place's BPM knob.
    let bpm = clock.map(|c| c.bpm).or_else(|| {
        knobs
            .as_ref()
            .and_then(|k| k.0.iter().find(|((_, p), _)| p == "bpm").map(|(_, v)| *v))
    });
    let beat = bpm.filter(|b| *b > 0.).map(|b| 60. / b);
    for (root, view, module, params) in &views {
        let Some(screen) = catalog
            .0
            .get(&module.asset)
            .and_then(|profile| profile.screen.as_ref())
        else {
            continue;
        };
        let Some(definition) = RACK_MODULES.get(&module.asset) else {
            continue;
        };
        let Some(part) = definition.parts.iter().find(|part| part.kind == "Scope") else {
            continue;
        };
        let Some((panel, trace)) = children.iter_descendants(root).find_map(|entity| {
            let (name, trace) = panels.get(entity).ok()?;
            (name.as_str() == screen.node).then_some((entity, trace))
        }) else {
            continue;
        };

        // A 0.1 s capture at near full resolution, read once a beat.
        let now = time.elapsed_secs();
        let entry = held
            .entry(panel)
            .or_insert_with(|| (f32::NEG_INFINITY, vec![0.0; screen.samples.max(2)], 0.5));
        if now - entry.0 >= beat.unwrap_or(0.5) {
            samples.resize(SCOPE_CAPTURE_POINTS, 0.0);
            let id = rack.parts.get(&(view.place.clone(), part.part.clone()));
            let ok = id.is_some_and(|id| thread.read_scope(*id, &mut samples));
            if ok && let Some((cycles, scale)) = triggered_cycles(&samples, screen.samples.max(2)) {
                *entry = (now, cycles, scale);
            }
        }
        let (_, shown, scale) = entry.clone();
        if let Some(knobs) = knobs.as_mut() {
            let mut set = |param: &str, value: f32| {
                let key = (view.place.clone(), param.to_owned());
                let old = knobs.0.get(&key).copied().unwrap_or(f32::NAN);
                if !(old.is_finite() && ((old - value) / value).abs() < 0.05) {
                    knobs.0.insert(key, value);
                }
            };
            set("time", SCOPE_CAPTURE_S);
            set("scale", scale);
        }
        let _ = params;
        let positions = waveform_ribbon(&shown, screen.width_metres, screen.height_metres, scale);
        if let Some(ScopeTrace(handle)) = trace {
            if let Some(mut mesh) = meshes.get_mut(handle) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            }
        } else {
            let handle = meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleStrip,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions),
            );
            let [r, g, b] = screen.colour_linear;
            commands.spawn((
                ChildOf(panel),
                Mesh3d(handle.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::linear_rgb(r, g, b),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                // The exported Blender empty points local +Y out of the panel.
                Transform::from_xyz(0.0, 0.002, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                bevy::camera::visibility::NoFrustumCulling,
                bevy::camera::visibility::RenderLayers::layer(crate::machine_config::HUD_LAYER),
                Pickable::IGNORE,
            ));
            commands.entity(panel).insert(ScopeTrace(handle));
        }
    }
}

/// The scope's capture: how long and at how many points it is read.
const SCOPE_CAPTURE_S: f32 = 0.1;
const SCOPE_CAPTURE_POINTS: usize = 4800; // 0.1 s at 48 kHz: every sample

/// Two cycles of the wave in `capture`, starting on a rising zero crossing, resampled to
/// `points`, with the scale that fits its peak to 0.45 of the screen either side; `None` when
/// the capture is silent or has no steady period (then the scope keeps what it showed).
pub fn triggered_cycles(capture: &[f32], points: usize) -> Option<(Vec<f32>, f32)> {
    let n = capture.len();
    let mean = capture.iter().sum::<f32>() / n.max(1) as f32;
    let x: Vec<f32> = capture.iter().map(|s| s - mean).collect();
    let peak = x.iter().fold(0f32, |m, s| m.max(s.abs()));
    if n < 64 || peak < 1e-3 {
        return None;
    }
    // The period: after the autocorrelation first falls below zero, the shortest lag that
    // matches nearly as well as the best (normalised per overlap, so long lags are fair):
    // the fundamental, not a partial's echo.
    let r = |lag: usize| {
        x[..n - lag]
            .iter()
            .zip(&x[lag..])
            .map(|(a, b)| a * b)
            .sum::<f32>()
            / (n - lag) as f32
    };
    let r0 = r(0).max(1e-12);
    let mut lag = 2;
    while lag < n / 3 && r(lag) > 0.0 {
        lag += 1;
    }
    let scores: Vec<(usize, f32)> = (lag..n / 3).map(|l| (l, r(l))).collect();
    let best = scores.iter().map(|s| s.1).fold(f32::MIN, f32::max);
    if best / r0 < 0.3 {
        return None;
    }
    let period = scores
        .windows(3)
        .find(|w| w[1].1 >= w[0].1 && w[1].1 >= w[2].1 && w[1].1 >= 0.9 * best)
        .map(|w| w[1].0)?;
    // The latest rising zero crossing that leaves room for two cycles.
    let span = 2 * period;
    let start = (1..n.saturating_sub(span))
        .rev()
        .find(|&i| x[i - 1] < 0.0 && x[i] >= 0.0)?;
    let cycles: Vec<f32> = (0..points)
        .map(|k| {
            let at = start as f32 + span as f32 * k as f32 / (points - 1) as f32;
            let i = at.floor() as usize;
            let f = at - i as f32;
            x[i.min(n - 1)] * (1.0 - f) + x[(i + 1).min(n - 1)] * f
        })
        .collect();
    let shown_peak = cycles.iter().fold(0f32, |m, s| m.max(s.abs())).max(1e-3);
    Some((cycles, (0.45 / shown_peak).clamp(0.1, 4.0)))
}

fn waveform_ribbon(samples: &[f32], width: f32, height: f32, scale: f32) -> Vec<[f32; 3]> {
    let half_stroke = 0.0022;
    let vertical_limit = (height * 0.5 - half_stroke).max(0.0);
    let step = width / (samples.len() - 1) as f32;
    let points: Vec<Vec2> = samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            Vec2::new(
                i as f32 * step - width * 0.5,
                (sample * scale * height).clamp(-vertical_limit, vertical_limit),
            )
        })
        .collect();
    // Widen across the line, not only up and down, so steep edges keep their width.
    (0..points.len())
        .flat_map(|i| {
            let a = points[i.saturating_sub(1)];
            let b = points[(i + 1).min(points.len() - 1)];
            let normal = (b - a).perp().normalize_or(Vec2::Y) * half_stroke;
            let p = points[i];
            [
                [p.x - normal.x, p.y - normal.y, 0.0],
                [p.x + normal.x, p.y + normal.y, 0.0],
            ]
        })
        .collect()
}

// ------------------------------------------------------------------ mounting

#[derive(Component)]
pub struct MountButton {
    machine: Entity,
    place: String,
    item: String,
    module: String,
}

/// One button per open place the player can fill from the inventory.
fn mount_buttons<S: Stock>(
    mut commands: Commands,
    server: Res<AssetServer>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    player: Res<S>,
    names: Res<ItemNames>,
    racks: Query<(&PlacedAsset, &RackState)>,
    actions: Query<Entity, With<RackActions>>,
    boxes: Query<Entity, With<crate::machine_config::RackInventory>>,
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
    let held: u32 = player.boxes().iter().map(|(_, n)| n).sum();
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
    let relevant: Vec<_> = relevant(config, state, &*player);
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
    let name = |id: &str| names.name(id);
    let mount = |place: &crate::machine_config::RackPlace,
                 accept: &crate::machine_config::Accept| {
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
            let count = player.quantity(&accept.item);
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
                    stubby_ui::ItemName(name(&accept.item)),
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
        let held = player.boxes();
        for index in 0..player.box_count() {
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
    held: &impl Stock,
) -> Vec<(
    &'a crate::machine_config::RackPlace,
    &'a crate::machine_config::Accept,
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
fn number_keys<S: Stock>(
    keys: Res<ButtonInput<KeyCode>>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut player: ResMut<S>,
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
    let Some((place, accept)) = relevant(config, &state, &*player)
        .get(index)
        .map(|(p, a)| (p.place.clone(), (*a).clone()))
    else {
        return;
    };
    state.mount(&mut *player, &place, &accept.item, &accept.module);
}

fn module_name(module: &str) -> String {
    match module {
        "square_vco" => "Square VCO".into(),
        other => other.replace('_', " "),
    }
}

/// Clicking a mount button (or its label) takes the item and mounts the module.
fn press_mount<S: Stock>(
    click: On<PointerClick>,
    buttons: Query<&MountButton>,
    parents: Query<&ChildOf>,
    mut player: ResMut<S>,
    mut racks: Query<&mut RackState>,
) {
    let Some(mount) = mount_of(click.entity, &buttons, &parents) else {
        return;
    };
    let Ok(mut state) = racks.get_mut(mount.machine) else {
        return;
    };
    state.mount(&mut *player, &mount.place, &mount.item, &mount.module);
}

use stubby_ui::{icon_slot, item_icon};

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
    hud: Query<(&Camera, &GlobalTransform), With<stubby_core::HudCamera>>,
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
        bevy::camera::visibility::RenderLayers::layer(crate::machine_config::HUD_LAYER),
        Pickable::IGNORE,
    ));
}

/// Dropping a Relevant item on its empty slot mounts the module, Satisfactory-style.
fn drop_item<S: Stock>(
    drop: On<bevy::picking::events::PointerDragDrop>,
    slots: Query<&crate::machine_config::SlotHit>,
    buttons: Query<&MountButton>,
    parents: Query<&ChildOf>,
    mut player: ResMut<S>,
    mut racks: Query<&mut RackState>,
) {
    let (Ok(slot), Some(mount)) = (
        slots.get(drop.entity),
        mount_of(drop.dropped, &buttons, &parents),
    ) else {
        return;
    };
    if slot.0 != mount.place {
        player.say(format!(
            "A {} does not fit there.",
            module_name(&mount.module)
        ));
        return;
    }
    let Ok(mut state) = racks.get_mut(mount.machine) else {
        return;
    };
    state.mount(&mut *player, &mount.place, &mount.item, &mount.module);
}

// ------------------------------------------------------------------ patching

/// Click one jack, then the other: one of your patch cables joins them. Clicking a
/// patched jack unplugs its cable back into your inventory.
#[allow(clippy::too_many_arguments)] // A click reads the rack, its cables and the inventory.
fn click_jack<S: Stock>(
    click: On<PointerClick>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut dragged: ResMut<DraggedJack>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<S>,
) {
    // A drag that started here already pressed this jack; its click is the same gesture.
    if dragged.0.take() == Some(click.entity) {
        return;
    }
    let Ok(JackHit(jack)) = jacks.get(click.entity) else {
        return;
    };
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut *player)
    });
}

/// VCV Rack's gesture: drag from a jack and drop on another jack.
/// Starting a drag presses the jack it starts on; dropping presses the jack under it.
#[allow(clippy::too_many_arguments)] // As `click_jack`.
fn drag_from_jack<S: Stock>(
    drag: On<bevy::picking::events::PointerDragStart>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut dragged: ResMut<DraggedJack>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<S>,
) {
    let Ok(JackHit(jack)) = jacks.get(drag.entity) else {
        return;
    };
    dragged.0 = Some(drag.entity);
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut *player)
    });
}

#[allow(clippy::too_many_arguments)] // As `click_jack`.
fn drop_on_jack<S: Stock>(
    drop: On<bevy::picking::events::PointerDragDrop>,
    jacks: Query<&JackHit>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    mut selection: ResMut<PatchSelection>,
    mut racks: Query<(&PlacedAsset, &mut RackState)>,
    mut player: ResMut<S>,
) {
    let (Ok(JackHit(jack)), true) = (jacks.get(drop.entity), jacks.contains(drop.dropped)) else {
        return;
    };
    if drop.entity == drop.dropped {
        return;
    }
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        press_jack(jack, config, state, &mut selection, &mut *player)
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
fn press_jack<S: Stock>(
    jack: &JackRef,
    config: &RackConfiguration,
    state: &mut RackState,
    selection: &mut PatchSelection,
    player: &mut S,
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
    // A cable at this jack: unplugged, back into the inventory.
    if let Some(i) = state.own.iter().position(|(a, b)| a == jack || b == jack) {
        // No room for it: the cable stays patched rather than being lost.
        if !player.give(PATCH_CABLE, 1) {
            return;
        }
        state.own.remove(i);
        state.version += 1;
        selection.0 = None;
        let left = player.quantity(PATCH_CABLE);
        player.say(format!("Patch cable back in your inventory ({left})."));
        return;
    }
    let Some(first) = selection.0.take() else {
        if player.quantity(PATCH_CABLE) == 0 {
            player.say("You have no patch cables.".into());
            return;
        }
        selection.0 = Some(jack.clone());
        return;
    };
    if &first == jack {
        selection.0 = Some(first);
        return;
    }
    let Some((from, to)) = orient(&first, jack, state) else {
        // Not an output and a free input: the cable stays in hand.
        selection.0 = Some(first);
        return;
    };
    if player.take(PATCH_CABLE, 1) {
        state.own.push((from, to));
        state.version += 1;
        player.say(format!(
            "Patched a cable ({} left).",
            player.quantity(PATCH_CABLE)
        ));
    } else {
        player.say("You have no patch cables.".into());
    }
}

/// Hovering a patched cable lights it end to end.
fn cable_over(
    over: On<bevy::picking::events::PointerOver>,
    cables: Query<(), With<PatchedCable>>,
    mut hovered: ResMut<HoveredCable>,
) {
    if cables.contains(over.entity) {
        hovered.0 = Some(over.entity);
    }
}
fn cable_out(out: On<bevy::picking::events::PointerOut>, mut hovered: ResMut<HoveredCable>) {
    if hovered.0 == Some(out.entity) {
        hovered.0 = None;
    }
}
fn light_hovered_cable(
    hovered: Res<HoveredCable>,
    cables: Query<(Entity, &PatchedCable, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, cable, material) in &cables {
        let glow = if hovered.0 == Some(entity) {
            cable.glow
        } else {
            LinearRgba::BLACK
        };
        if let Some(mut m) = materials.get_mut(&material.0)
            && m.emissive != glow
        {
            m.emissive = glow;
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

/// A WAV as (sample rate, one buffer per channel), decoded by `hound`.
fn decode_wav(bytes: &[u8]) -> Option<(u32, Vec<Vec<f32>>)> {
    let mut reader = hound::WavReader::new(std::io::Cursor::new(bytes)).ok()?;
    let spec = reader.spec();
    let channels = usize::from(spec.channels.max(1));
    let scale = 1. / (1u32 << (spec.bits_per_sample.max(1) - 1)) as f32;
    let mut out = vec![Vec::new(); channels];
    for (i, sample) in reader.samples::<i32>().enumerate() {
        out[i % channels].push(sample.ok()? as f32 * scale);
    }
    Some((spec.sample_rate, out))
}

/// Load the snaps into the engine once its thread is up.
fn load_snaps(synth: Option<Res<Synth>>, mut done: Local<bool>) {
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

/// A cable going into a jack snaps.
fn snap_on_plug(
    synth: Option<Res<Synth>>,
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
fn clear_feedback_on_open<S: Stock>(
    mut opened: MessageReader<crate::machine_config::MachineConfigurationOpened>,
    mut player: ResMut<S>,
    mut selection: ResMut<PatchSelection>,
) {
    if opened.read().next().is_some() {
        player.say(String::new());
        selection.0 = None;
    }
}

/// Every cable is this item, from the player's inventory.
const PATCH_CABLE: &str = "patch_cable";

fn direction(mounted: &HashMap<String, String>, j: &JackRef) -> Option<bool> {
    RACK_MODULES.jack_is_output(mounted.get(&j.place)?, &j.port)
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

/// Pods pulse in the colour of the patch still waiting for their jack; others rest. While
/// a cable's first end is in one end of a known patch, only the patch's other end shows,
/// blinking, so the player sees where the cable goes (user, 2026-11-18).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn pods(
    time: Res<Time>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    selection: Res<PatchSelection>,
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
    let ready = state.ready(config);
    let open = |c: &&RackCable| ready && state.offers(config, c) && !state.joins(&c.from, &c.to);
    // The known patch the cable in hand starts, and the jack it should end in.
    let partner = selection.0.as_ref().and_then(|first| {
        config.cables.iter().filter(open).find_map(|c| {
            if &c.from == first {
                Some((c, c.to.clone()))
            } else if &c.to == first {
                Some((c, c.from.clone()))
            } else {
                None
            }
        })
    });
    let t = time.elapsed_secs();
    let pulse = match partner {
        // A quick, bright blink.
        Some(_) => (0.5 + 0.5 * (t * std::f32::consts::TAU / 0.45).sin()).powi(2) * 1.6,
        None => 0.5 + 0.5 * (t * std::f32::consts::TAU / 1.2).sin(),
    };
    for (entity, (pod, mut material, mut visibility)) in entities.iter().zip(meshes.iter_mut()) {
        let waiting = match &partner {
            Some((cable, jack)) => (jack == &pod.0).then_some(*cable),
            None => config
                .cables
                .iter()
                .filter(open)
                .find(|c| c.from == pod.0 || c.to == pod.0),
        };
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
    selection: &PatchSelection,
    ports: &Query<(&PortNode, &GlobalTransform)>,
) -> Option<(JackRef, Vec3)> {
    let anchor = selection.0.as_ref();
    anchor?;
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
fn snap_release<S: Stock>(
    mut inputs: MessageReader<bevy::picking::pointer::PointerInput>,
    hovers: Res<bevy::picking::hover::HoverMap>,
    jacks: Query<(), With<JackHit>>,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    hud: Query<(&Camera, &GlobalTransform), With<stubby_core::HudCamera>>,
    ports: Query<(&PortNode, &GlobalTransform)>,
    mut hand: (ResMut<PatchSelection>, ResMut<S>),
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
    let (selection, player) = (&mut *hand.0, &mut *hand.1);
    on_rack(&screens, &profiles, &mut racks, |config, state| {
        if let Some((jack, _)) = snap_target(at, camera, eye, config, state, selection, &ports) {
            press_jack(&jack, config, state, selection, player);
        } else {
            // Let go over nothing it can go into: the cable is dropped, as in VCV Rack
            // (user, 2026-11-18: a started cable could not be cancelled).
            selection.0 = None;
        }
    });
}

/// Right click drops the cable in hand, too.
fn cancel_cable(
    buttons: Res<ButtonInput<MouseButton>>,
    screens: Res<ScreenMode>,
    mut selection: ResMut<PatchSelection>,
) {
    if open_rack(&screens).is_some()
        && buttons.just_pressed(MouseButton::Right)
        && selection.0.is_some()
    {
        selection.0 = None;
    }
}

/// The cable in hand, drawn as VCV Rack draws a cable being patched, and alive: it is
/// a small spring simulation, so it swings after the pointer instead of snapping.
/// Once one end is in a jack, it runs from that jack to the pointer with the same
/// elastic sag as a patched cable, the sag springing after every move. The plugs are the cable's own patch_plug models.
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
    selection: Res<PatchSelection>,
    racks: Query<(&PlacedAsset, &RackState)>,
    ports: Query<(&PortNode, &GlobalTransform)>,
    hud: Query<(&Camera, &GlobalTransform), With<stubby_core::HudCamera>>,
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
    // The anchored end: the jack the cable's first end went into.
    let anchor = selection.0.as_ref().and_then(port);
    let carrying = anchor.is_some();
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
        && let Some((_, at)) = snap_target(screen, camera, eye, config, state, &selection, &ports)
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
    // Starting a known patch, the cable is that patch's colour.
    let colour = selection
        .0
        .as_ref()
        .and_then(|first| {
            config.cables.iter().find(|c| {
                (&c.from == first || &c.to == first)
                    && rack.is_some_and(|(_, s)| !s.joins(&c.from, &c.to))
            })
        })
        .map_or("bone".to_string(), |c| c.color.clone());
    let layer = bevy::camera::visibility::RenderLayers::layer(crate::machine_config::HUD_LAYER);
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
                            WorldAssetRoot(server.load("cables/patch_plug/patch_plug.glb#Scene0")),
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

/// Patched cables between their jacks, and a ghost for each patch still to make. A cable
/// making a known patch takes that patch's colour; others are bone. Redrawn when the rack
/// changes or its parts first appear.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw_cables(
    mut commands: Commands,
    screens: Res<ScreenMode>,
    profiles: Res<Profiles>,
    racks: Query<(&PlacedAsset, &RackState)>,
    ports: Query<(&PortNode, &GlobalTransform)>,
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
    let layer = bevy::camera::visibility::RenderLayers::layer(crate::machine_config::HUD_LAYER);
    let ready = state.ready(config);
    let ghost = materials.add(StandardMaterial {
        base_color: Color::srgba(1., 0.83, 0.6, 0.22),
        emissive: LinearRgba::rgb(1.5, 1.2, 0.8),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    for (from, to) in state.connections() {
        let Some((a, b)) = port(from).zip(port(to)) else {
            continue;
        };
        let colour = config
            .cables
            .iter()
            .find(|c| &c.from == from && &c.to == to)
            .map_or("bone", |c| c.color.as_str());
        let base = cable_color(colour);
        // Each cable its own material, so hovering lights it alone.
        let material = materials.add(StandardMaterial {
            base_color: base,
            perceptual_roughness: 0.6,
            ..default()
        });
        commands.spawn((
            CableVisual,
            PatchedCable {
                from: from.clone(),
                to: to.clone(),
                glow: base.to_linear() * 6.,
            },
            Pickable::default(),
            Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS * 1.3))),
            MeshMaterial3d(material),
            layer.clone(),
        ));
    }
    // The ghost of each suggested patch, until some cable makes it.
    for cable in &config.cables {
        if ready
            && state.offers(config, cable)
            && !state.joins(&cable.from, &cable.to)
            && let (Some(a), Some(b)) = (port(&cable.from), port(&cable.to))
        {
            commands.spawn((
                CableVisual,
                Mesh3d(meshes.add(tube(&cable_curve(a, b), CABLE_RADIUS * 0.7))),
                MeshMaterial3d(ghost.clone()),
                layer.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scope_triggers_on_two_cycles_of_the_wave() {
        // A saw of period 100 samples, behind a silent stretch (a closed gate).
        let capture: Vec<f32> = (0..4096)
            .map(|i| {
                if i < 1000 {
                    0.0
                } else {
                    ((i % 100) as f32 / 50.0) - 1.0
                }
            })
            .collect();
        let (cycles, scale) = triggered_cycles(&capture, 256).expect("a steady wave");
        assert_eq!(cycles.len(), 256);
        // It starts on a rising zero crossing and spans two periods: zero again mid-screen.
        assert!(cycles[0].abs() < 0.05, "{}", cycles[0]);
        assert!((scale - 0.45).abs() < 0.05, "{scale}");
        assert!(triggered_cycles(&vec![0.0; 4096], 256).is_none());
    }
    use stubby_audio::ModuleKind;

    fn machine_clock(engine: ModuleId) -> AssetClock {
        AssetClock {
            engine,
            clock: 0,
            bpm: 120.,
        }
    }

    #[test]
    fn the_burbler_sounds_only_with_both_cables_patched() {
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut state = RackState::default();
        state.mounted.insert("vco".into(), "square_vco".into());
        let connections = |state: &RackState| {
            let mut next = 100;
            plan(
                config,
                state,
                &RackKnobs::default(),
                Some(&machine_clock(ModuleId(1))),
                &KnobDefaults::default(),
                || {
                    next += 1;
                    Some(ModuleId(next))
                },
            )
            .commands
            .into_iter()
            .filter(|c| matches!(c, Command::Connect { .. }))
            .count()
        };
        // The normalled output only: no tone.
        assert_eq!(connections(&state), 1);
        let link = |c: &crate::machine_config::RackCable| (c.from.clone(), c.to.clone());
        state.own.push(link(&config.cables[0]));
        assert!(!state.complete(config));
        state.own.push(link(&config.cables[1]));
        assert!(state.complete(config));
        assert_eq!(connections(&state), 3, "GATE->TRIG and OUT->IN added");
    }

    #[test]
    fn a_rack_offers_only_the_places_its_machine_needs() {
        // The burbler's work is a heartbeat: clock, VCO and volume, no sequencer place and
        // no optional patch (user, 2026-11-18). The campfire's needs a sequencer.
        let profiles = Profiles::default();
        let burbler = &profiles.0["mudpit_burbler"];
        assert!(burbler.places.iter().all(|p| p.kind != "sequencer"));
        assert!(burbler.cables.iter().all(|c| !c.optional));
        let campfire = &profiles.0["campfire"];
        assert!(campfire.places.iter().any(|p| p.kind == "sequencer"));
    }

    #[derive(Resource, Default)]
    struct Shells(u32);
    impl Stock for Shells {
        fn quantity(&self, _: &str) -> u32 {
            self.0
        }
        fn boxes(&self) -> Vec<(String, u32)> {
            vec![]
        }
        fn box_count(&self) -> usize {
            16
        }
        fn take(&mut self, _: &str, count: u32) -> bool {
            let Some(left) = self.0.checked_sub(count) else {
                return false;
            };
            self.0 = left;
            true
        }
        fn give(&mut self, _: &str, count: u32) -> bool {
            self.0 += count;
            true
        }
        fn say(&mut self, _: String) {}
    }

    #[test]
    fn cables_come_from_the_inventory_and_go_back_to_it() {
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut state = RackState::default();
        state.mounted.insert("vco".into(), "square_vco".into());
        let mut selection = PatchSelection::default();
        let jack = |place: &str, port: &str| JackRef {
            place: place.into(),
            port: port.into(),
        };
        // None held: a jack takes no cable in hand.
        let mut none = Shells(0);
        press_jack(
            &jack("vco", "out"),
            config,
            &mut state,
            &mut selection,
            &mut none,
        );
        assert!(selection.0.is_none(), "no cables: nothing in hand");
        // OUT then IN: one cable patched, taken from the inventory.
        let mut cables = Shells(2);
        press_jack(
            &jack("vco", "out"),
            config,
            &mut state,
            &mut selection,
            &mut cables,
        );
        press_jack(
            &jack("volume", "in"),
            config,
            &mut state,
            &mut selection,
            &mut cables,
        );
        assert_eq!(state.own, vec![(jack("vco", "out"), jack("volume", "in"))]);
        assert_eq!(cables.0, 1);
        // Clicking either end unplugs it, back into the inventory.
        press_jack(
            &jack("volume", "in"),
            config,
            &mut state,
            &mut selection,
            &mut cables,
        );
        assert!(state.own.is_empty());
        assert_eq!(cables.0, 2);
    }

    #[test]
    fn mounting_a_held_shell_takes_one_and_a_failed_mount_takes_nothing() {
        let mut state = RackState::default();
        let mut none = Shells(0);
        assert!(!state.mount(&mut none, "seq", "snail_shell", "snail_sequencer"));
        assert!(
            !state.mounted.contains_key("seq"),
            "nothing held: nothing mounted"
        );
        let mut two = Shells(2);
        assert!(state.mount(&mut two, "seq", "snail_shell", "snail_sequencer"));
        assert_eq!(two.0, 1, "one shell taken");
        assert_eq!(state.mounted["seq"], "snail_sequencer");
        assert!(!state.mount(&mut two, "seq", "snail_shell", "snail_sequencer"));
        assert_eq!(two.0, 1, "an occupied place takes nothing");
    }

    #[test]
    fn vcos_start_straight_up_on_a_note() {
        // User, 2026-11-18: VCOs are quantized and start with the knob straight up.
        let catalog = ModuleCatalog::default();
        // Every VCO: a saw or triangle that started on C4 met the purifier's C4 target untouched.
        for vco in ["square_vco", "sine_vco", "saw_vco", "triangle_vco"] {
            let knob = RACK_MODULES.knob(vco, "freq").expect("a frequency knob");
            assert!(knob.quantize.is_some(), "{vco} is quantized");
            let start = initial_knob(vco, "freq", None, &KnobDefaults::default()).unwrap();
            assert!(
                knob.readout(start).starts_with("C#3"),
                "{vco}: {}",
                knob.readout(start)
            );
            let p = &catalog.0[vco].params[0];
            assert!(
                (knob_norm(start, p) - 0.5).abs() < 0.01,
                "{vco} points straight up"
            );
        }
    }

    #[test]
    fn a_bugs_pitch_starts_the_vco_on_its_nearest_note() {
        // A square VCO's frequency knob is quantized to semitones (add-reed-water-purifier):
        // a bug's 300 Hz becomes D4, so the engine, the knob and its readout agree.
        let defaults = KnobDefaults([(("SquareVco".to_owned(), "freq".to_owned()), 300.)].into());
        let start = initial_knob("square_vco", "freq", None, &defaults).unwrap();
        assert!((start - 293.66).abs() < 0.01, "{start}");
        let knob = RACK_MODULES.knob("square_vco", "freq").unwrap();
        assert!(knob.readout(start).starts_with("D4"));
    }

    #[test]
    fn knobs_set_the_modules_they_turn() {
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut state = RackState::default();
        state.mounted.insert("vco".into(), "square_vco".into());
        let mut knobs = RackKnobs::default();
        knobs.0.insert(("vco".into(), "freq".into()), 440.);
        let volume = config
            .places
            .iter()
            .find(|p| p.module.as_deref() == Some("volume"))
            .expect("the burbler has a volume")
            .place
            .clone();
        knobs.0.insert((volume.clone(), "gain".into()), 0.8);
        let mut next = 100;
        let plan = plan(
            config,
            &state,
            &knobs,
            Some(&machine_clock(ModuleId(1))),
            &KnobDefaults::default(),
            || {
                next += 1;
                Some(ModuleId(next))
            },
        );
        let kind = |id: ModuleId| {
            plan.commands.iter().find_map(|c| match c {
                Command::AddModule { id: i, kind } if *i == id => Some(*kind),
                _ => None,
            })
        };
        assert!(matches!(
            kind(plan.parts[&("vco".to_owned(), "vco".to_owned())]),
            Some(ModuleKind::SquareVco { freq_hz }) if freq_hz == 440.
        ));
        assert!(matches!(
            kind(plan.parts[&(volume, "vca".to_owned())]),
            Some(ModuleKind::Vca { gain }) if gain == 0.8
        ));
        assert_eq!(
            plan.parts[&("clock".to_owned(), "clock".to_owned())],
            ModuleId(1)
        );
        // A knob's position and its value agree, linear and log.
        let catalog = ModuleCatalog::default();
        for (module, value) in [
            ("square_vco", 440.),
            ("volume", 0.8),
            ("burbler_clock", 120.),
        ] {
            let p = &catalog.0[module].params[0];
            assert!((knob_value(knob_norm(value, p), p) - value).abs() < 1e-2 * value);
        }
    }

    /// Development only: renders the burbler's patch to WAVs for listening (not run in
    /// normal builds). `STUBBY_RENDER_DIR=<dir> cargo test -p stubby_game --lib
    /// render_burbler -- --ignored` writes `burbler-patched.wav` (its two patches).
    #[test]
    #[ignore]
    fn render_burbler() {
        use stubby_audio::{Engine, EngineConfig};
        let dir = std::env::var("STUBBY_RENDER_DIR").unwrap_or_else(|_| ".".into());
        let profiles = Profiles::default();
        let config = &profiles.0["mudpit_burbler"];
        let mut patched = RackState {
            own: config
                .cables
                .iter()
                .map(|c| (c.from.clone(), c.to.clone()))
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
            let machine = machine_clock(clock);
            for command in plan(
                config,
                &state,
                &RackKnobs::default(),
                Some(&machine),
                &KnobDefaults::default(),
                || {
                    next += 1;
                    Some(ModuleId(next))
                },
            )
            .commands
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

#[cfg(test)]
mod path_tests {
    use super::*;
    fn jack(place: &str, port: &str) -> JackRef {
        JackRef {
            place: place.into(),
            port: port.into(),
        }
    }

    #[test]
    fn a_patch_through_a_sequencer_still_reaches_its_input() {
        let mut state = RackState::default();
        // Clock GATE → sequencer CLOCK, sequencer GATE → volume TRIG.
        state
            .own
            .push((jack("clock", "gate"), jack("seq", "clock")));
        state
            .own
            .push((jack("seq", "gate"), jack("volume", "trig")));
        assert!(state.reaches(&jack("clock", "gate"), &jack("volume", "trig")));
        assert!(!state.reaches(&jack("clock", "gate"), &jack("volume", "in")));
        // A loop back into the sequencer ends.
        state.own.push((jack("seq", "cv"), jack("seq", "reset")));
        assert!(!state.reaches(&jack("vco", "out"), &jack("volume", "in")));
    }
}
