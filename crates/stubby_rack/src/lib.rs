//! Rack modules as data, and the engine patch a rack plans from them.
//!
//! Each module that can sit in a rack has one definition, `module.json`, beside its
//! art-generated panel profile, `asset.json`: the engine parts it is built from, wiring
//! inside it, the engine port behind each panel jack and the engine parameter behind
//! each knob. [`Modules::load`] reads them strictly, reports every error at once, and
//! checks each definition against its panel. [`Modules::plan`] turns a rack's mounted
//! modules, knob values and patched cables into engine commands.
//!
//! This crate is one independent system: it depends on the audio engine only and never
//! names items, creatures, machines or Bevy. Whoever composes the game supplies
//! starting knob values and any engine modules it already owns.

mod plan;

pub use plan::{Jack, Plan, RackPatch};

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use stubby_audio::ModuleKind;

/// One module's files: its definition and its panel profile.
pub struct Source<'a> {
    /// Where the definition lives, for error reports (`assets/modules/volume/module.json`).
    pub file: &'a str,
    pub definition: &'a str,
    pub panel: &'a str,
}

/// The modules shipped with the game, embedded at build time so the game and the
/// Jackdaw adapter read identical data.
pub fn shipped_sources() -> Vec<Source<'static>> {
    macro_rules! module {
        ($id:literal) => {
            Source {
                file: concat!("assets/modules/", $id, "/module.json"),
                definition: include_str!(concat!("../../../assets/modules/", $id, "/module.json")),
                panel: include_str!(concat!("../../../assets/modules/", $id, "/asset.json")),
            }
        };
    }
    vec![
        module!("burbler_clock"),
        module!("clock_lamp"),
        module!("sine_vco"),
        module!("square_vco"),
        module!("saw_vco"),
        module!("triangle_vco"),
        module!("mixer"),
        module!("vcf"),
        module!("scope"),
        module!("sample_player"),
        module!("snail_sequencer"),
        module!("stream_blank"),
        module!("volume"),
    ]
}

/// The shipped modules, validated.
pub fn shipped_modules() -> Result<Modules, RackErrors> {
    Modules::load(&shipped_sources())
}

// ------------------------------------------------------------------ files

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    id: String,
    parts: Vec<PartDefinition>,
    #[serde(default)]
    internal: Vec<WireDefinition>,
    jacks: BTreeMap<String, String>,
    #[serde(default)]
    knobs: BTreeMap<String, KnobDefinition>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartDefinition {
    part: String,
    kind: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDefinition {
    from: String,
    to: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KnobDefinition {
    param: String,
    default: f32,
    /// The knob clicks through multiples of this (a game rule on top of the engine's own
    /// quantum): tempos on the 24 BPM grid (user, 2026-11-17).
    #[serde(default)]
    step: Option<f32>,
    /// The knob clicks through a musical grid instead (`"semitone"`: equal-tempered
    /// notes, A4 = 440 Hz), so pitches are easy to set (add-reed-water-purifier).
    #[serde(default)]
    quantize: Option<Quantize>,
}

/// A musical grid a knob clicks through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quantize {
    /// Equal-tempered semitones, in Hz (A4 = 440 Hz).
    Semitone,
}

/// Semitones from A4 to a frequency.
fn semitones_from_a4(hz: f32) -> f32 {
    12. * (hz / 440.).log2()
}
fn hz_at(semitones_from_a4: f32) -> f32 {
    440. * 2f32.powf(semitones_from_a4 / 12.)
}

/// A frequency's nearest note, as its name and octave (`C4`, `F#2`).
pub fn note_name(hz: f32) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    // Semitones from C0 (A4 is 57 above it).
    let n = semitones_from_a4(hz).round() as i32 + 57;
    format!("{}{}", NAMES[n.rem_euclid(12) as usize], n.div_euclid(12))
}

/// A setting a machine needs for its work (add-reed-water-purifier "target settings"):
/// a knob in a place at a note (`"note": "C2"`, for a semitone knob) or at a value
/// (`"value": 1`, a step gate on). Machine profiles list them under `configure.targets`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub place: String,
    pub knob: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub value: Option<f32>,
}
impl Target {
    /// Whether the knob's value meets the target: the same note, or the same value.
    pub fn met(&self, value: f32) -> bool {
        match (&self.note, self.value) {
            (Some(note), None) => note_name(value) == *note,
            (None, Some(want)) => (value - want).abs() <= 1e-3 * want.abs().max(1.),
            _ => false,
        }
    }
    /// What is wrong with the target as written, if anything.
    pub fn problem(&self) -> Option<String> {
        match (&self.note, self.value) {
            (Some(_), Some(_)) | (None, None) => {
                Some("give a `note` or a `value`, not both".into())
            }
            (Some(note), None) if note_hz(note).map(note_name).as_ref() != Some(note) => {
                Some(format!("`{note}` is not a note like C4 or F#2"))
            }
            _ => None,
        }
    }
}

