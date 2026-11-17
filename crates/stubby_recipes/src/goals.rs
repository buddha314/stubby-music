//! Goal definitions: what the hub tells the player to do next (add-inventory-and-goals-hud).
//!
//! A goal is met either by items delivered to it (`requires`) or by a condition in the
//! world (`condition`): the first short-term goals, the chores, are conditions. The
//! conditions are a closed list owned by this schema; adding one is a rule PR. The
//! Bevy-free reducer below owns paid amounts, completion, active goals and recipe pins.
use crate::rules::{RulesError, valid_id};
use crate::{CatalogDefinition, Inventory, RecipeBook, Rules, Stack};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GOALS: &str = "goals.json";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    Long,
    Short,
}

/// Game states a goal can wait for. The game reports each when it starts to hold.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    /// Every `configure.cables` patch of a placed machine is made, so its voice sounds.
    MachineConfigured {
        #[schemars(extend("x-stubby-ref" = "mines"))]
        machine: String,
    },
    /// A stream delivers water to a placed tree of this mine (needs add-digging).
    TreeWatered {
        #[schemars(extend("x-stubby-ref" = "mines"))]
        mine: String,
    },
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
    #[schemars(extend("x-stubby-ref" = "goals"))]
    pub after: Vec<String>,
    /// Items discovered in the recipe book when it completes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(extend("x-stubby-ref" = "items"))]
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
        let mut totals = BTreeMap::<&str, u32>::new();
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
            let total = totals.entry(stack.item.as_str()).or_default();
            if let Some(sum) = total.checked_add(stack.quantity) {
                *total = sum;
            } else {
                errors.push(err(
                    &goal.id,
                    "requires",
                    format!("quantity overflow for {}", stack.item),
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalProgress {
    completed: BTreeSet<String>,
    paid: BTreeMap<String, BTreeMap<String, u32>>,
    active_long: Option<String>,
    active_short: Option<String>,
    met_conditions: BTreeSet<Condition>,
    pins: Pins,
}

impl GoalProgress {
    pub fn is_complete(&self, goal: &str) -> bool {
        self.completed.contains(goal)
    }

    pub fn completed(&self) -> impl Iterator<Item = &str> {
        self.completed.iter().map(String::as_str)
    }

    pub fn paid(&self, goal: &str, item: &str) -> u32 {
        self.paid
            .get(goal)
            .and_then(|items| items.get(item))
            .copied()
            .unwrap_or(0)
    }

    pub fn pins(&self) -> &Pins {
        &self.pins
    }

    pub fn active_goal<'a>(
        &self,
        rules: &'a Rules,
        horizon: Horizon,
    ) -> Option<&'a GoalDefinition> {
        let chosen = match horizon {
            Horizon::Long => self.active_long.as_deref(),
            Horizon::Short => self.active_short.as_deref(),
        };
        let open = |goal: &&GoalDefinition| {
            goal.horizon == horizon
                && !self.completed.contains(&goal.id)
                && goal.after.iter().all(|id| self.completed.contains(id))
        };
        chosen
            .and_then(|id| rules.goals.iter().find(|goal| goal.id == id).filter(open))
            .or_else(|| rules.goals.iter().find(open))
    }

    pub fn item_progress(
        &self,
        goal: &GoalDefinition,
        item: &str,
        inventory: &Inventory,
    ) -> Option<ItemProgress> {
        let required = goal
            .requires
            .iter()
            .filter(|stack| stack.item == item)
            .try_fold(0u32, |sum, stack| sum.checked_add(stack.quantity))?;
        if required == 0 {
            return None;
        }
        let paid = self.paid(&goal.id, item);
        let held = inventory.quantity(item);
        Some(ItemProgress {
            paid,
            held,
            required,
            state: progress_state(paid, held, required),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pins(BTreeMap<String, u32>);

impl Pins {
    pub fn count(&self, recipe: &str) -> u32 {
        self.0.get(recipe).copied().unwrap_or(0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, u32)> {
        self.0
            .iter()
            .map(|(recipe, count)| (recipe.as_str(), *count))
    }

    pub fn totals(&self, rules: &Rules) -> Result<Vec<Stack>, GoalError> {
        let recipes: BTreeMap<_, _> = rules
            .definition
            .recipes
            .iter()
            .map(|recipe| (recipe.id.as_str(), recipe))
            .collect();
        let mut totals = BTreeMap::<String, u32>::new();
        for (id, count) in &self.0 {
            let recipe = recipes
                .get(id.as_str())
                .ok_or_else(|| GoalError::UnknownRecipe(id.clone()))?;
            for stack in &recipe.inputs {
                let quantity = stack
                    .quantity
                    .checked_mul(*count)
                    .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
                let total = totals.entry(stack.item.clone()).or_default();
                *total = total
                    .checked_add(quantity)
                    .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
            }
        }
        Ok(totals
            .into_iter()
            .map(|(item, quantity)| Stack { item, quantity })
            .collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressState {
    Empty,
    Low,
    High,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemProgress {
    pub paid: u32,
    pub held: u32,
    pub required: u32,
    pub state: ProgressState,
}

fn progress_state(paid: u32, held: u32, required: u32) -> ProgressState {
    let total = u64::from(paid) + u64::from(held);
    if total == 0 {
        ProgressState::Empty
    } else if total >= u64::from(required) {
        ProgressState::Full
    } else if total * 2 >= u64::from(required) {
        ProgressState::High
    } else {
        ProgressState::Low
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalEvent {
    Deliver(String),
    ConditionMet(Condition),
    Crafted(String),
    Choose {
        horizon: Horizon,
        goal: Option<String>,
    },
    Pin {
        recipe: String,
        count: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalOutcome {
    Paid {
        goal: String,
        item: String,
        quantity: u32,
    },
    NeedItems {
        goal: String,
        items: Vec<Stack>,
    },
    GoalCompleted(String),
    Opened(String),
    Discovered(String),
    PinDone(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GoalError {
    #[error("unknown goal: {0}")]
    UnknownGoal(String),
    #[error("goal is not open: {0}")]
    GoalNotOpen(String),
    #[error("goal is met by a condition and does not accept item delivery: {0}")]
    ConditionGoal(String),
    #[error("unknown recipe: {0}")]
    UnknownRecipe(String),
    #[error("quantity overflow for {0}")]
    QuantityOverflow(String),
    #[error(transparent)]
    Inventory(#[from] crate::RecipeError),
}

/// Apply one goal event atomically. Rules remain the source of definitions; the
/// inventory and recipe book are explicit state at the Bevy-free boundary.
pub fn apply(
    rules: &Rules,
    progress: &mut GoalProgress,
    inventory: &mut Inventory,
    recipe_book: &mut RecipeBook,
    event: GoalEvent,
) -> Result<Vec<GoalOutcome>, GoalError> {
    let mut staged_progress = progress.clone();
    let mut staged_inventory = inventory.clone();
    let mut staged_recipe_book = recipe_book.clone();
    let outcomes = apply_staged(
        rules,
        &mut staged_progress,
        &mut staged_inventory,
        &mut staged_recipe_book,
        event,
    )?;
    *progress = staged_progress;
    *inventory = staged_inventory;
    *recipe_book = staged_recipe_book;
    Ok(outcomes)
}

fn apply_staged(
    rules: &Rules,
    progress: &mut GoalProgress,
    inventory: &mut Inventory,
    recipe_book: &mut RecipeBook,
    event: GoalEvent,
) -> Result<Vec<GoalOutcome>, GoalError> {
    let open_before: BTreeSet<_> = rules
        .goals
        .iter()
        .filter(|goal| {
            !progress.completed.contains(&goal.id)
                && goal.after.iter().all(|id| progress.completed.contains(id))
        })
        .map(|goal| goal.id.clone())
        .collect();
    let mut outcomes = Vec::new();
    match event {
        GoalEvent::Deliver(id) => {
            let goal = rules
                .goals
                .iter()
                .find(|goal| goal.id == id)
                .ok_or_else(|| GoalError::UnknownGoal(id.clone()))?;
            if goal.condition.is_some() {
                return Err(GoalError::ConditionGoal(id));
            }
            if progress.completed.contains(&id)
                || !goal
                    .after
                    .iter()
                    .all(|need| progress.completed.contains(need))
            {
                return Ok(outcomes);
            }
            let mut required = BTreeMap::<String, u32>::new();
            for stack in &goal.requires {
                let total = required.entry(stack.item.clone()).or_default();
                *total = total
                    .checked_add(stack.quantity)
                    .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
            }
            let delivered: Vec<_> = required
                .iter()
                .filter_map(|(item, quantity)| {
                    let paid = progress.paid(&id, item);
                    let remaining = quantity.saturating_sub(paid);
                    let available = inventory.quantity(item);
                    let accepted = remaining.min(available);
                    (accepted > 0).then(|| Stack {
                        item: item.clone(),
                        quantity: accepted,
                    })
                })
                .collect();
            if delivered.is_empty() {
                let items = required
                    .iter()
                    .filter_map(|(item, quantity)| {
                        let remaining = quantity.saturating_sub(progress.paid(&id, item));
                        (remaining > 0).then(|| Stack {
                            item: item.clone(),
                            quantity: remaining,
                        })
                    })
                    .collect();
                outcomes.push(GoalOutcome::NeedItems { goal: id, items });
            } else {
                inventory.take_stacks(&delivered)?;
                let paid = progress.paid.entry(id.clone()).or_default();
                for stack in delivered {
                    let amount = paid.entry(stack.item.clone()).or_default();
                    *amount = amount
                        .checked_add(stack.quantity)
                        .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
                    outcomes.push(GoalOutcome::Paid {
                        goal: id.clone(),
                        item: stack.item,
                        quantity: stack.quantity,
                    });
                }
                if required
                    .iter()
                    .all(|(item, quantity)| progress.paid(&id, item) >= *quantity)
                {
                    complete_goal(goal, progress, recipe_book, &mut outcomes);
                }
            }
        }
        GoalEvent::ConditionMet(condition) => {
            progress.met_conditions.insert(condition.clone());
            for goal in &rules.goals {
                if !progress.completed.contains(&goal.id)
                    && goal
                        .after
                        .iter()
                        .all(|need| progress.completed.contains(need))
                    && goal.condition.as_ref() == Some(&condition)
                {
                    complete_goal(goal, progress, recipe_book, &mut outcomes);
                }
            }
        }
        GoalEvent::Crafted(recipe) => {
            if let Some(count) = progress.pins.0.get_mut(&recipe) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    progress.pins.0.remove(&recipe);
                    outcomes.push(GoalOutcome::PinDone(recipe));
                }
            }
        }
        GoalEvent::Choose { horizon, goal } => {
            if let Some(id) = &goal {
                let candidate = rules
                    .goals
                    .iter()
                    .find(|candidate| candidate.id == *id)
                    .ok_or_else(|| GoalError::UnknownGoal(id.clone()))?;
                if candidate.horizon != horizon
                    || progress.completed.contains(id)
                    || !candidate
                        .after
                        .iter()
                        .all(|need| progress.completed.contains(need))
                {
                    return Err(GoalError::GoalNotOpen(id.clone()));
                }
            }
            match horizon {
                Horizon::Long => progress.active_long = goal,
                Horizon::Short => progress.active_short = goal,
            }
        }
        GoalEvent::Pin { recipe, count } => {
            if !rules
                .definition
                .recipes
                .iter()
                .any(|candidate| candidate.id == recipe)
            {
                return Err(GoalError::UnknownRecipe(recipe));
            }
            if count == 0 {
                progress.pins.0.remove(&recipe);
            } else {
                progress.pins.0.insert(recipe, count);
            }
        }
    }
    for goal in &rules.goals {
        let is_open = !progress.completed.contains(&goal.id)
            && goal.after.iter().all(|id| progress.completed.contains(id));
        if is_open && !open_before.contains(&goal.id) {
            outcomes.push(GoalOutcome::Opened(goal.id.clone()));
            if let Some(condition) = &goal.condition
                && progress.met_conditions.contains(condition)
            {
                complete_goal(goal, progress, recipe_book, &mut outcomes);
            }
        }
    }
    Ok(outcomes)
}

fn complete_goal(
    goal: &GoalDefinition,
    progress: &mut GoalProgress,
    recipe_book: &mut RecipeBook,
    outcomes: &mut Vec<GoalOutcome>,
) {
    if progress.completed.insert(goal.id.clone()) {
        outcomes.push(GoalOutcome::GoalCompleted(goal.id.clone()));
        for item in &goal.unlocks {
            recipe_book.discover(item);
            outcomes.push(GoalOutcome::Discovered(item.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RecipeDefinition;

    fn stack(item: &str, quantity: u32) -> Stack {
        Stack {
            item: item.into(),
            quantity,
        }
    }

    fn goal(id: &str, requires: Vec<Stack>, after: &[&str], unlocks: &[&str]) -> GoalDefinition {
        GoalDefinition {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            horizon: Horizon::Short,
            requires,
            condition: None,
            after: after.iter().map(|id| id.to_string()).collect(),
            unlocks: unlocks.iter().map(|id| id.to_string()).collect(),
        }
    }

    /// Shipped rules plus an item goal (4 square waves, 6 reeds) that opens "next",
    /// and a recipe needing 6 reeds for pins.
    fn rules() -> Rules {
        let mut rules = Rules::load(&crate::rules::shipped_files()).unwrap();
        rules.goals.push(goal(
            "deliver",
            vec![stack("square_wave", 4), stack("reed", 6)],
            &[],
            &["clock"],
        ));
        rules
            .goals
            .push(goal("next", vec![stack("reed", 1)], &["deliver"], &[]));
        rules.definition.recipes.push(RecipeDefinition {
            id: "stream_kit".into(),
            name: "Stream kit".into(),
            inputs: vec![stack("reed", 6)],
            outputs: vec![stack("patch_cable", 1)],
        });
        rules
    }

    fn held(stacks: &[(&str, u32)]) -> Inventory {
        let mut inventory = Inventory::default();
        for (item, quantity) in stacks {
            inventory.add(item, *quantity).unwrap();
        }
        inventory
    }

    struct World {
        rules: Rules,
        progress: GoalProgress,
        inventory: Inventory,
        book: RecipeBook,
    }

    impl World {
        fn new(stacks: &[(&str, u32)]) -> Self {
            Self {
                rules: rules(),
                progress: GoalProgress::default(),
                inventory: held(stacks),
                book: RecipeBook::default(),
            }
        }
        fn apply(&mut self, event: GoalEvent) -> Vec<GoalOutcome> {
            apply(
                &self.rules,
                &mut self.progress,
                &mut self.inventory,
                &mut self.book,
                event,
            )
            .unwrap()
        }
        fn deliver(&mut self) -> Vec<GoalOutcome> {
            self.apply(GoalEvent::Deliver("deliver".into()))
        }
        fn goal(&self, id: &str) -> &GoalDefinition {
            self.rules.goals.iter().find(|g| g.id == id).unwrap()
        }
    }

    fn paid(goal: &str, item: &str, quantity: u32) -> GoalOutcome {
        GoalOutcome::Paid {
            goal: goal.into(),
            item: item.into(),
            quantity,
        }
    }

    #[test]
    fn thresholds_and_progress() {
        for (held_count, state) in [
            (0, ProgressState::Empty),
            (1, ProgressState::Low),
            (2, ProgressState::High),
            (4, ProgressState::Full),
        ] {
            let world = World::new(&[("square_wave", held_count)]);
            let progress = world
                .progress
                .item_progress(world.goal("deliver"), "square_wave", &world.inventory)
                .unwrap();
            assert_eq!((progress.held, progress.required), (held_count, 4));
            assert_eq!(progress.state, state, "held {held_count}");
        }
    }

    #[test]
    fn paid_and_held_count_together_but_holding_does_not_complete() {
        let mut world = World::new(&[("square_wave", 2)]);
        world.deliver();
        world.inventory.add("square_wave", 1).unwrap();
        let progress = world
            .progress
            .item_progress(world.goal("deliver"), "square_wave", &world.inventory)
            .unwrap();
        assert_eq!((progress.paid, progress.held), (2, 1));
        assert_eq!(progress.state, ProgressState::High);

        let mut world = World::new(&[("square_wave", 4), ("reed", 6)]);
        for item in ["square_wave", "reed"] {
            let progress =
                world
                    .progress
                    .item_progress(world.goal("deliver"), item, &world.inventory);
            assert_eq!(progress.unwrap().state, ProgressState::Full);
        }
        assert!(!world.progress.is_complete("deliver"));
        world.deliver();
        assert!(world.progress.is_complete("deliver"));
    }

    #[test]
    fn goals_not_yet_open_are_not_active() {
        let mut world = World::new(&[]);
        let active = world
            .progress
            .active_goal(&world.rules, Horizon::Short)
            .unwrap();
        assert_ne!(active.id, "next");
        let chosen = apply(
            &world.rules,
            &mut world.progress,
            &mut world.inventory,
            &mut world.book,
            GoalEvent::Choose {
                horizon: Horizon::Short,
                goal: Some("next".into()),
            },
        );
        assert_eq!(chosen, Err(GoalError::GoalNotOpen("next".into())));
    }

    #[test]
    fn partial_then_completing_delivery() {
        let mut world = World::new(&[("square_wave", 1), ("reed", 10)]);
        assert_eq!(
            world.deliver(),
            [
                paid("deliver", "reed", 6),
                paid("deliver", "square_wave", 1)
            ]
        );
        assert_eq!(world.progress.paid("deliver", "square_wave"), 1);
        assert_eq!(world.progress.paid("deliver", "reed"), 6);
        assert_eq!(world.inventory.quantity("reed"), 4);
        assert_eq!(world.inventory.quantity("square_wave"), 0);
        assert!(!world.progress.is_complete("deliver"));

        world.inventory.add("square_wave", 5).unwrap();
        assert_eq!(
            world.deliver(),
            [
                paid("deliver", "square_wave", 3),
                GoalOutcome::GoalCompleted("deliver".into()),
                GoalOutcome::Discovered("clock".into()),
                GoalOutcome::Opened("next".into()),
            ]
        );
        assert_eq!(world.inventory.quantity("square_wave"), 2);
        assert!(world.book.knows("clock"));

        // Repeated event: completion reported once, nothing more consumed.
        assert_eq!(world.deliver(), []);
        assert_eq!(world.inventory.quantity("square_wave"), 2);
        assert_eq!(world.inventory.quantity("reed"), 4);
    }

    #[test]
    fn nothing_to_give_changes_nothing() {
        let mut world = World::new(&[("patch_cable", 3)]);
        let before = world.inventory.clone();
        assert_eq!(
            world.deliver(),
            [GoalOutcome::NeedItems {
                goal: "deliver".into(),
                items: vec![stack("reed", 6), stack("square_wave", 4)],
            }]
        );
        assert_eq!(world.inventory, before);
        assert_eq!(world.progress, GoalProgress::default());
    }

    #[test]
    fn switching_goals_keeps_paid_amounts() {
        let mut world = World::new(&[("square_wave", 2)]);
        world.deliver();
        for goal in ["build_a_heartbeat", "deliver"] {
            world.apply(GoalEvent::Choose {
                horizon: Horizon::Short,
                goal: Some(goal.into()),
            });
            assert_eq!(
                world
                    .progress
                    .active_goal(&world.rules, Horizon::Short)
                    .unwrap()
                    .id,
                goal
            );
        }
        assert_eq!(world.progress.paid("deliver", "square_wave"), 2);
    }

    #[test]
    fn last_goal_leaves_nothing_active_and_play_continues() {
        let mut world = World::new(&[("square_wave", 4), ("reed", 7)]);
        world.deliver();
        world.apply(GoalEvent::Deliver("next".into()));
        world.apply(GoalEvent::ConditionMet(Condition::MachineConfigured {
            machine: "mudpit_burbler".into(),
        }));
        world.apply(GoalEvent::ConditionMet(Condition::TreeWatered {
            mine: "scrub_oak".into(),
        }));
        assert!(
            world
                .rules
                .goals
                .iter()
                .all(|g| world.progress.is_complete(&g.id))
        );
        assert!(
            world
                .progress
                .active_goal(&world.rules, Horizon::Short)
                .is_none()
        );
        assert!(
            world
                .progress
                .active_goal(&world.rules, Horizon::Long)
                .is_none()
        );
        // Further events still apply.
        world.apply(GoalEvent::Pin {
            recipe: "stream_kit".into(),
            count: 1,
        });
    }

    #[test]
    fn pin_totals_and_crafting_lowers_the_pin() {
        let mut world = World::new(&[("reed", 10)]);
        world.apply(GoalEvent::Pin {
            recipe: "stream_kit".into(),
            count: 3,
        });
        assert_eq!(
            world.progress.pins().totals(&world.rules).unwrap(),
            [stack("reed", 18)]
        );
        world.apply(GoalEvent::Pin {
            recipe: "stream_kit".into(),
            count: 1,
        });
        assert_eq!(
            world.apply(GoalEvent::Crafted("stream_kit".into())),
            [GoalOutcome::PinDone("stream_kit".into())]
        );
        assert_eq!(world.progress.pins().count("stream_kit"), 0);
    }

    #[test]
    fn build_a_heartbeat_opens_water_the_tree_and_stays_complete() {
        let mut world = World::new(&[]);
        let burbler = Condition::MachineConfigured {
            machine: "mudpit_burbler".into(),
        };
        assert_eq!(
            world.apply(GoalEvent::ConditionMet(burbler.clone())),
            [
                GoalOutcome::GoalCompleted("build_a_heartbeat".into()),
                GoalOutcome::Opened("water_the_tree".into()),
            ]
        );
        // Unpatching sends nothing that reopens it; repeating the condition is harmless.
        assert_eq!(world.apply(GoalEvent::ConditionMet(burbler)), []);
        assert!(world.progress.is_complete("build_a_heartbeat"));
    }

    #[test]
    fn a_condition_met_before_its_goal_opens_completes_it_on_opening() {
        let mut world = World::new(&[]);
        world.apply(GoalEvent::ConditionMet(Condition::TreeWatered {
            mine: "scrub_oak".into(),
        }));
        assert!(!world.progress.is_complete("water_the_tree"));
        let outcomes = world.apply(GoalEvent::ConditionMet(Condition::MachineConfigured {
            machine: "mudpit_burbler".into(),
        }));
        assert!(outcomes.contains(&GoalOutcome::GoalCompleted("water_the_tree".into())));
    }

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
