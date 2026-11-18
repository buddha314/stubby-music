//! Machine settings: what each kind of machine needs and how far it reaches, tuned in
//! play (add-machine-rules). One row per machine kind, naming its placed asset ID
//! (`campfire`, `mudpit_burbler`). The machine's art (model, rack places, cables) stays
//! in its profile; these rows are the numbers the game's rules read, and dev builds reload
//! them while the game runs.
//!
//! This core checks a row's shape only. Whether a target names a real rack place and knob
//! is checked where the profiles and modules are known (`bevy_stubby_rack`'s tests).
use crate::rules::{RulesError, valid_id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MACHINES: &str = "machines.json";

/// A knob a machine's work needs set: in a rack place, at a note (`C2`, for a semitone
/// knob) or at a value (`1`, a step gate on). Give one of `note` or `value`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MachineTarget {
    /// The rack place (`seq`, `vco_low`), as the machine's profile names it.
    pub place: String,
    /// The knob, as the module's data names it (`gate_2`, `freq`).
    pub knob: String,
    /// A note with its octave: `C4`, `F#2`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MachineDefinition {
    pub id: String,
    /// The machine kind these settings are for: its placed asset ID.
    pub machine: String,
    /// The knobs its work needs, all at once (the campfire lights only with these met).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<MachineTarget>,
    /// How far, in metres, a machine with a borrowed clock reaches for a running one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock_reach_m: Option<f32>,
}

/// The farthest a machine may reach for a clock.
pub const MAX_CLOCK_REACH_M: f32 = 50.;

const NOTES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Whether `name` is a note with its octave, as the rack writes them (`C4`, `F#2`, `B-1`).
pub fn is_note(name: &str) -> bool {
    let Some(split) = name.find(|c: char| c == '-' || c.is_ascii_digit()) else {
        return false;
    };
    let (letter, octave) = name.split_at(split);
    NOTES.contains(&letter) && octave.parse::<i32>().is_ok()
}

pub fn validate_machines(rows: &[MachineDefinition]) -> Vec<RulesError> {
    let mut errors = Vec::new();
    let mut machines = BTreeSet::new();
    let error = |id: &str, field: &str, message: String| RulesError {
        file: MACHINES.into(),
        id: id.into(),
        field: field.into(),
        message,
    };
    for row in rows {
        if !valid_id(&row.id) {
            errors.push(error(&row.id, "id", "IDs are snake_case [a-z0-9_]".into()));
        }
        if row.machine.is_empty() || !machines.insert(row.machine.clone()) {
            errors.push(error(
                &row.id,
                "machine",
                format!("`{}` is empty or already has settings", row.machine),
            ));
        }
        if let Some(reach) = row.clock_reach_m
            && !(reach > 0. && reach <= MAX_CLOCK_REACH_M)
        {
            errors.push(error(
                &row.id,
                "clock_reach_m",
                format!("{reach} m is outside 0..{MAX_CLOCK_REACH_M}"),
            ));
        }
        let mut seen = BTreeSet::new();
        for (k, target) in row.targets.iter().enumerate() {
            let field = format!("targets/{k}");
            match (&target.note, target.value) {
                (Some(_), Some(_)) | (None, None) => errors.push(error(
                    &row.id,
                    &field,
                    "give a `note` or a `value`, not both".into(),
                )),
                (Some(note), None) if !is_note(note) => errors.push(error(
                    &row.id,
                    &format!("{field}/note"),
                    format!("`{note}` is not a note like C4 or F#2"),
                )),
                _ => {}
            }
            if !seen.insert((&target.place, &target.knob)) {
                errors.push(error(
                    &row.id,
                    &field,
                    format!("{}.{} has two targets", target.place, target.knob),
                ));
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_target_is_reported_with_its_field() {
        let target = |note: Option<&str>, value: Option<f32>| MachineTarget {
            place: "seq".into(),
            knob: "gate_2".into(),
            note: note.map(Into::into),
            value,
        };
        let row = MachineDefinition {
            id: "campfire_tuning".into(),
            machine: "campfire".into(),
            targets: vec![
                target(Some("C4"), Some(1.)),
                target(Some("H2"), None),
                target(None, Some(1.)),
            ],
            clock_reach_m: Some(500.),
        };
        let errors = validate_machines(&[row]);
        let fields: Vec<_> = errors.iter().map(|e| e.field.as_str()).collect();
        assert!(fields.contains(&"targets/0"), "{errors:?}");
        assert!(fields.contains(&"targets/1/note"), "{errors:?}");
        assert!(fields.contains(&"clock_reach_m"), "{errors:?}");
        assert!(is_note("F#2") && is_note("B-1") && !is_note("C"));
    }
}