/// A note's frequency (`C4` → 261.63 Hz), if it is written as a note.
pub fn note_hz(name: &str) -> Option<f32> {
    let split = name.find(|c: char| c == '-' || c.is_ascii_digit())?;
    let (letter, octave) = name.split_at(split);
    let octave: i32 = octave.parse().ok()?;
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let index = NAMES.iter().position(|n| *n == letter)? as i32;
    Some(hz_at((octave * 12 + index - 57) as f32))
}

/// The parts of `asset.json` the rack reads. The panel belongs to art and carries much
/// more (model, lights, display), so unknown fields are ignored here.
#[derive(Deserialize)]
struct Panel {
    id: String,
    #[serde(default)]
    ports: Vec<PanelPort>,
    #[serde(default)]
    params: Vec<PanelParam>,
    /// Lit push buttons (a sequencer's step keys): each toggles a parameter between 0
    /// and 1, bound in `module.json` under `knobs` like any control.
    #[serde(default)]
    buttons: Vec<PanelButton>,
}
#[derive(Deserialize)]
struct PanelButton {
    param: String,
}
impl Panel {
    /// Every control on the panel: its knobs, and its buttons as 0..1 controls.
    fn controls(&self) -> Vec<PanelParam> {
        self.params
            .iter()
            .map(|p| PanelParam {
                param: p.param.clone(),
                min: p.min,
                max: p.max,
            })
            .chain(self.buttons.iter().map(|b| PanelParam {
                param: b.param.clone(),
                min: 0.,
                max: 1.,
            }))
            .collect()
    }
}

#[derive(Deserialize)]
struct PanelPort {
    port: String,
    direction: String,
}

#[derive(Deserialize)]
struct PanelParam {
    param: String,
    min: f32,
    max: f32,
}

// ------------------------------------------------------------------ validated model

