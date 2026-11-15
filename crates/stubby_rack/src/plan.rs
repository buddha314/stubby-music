//! The engine patch a rack plans: exactly what the rack shows, nothing more.

use crate::Modules;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};

/// A panel jack on a rack: the place holding the module, and the module's jack.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct Jack {
    pub place: String,
    pub port: String,
}

/// What a rack holds, as the planner needs it.
pub struct RackPatch<'a> {
    /// Place -> module ID, for every place that holds a module (fixed or mounted).
    pub modules: &'a HashMap<String, String>,
    /// Patched cables, output jack to input jack.
    pub cables: &'a [(Jack, Jack)],
    /// The output normalled to the machine's outlet, if any.
    pub normalled_output: Option<&'a Jack>,
    /// Engine modules the caller already owns for a (place, part), which the planner
    /// uses instead of adding one (a machine's built-in clock, for example).
    pub provided: &'a HashMap<(String, String), ModuleId>,
}

/// A rack's engine patch: the modules it adds, the commands that build it, and the
/// engine module behind each (place, part), provided ones included.
#[derive(Debug, Default)]
pub struct Plan {
    pub owned: Vec<ModuleId>,
    pub commands: Vec<Command>,
    pub parts: HashMap<(String, String), ModuleId>,
    pub voice: Option<PortRef>,
}

impl Modules {
    /// Plan a rack's engine patch. `knob(place, knob)` gives a knob's value when the
    /// caller has one (turned by the player, or a starting value), clamped to the knob's
    /// travel; otherwise the module's default is used. `fresh` hands out module IDs. Places are planned in
    /// order, so the same rack always gets the same IDs.
    pub fn plan(
        &self,
        rack: &RackPatch,
        knob: impl Fn(&str, &str) -> Option<f32>,
        mut fresh: impl FnMut() -> Option<ModuleId>,
    ) -> Plan {
        let mut plan = Plan::default();
        let places: BTreeMap<&String, &String> = rack.modules.iter().collect();
        for (place, id) in &places {
            let Some(module) = self.get(id) else { continue };
            for part in &module.parts {
                let key = ((*place).clone(), part.part.clone());
                if let Some(&provided) = rack.provided.get(&key) {
                    plan.parts.insert(key, provided);
                    continue;
                }
                let value = |param: &str| {
                    let (name, k) = module
                        .knobs
                        .iter()
                        .find(|(_, k)| k.part == part.part && k.param == param)?;
                    Some(knob(place, name).map_or(k.default, |v| v.clamp(k.min, k.max)))
                };
                let (Some(kind), Some(engine)) =
                    (ModuleKind::from_params(part.kind, value), fresh())
                else {
                    continue;
                };
                plan.owned.push(engine);
                plan.parts.insert(key, engine);
                plan.commands.push(Command::AddModule { id: engine, kind });
            }
            for ((a, ap), (b, bp)) in &module.internal {
                let from = plan.parts.get(&((*place).clone(), a.clone()));
                let to = plan.parts.get(&((*place).clone(), b.clone()));
                if let (Some(&from), Some(&to)) = (from, to) {
                    plan.commands.push(Command::Connect {
                        from: PortRef::new(from, ap),
                        to: PortRef::new(to, bp),
                    });
                }
            }
        }
        let port = |jack: &Jack| {
            let binding = self
                .get(rack.modules.get(&jack.place)?)?
                .jacks
                .get(&jack.port)?;
            let engine = plan
                .parts
                .get(&(jack.place.clone(), binding.part.clone()))?;
            Some(PortRef::new(*engine, binding.port))
        };
        let mut connects = Vec::new();
        for (from, to) in rack.cables {
            if let (Some(from), Some(to)) = (port(from), port(to)) {
                connects.push(Command::Connect { from, to });
            }
        }
        let voice = rack.normalled_output.and_then(port);
        plan.commands.extend(connects);
        if let Some(out) = voice {
            plan.commands.push(Command::Connect {
                from: out,
                to: PortRef::output(),
            });
        }
        plan.voice = voice;
        plan
    }
}
