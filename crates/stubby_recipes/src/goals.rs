//! Goal definitions: what the hub tells the player to do next (add-inventory-and-goals-hud).
//!
//! A goal is met either by items delivered to it (`requires`) or by a condition in the
//! world (`condition`): the first short-term goals, the chores, are conditions. The
//! conditions are a closed list owned by this schema; adding one is a rule PR. Progress
//! (paid amounts, completion) is not here yet: these are the definitions and their checks.
use crate::rules::{RulesError, valid_id};
use crate::{CatalogDefinition, Stack};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GOALS: &str = "goals.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    Long,
    Short,
}

/// Game states a goal can wait for. The game reports each when it starts to hold.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    /// Every `configure.cables` patch of a placed machine is made, so its voice sounds.
    MachineConfigured { machine: String },
    /// A stream delivers water to a placed tree of this mine (needs add-digging).
    TreeWatered { mine: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoalDefinition {
    pub id: String,
    pub name: String,
    /// One line the hub shows under the name.
    pub description: String,
    pub horizon: Horizon,
    /// Items delivered to meet it. Exactly one of `requires` and `condition`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<Stack>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    /// Goals that must be complete before this one opens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
    /// Items discovered in the recipe book when it completes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unlocks: Vec<String>,
}

/// Checks over the goals against the rest of the rules; every problem is collected.
pub fn validate_goals(
    definition: &CatalogDefinition,
    rule_ids: &BTreeSet<String>,
    goals: &[GoalDefinition],
) -> Vec<RulesError> {
    let err = |id: &str, field: &str, message: String| RulesError {
        file: GOALS.into(),
        id: id.into(),
        field: field.into(),
        message,
    };
    let mut errors = Vec::new();
    let items: BTreeSet<&str> = definition.items.iter().map(|i| i.id.as_str()).collect();
    let mines: BTreeSet<&str> = definition.mines.iter().map(|m| m.id.as_str()).collect();
    let ids: BTreeSet<&str> = goals.iter().map(|g| g.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    for goal in goals {
        if !valid_id(&goal.id) {
            errors.push(err(
                &goal.id,
                "id",
                "IDs are snake_case: a lowercase letter, then a-z, 0-9 or _".into(),
            ));
        }
        if !seen.insert(goal.id.as_str()) || rule_ids.contains(&goal.id) {
            errors.push(err(
                &goal.id,
                "id",
                "ID already used; IDs are unique across all rules files".into(),
            ));
        }
        match (goal.requires.is_empty(), &goal.condition) {
            (true, None) => errors.push(err(
                &goal.id,
                "requires",
                "needs `requires` or `condition`".into(),
            )),
            (false, Some(_)) => errors.push(err(
                &goal.id,
                "condition",
                "has both `requires` and `condition`; use one".into(),
            )),
            _ => {}
        }
        for stack in &goal.requires {
            if stack.quantity == 0 || !items.contains(stack.item.as_str()) {
                errors.push(err(
                    &goal.id,
                    "requires",
                    format!(
                        "{} x {} is not a known item and positive quantity",
                        stack.quantity, stack.item
                    ),
                ));
            }
        }
        match &goal.condition {
            Some(Condition::MachineConfigured { machine }) if !mines.contains(machine.as_str()) => {
                errors.push(err(
                    &goal.id,
                    "condition",
                    format!("unknown machine {machine}"),
                ));
            }
            Some(Condition::TreeWatered { mine }) if !mines.contains(mine.as_str()) => {
                errors.push(err(&goal.id, "condition", format!("unknown mine {mine}")));
            }
            _ => {}
        }
        for before in &goal.after {
            if !ids.contains(before.as_str()) {
                errors.push(err(&goal.id, "after", format!("unknown goal {before}")));
            }
        }
        for item in &goal.unlocks {
            if !items.contains(item.as_str()) {
                errors.push(err(&goal.id, "unlocks", format!("unknown item {item}")));
            }
        }
    }
    // Prerequisite cycles: repeatedly open goals whose `after` are all open.
    let after: BTreeMap<&str, Vec<&str>> = goals
        .iter()
        .map(|g| (g.id.as_str(), g.after.iter().map(String::as_str).collect()))
        .collect();
    let mut open: BTreeSet<&str> = BTreeSet::new();
    loop {
        let before = open.len();
        for (id, needs) in &after {
            if needs.iter().all(|n| open.contains(n) || !ids.contains(n)) {
                open.insert(id);
            }
        }
        if open.len() == before {
            break;
        }
    }
    for id in ids.difference(&open) {
        errors.push(err(
            id,
            "after",
            "prerequisite cycle: this goal can never open".into(),
        ));
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prerequisite_cycle_is_rejected() {
        let rules = crate::Rules::load(&crate::rules::shipped_files()).unwrap();
        let mut goals = rules.goals.clone();
        goals[0].after = vec![goals[1].id.clone()];
        let errors = validate_goals(&rules.definition, &BTreeSet::new(), &goals);
        assert!(
            errors.iter().any(|e| e.message.contains("cycle")),
            "{errors:?}"
        );
    }
}