/// A validated rack module.
#[derive(Debug, Clone)]
pub struct Module {
    pub id: String,
    /// Engine parts by name, each with the kind it is built as.
    pub parts: Vec<Part>,
    /// Connections inside the module: (part, output) -> (part, input).
    pub internal: Vec<((String, &'static str), (String, &'static str))>,
    /// Panel jack -> the engine port behind it.
    pub jacks: BTreeMap<String, JackBinding>,
    /// Panel knob -> the engine parameter it sets.
    pub knobs: BTreeMap<String, Knob>,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub part: String,
    /// The engine kind's name (`"Vca"`); [`ModuleKind::from_params`] builds it.
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JackBinding {
    pub part: String,
    pub port: &'static str,
    pub output: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Knob {
    pub part: String,
    pub param: &'static str,
    pub default: f32,
    /// The knob's travel: where the panel's range and the engine's overlap.
    pub min: f32,
    pub max: f32,
    /// Values are multiples of this, inside `min..=max`.
    pub step: Option<f32>,
    /// Values are notes on this grid, inside `min..=max`.
    pub quantize: Option<Quantize>,
}
impl Knob {
    /// Whether the knob clicks (a step or a musical grid) rather than turning smoothly.
    pub fn clicks(&self) -> bool {
        self.quantize.is_some() || self.step.is_some_and(|step| step > 0.)
    }
    /// Snap a value to the knob's grid, staying inside its travel.
    pub fn snap(&self, value: f32) -> f32 {
        self.nudge(value, 0)
    }
    /// The value `steps` clicks from `value` (0 snaps it), staying inside the travel.
    pub fn nudge(&self, value: f32, steps: i32) -> f32 {
        if self.quantize == Some(Quantize::Semitone) && self.min > 0. {
            let lowest = semitones_from_a4(self.min).ceil();
            let highest = semitones_from_a4(self.max).floor();
            let at = semitones_from_a4(value.max(f32::MIN_POSITIVE)).round() + steps as f32;
            return hz_at(at.clamp(lowest, highest));
        }
        let Some(step) = self.step.filter(|step| *step > 0.) else {
            return value.clamp(self.min, self.max);
        };
        let lowest = (self.min / step).ceil() * step;
        let highest = (self.max / step).floor() * step;
        (((value / step).round() + steps as f32) * step).clamp(lowest, highest)
    }
    /// The value as the player reads it: a note name on a semitone knob.
    pub fn readout(&self, value: f32) -> String {
        match self.quantize {
            Some(Quantize::Semitone) => format!("{}  {value:.2} Hz", note_name(value)),
            None if self.step.is_some() => format!("{value:.0}"),
            None => format!("{value:.2}"),
        }
    }
}

/// Every validated module, by ID.
#[derive(Debug, Clone, Default)]
pub struct Modules(pub BTreeMap<String, Module>);

impl Modules {
    pub fn get(&self, id: &str) -> Option<&Module> {
        self.0.get(id)
    }

    /// Whether a module's panel jack is an output (`Some(true)`) or an input.
    pub fn jack_is_output(&self, module: &str, jack: &str) -> Option<bool> {
        Some(self.get(module)?.jacks.get(jack)?.output)
    }

    /// The engine parameter a module's knob sets, and its default.
    pub fn knob(&self, module: &str, knob: &str) -> Option<&Knob> {
        self.get(module)?.knobs.get(knob)
    }

    /// Load and validate definitions, checking each against its panel. Every error is
    /// reported, each with its file, module and field.
    pub fn load(sources: &[Source]) -> Result<Modules, RackErrors> {
        let mut errors = Vec::new();
        let mut modules = BTreeMap::new();
        for source in sources {
            let mut report = |module: &str, field: &str, message: String| {
                errors.push(RackError {
                    file: source.file.to_owned(),
                    module: module.to_owned(),
                    field: field.to_owned(),
                    message,
                })
            };
            let definition: Definition = match serde_json::from_str(source.definition) {
                Ok(d) => d,
                Err(e) => {
                    report("", "", e.to_string());
                    continue;
                }
            };
            let panel: Panel = match serde_json::from_str(source.panel) {
                Ok(p) => p,
                Err(e) => {
                    report(&definition.id, "asset.json", e.to_string());
                    continue;
                }
            };
            let module = validate(&definition, &panel, &mut report);
            if modules.contains_key(&definition.id) {
                report(&definition.id, "id", "duplicate module ID".into());
            } else if let Some(module) = module {
                modules.insert(definition.id.clone(), module);
            }
        }
        if errors.is_empty() {
            Ok(Modules(modules))
        } else {
            Err(RackErrors(errors))
        }
    }
}

/// `"part.name"` split in two.
fn split(reference: &str) -> Option<(&str, &str)> {
    reference.split_once('.')
}

fn validate(
    definition: &Definition,
    panel: &Panel,
    report: &mut impl FnMut(&str, &str, String),
) -> Option<Module> {
    let id = definition.id.as_str();
    let mut failed = false;
    let mut fail = |field: &str, message: String| {
        failed = true;
        report(id, field, message);
    };
    if panel.id != definition.id {
        fail("id", format!("the panel beside it is `{}`", panel.id));
    }

    // Parts and their engine kinds.
    let mut kinds: BTreeMap<&str, ModuleKind> = BTreeMap::new();
    let mut parts = Vec::new();
    for (i, part) in definition.parts.iter().enumerate() {
        let field = format!("parts[{i}]");
        if kinds.contains_key(part.part.as_str()) {
            fail(&field, format!("duplicate part `{}`", part.part));
            continue;
        }
        match ModuleKind::default_of(&part.kind) {
            Some(kind) => {
                kinds.insert(&part.part, kind);
                parts.push(Part {
                    part: part.part.clone(),
                    kind: kind.name(),
                });
            }
            None => fail(
                &format!("{field}.kind"),
                format!(
                    "`{}` is not an engine module kind the rack can build",
                    part.kind
                ),
            ),
        }
    }
    // An engine port, its direction (true = output).
    let port = |reference: &str| -> Result<(String, &'static str, bool), String> {
        let (part, port) = split(reference).ok_or(format!("`{reference}` is not `part.port`"))?;
        let kind = kinds.get(part).ok_or(format!("no part `{part}`"))?;
        if let Some(p) = kind.outputs().iter().find(|p| **p == port) {
            return Ok((part.to_owned(), p, true));
        }
        if let Some(p) = kind.inputs().iter().find(|p| **p == port) {
            return Ok((part.to_owned(), p, false));
        }
        Err(format!("{} has no port `{port}`", kind.name()))
    };

    // Internal wiring: output to input.
    let mut internal = Vec::new();
    for (i, wire) in definition.internal.iter().enumerate() {
        match (port(&wire.from), port(&wire.to)) {
            (Ok((a, ap, true)), Ok((b, bp, false))) => internal.push(((a, ap), (b, bp))),
            (Ok(_), Ok(_)) => fail(
                &format!("internal[{i}]"),
                "must run from an output to an input".into(),
            ),
            (Err(e), _) => fail(&format!("internal[{i}].from"), e),
            (_, Err(e)) => fail(&format!("internal[{i}].to"), e),
        }
    }

    // Jacks: exactly the panel's ports, in the panel's direction.
    let mut jacks = BTreeMap::new();
    for (name, reference) in &definition.jacks {
        let field = format!("jacks.{name}");
        let Some(panel_port) = panel.ports.iter().find(|p| &p.port == name) else {
            fail(&field, format!("the panel has no `{name}` port"));
            continue;
        };
        match port(reference) {
            Ok((part, port, output)) => {
                let panel_output = panel_port.direction == "out";
                if panel_output != output {
                    fail(
                        &field,
                        format!(
                            "the panel's `{name}` is an {}, but `{reference}` is an {}",
                            if panel_output { "output" } else { "input" },
                            if output { "output" } else { "input" }
                        ),
                    );
                } else {
                    jacks.insert(name.clone(), JackBinding { part, port, output });
                }
            }
            Err(e) => fail(&field, e),
        }
    }
    for panel_port in &panel.ports {
        if !definition.jacks.contains_key(&panel_port.port) {
            fail(
                "jacks",
                format!("the panel's `{}` port is not bound", panel_port.port),
            );
        }
    }

    // Knobs: exactly the panel's params, defaults inside both ranges.
    let mut knobs = BTreeMap::new();
    let controls = panel.controls();
    for (name, knob) in &definition.knobs {
        let field = format!("knobs.{name}");
        let Some(panel_param) = controls.iter().find(|p| &p.param == name) else {
            fail(&field, format!("the panel has no `{name}` knob"));
            continue;
        };
        let Some((part, param)) = split(&knob.param) else {
            fail(
                &format!("{field}.param"),
                format!("`{}` is not `part.param`", knob.param),
            );
            continue;
        };
        let Some(kind) = kinds.get(part) else {
            fail(&format!("{field}.param"), format!("no part `{part}`"));
            continue;
        };
        let Some(spec) = kind.param(param) else {
            fail(
                &format!("{field}.param"),
                format!("{} has no parameter `{param}`", kind.name()),
            );
            continue;
        };
        if knob
            .step
            .is_some_and(|step| !(step > 0.) || step > panel_param.max - panel_param.min)
        {
            fail(
                &format!("{field}.step"),
                "a step must be positive and fit inside the knob's range".into(),
            );
            continue;
        }
        if knob.quantize == Some(Quantize::Semitone) && !(panel_param.min > 0.) {
            fail(
                &format!("{field}.quantize"),
                "a semitone knob needs a range above 0 Hz".into(),
            );
            continue;
        }
        let d = knob.default;
        if !(panel_param.min..=panel_param.max).contains(&d) || !(spec.min..=spec.max).contains(&d)
        {
            fail(
                &format!("{field}.default"),
                format!(
                    "{d} is outside the knob's range {}..{} or the engine's {}..{}",
                    panel_param.min, panel_param.max, spec.min, spec.max
                ),
            );
            continue;
        }
        knobs.insert(
            name.clone(),
            Knob {
                part: part.to_owned(),
                param: spec.name,
                default: d,
                min: panel_param.min.max(spec.min),
                max: panel_param.max.min(spec.max),
                step: knob.step,
                quantize: knob.quantize,
            },
        );
    }
    for panel_param in &controls {
        if !definition.knobs.contains_key(&panel_param.param) {
            fail(
                "knobs",
                format!("the panel's `{}` knob is not bound", panel_param.param),
            );
        }
    }

    (!failed).then(|| Module {
        id: definition.id.clone(),
        parts,
        internal,
        jacks,
        knobs,
    })
}

// ------------------------------------------------------------------ errors

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RackError {
    pub file: String,
    pub module: String,
    pub field: String,
    pub message: String,
}

impl fmt::Display for RackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.module)?;
        if !self.field.is_empty() {
            write!(f, ".{}", self.field)?;
        }
        write!(f, ": {}", self.message)
    }
}

/// Every error found while loading, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RackErrors(pub Vec<RackError>);

impl fmt::Display for RackErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.0 {
            writeln!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RackErrors {}

#[cfg(test)]
mod tests;
