//! Goals: what the hub tells the player to do next (add-inventory-and-goals-hud,
//! sync-major-minor-milestones).
//!
//! Goals are **major** goals made of ordered **minor** steps (Satisfactory's phases and
//! milestones; the Godot game's goal bar). A major goal has no cost of its own: it
//! completes when its last step does. A step is met either by items delivered to it
//! (`requires`) or by a condition (`condition`). Conditions are a closed list owned by
//! this schema; adding one is a rule PR.
//!
//! Conditions are state, not events: the game reports when a world condition starts and
//! stops holding, `item_held` is read from the inventory, and every open step is checked
//! again whenever anything changes. So a condition already true when its step opens
//! completes it at once, and completion latches. The Bevy-free reducer [`apply`] owns
//! paid amounts, completion, the chosen major goal and recipe pins.
use crate::rules::{RulesError, valid_id};
use crate::{CatalogDefinition, Inventory, RecipeBook, Rules, Stack};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GOALS: &str = "goals.json";
/// `goals.json`'s format. 2 replaced `horizon` with `scale` and `part_of`.
pub const GOALS_FORMAT: u32 = 2;

/// A goal's level: a major goal is a set of minor steps.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    Major,
    Minor,
}

pub use crate::growth::GrowthLevel;

/// Game states a step can wait for.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    /// Every `configure.cables` patch of a placed machine is made, so its voice sounds.
    MachineConfigured {
        #[schemars(extend("x-stubby-ref" = "mines"))]
        machine: String,
    },
    /// A flowing stream waters a placed tree of this mine.
    TreeWatered {
        #[schemars(extend("x-stubby-ref" = "mines"))]
        mine: String,
    },
    /// A placed tree of this mine is at this bloom level or above.
    TreeGrown {
        #[schemars(extend("x-stubby-ref" = "mines"))]
        mine: String,
        level: GrowthLevel,
    },
    /// A placed buildable's fire has caught (the campfire, lit on a backbeat). It latches:
    /// the fire dying later does not clear it.
    FireLit {
        #[schemars(extend("x-stubby-ref" = "buildables"))]
        buildable: String,
    },
    /// The player has built one of this buildable with the build menu (Q). It latches:
    /// dismantling it later does not clear it.
    Built {
        #[schemars(extend("x-stubby-ref" = "buildables"))]
        buildable: String,
    },
    /// The player's recipe book knows this item: collected or crafted at least once. It
    /// latches: using the item up later does not clear it (add-item-found-condition).
    ItemFound {
        #[schemars(extend("x-stubby-ref" = "items"))]
        item: String,
    },
    /// The player holds at least this many of an item.
    ItemHeld {
        #[schemars(extend("x-stubby-ref" = "items"))]
        item: String,
        quantity: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoalDefinition {
    pub id: String,
    pub name: String,
    /// One line the hub shows under the name.
    pub description: String,
    pub scale: Scale,
    /// The major goal a minor step belongs to. Steps run in file order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(extend("x-stubby-ref" = "goals"))]
    pub part_of: Option<String>,
    /// Items delivered to meet a step. A step has exactly one of `requires` and `condition`.
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
    let buildables: BTreeSet<&str> = definition
        .buildables
        .iter()
        .map(|b| b.id.as_str())
        .collect();
    let ids: BTreeSet<&str> = goals.iter().map(|g| g.id.as_str()).collect();
    let scales: BTreeMap<&str, Scale> = goals.iter().map(|g| (g.id.as_str(), g.scale)).collect();
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
        match goal.scale {
            Scale::Major => {
                if goal.part_of.is_some() {
                    errors.push(err(
                        &goal.id,
                        "part_of",
                        "a major goal is not part of another goal".into(),
                    ));
                }
                if goal.condition.is_some() {
                    errors.push(err(
                        &goal.id,
                        "condition",
                        "a major goal completes with its steps; put the condition on a step".into(),
                    ));
                }
                if !goal.requires.is_empty() {
                    errors.push(err(
                        &goal.id,
                        "requires",
                        "a major goal completes with its steps; put the cost on a step".into(),
                    ));
                }
                if !goals
                    .iter()
                    .any(|step| step.part_of.as_deref() == Some(goal.id.as_str()))
                {
                    errors.push(err(
                        &goal.id,
                        "scale",
                        "a major goal needs at least one minor step (`part_of` it)".into(),
                    ));
                }
            }
            Scale::Minor => {
                match goal.part_of.as_deref() {
                    None => errors.push(err(
                        &goal.id,
                        "part_of",
                        "a minor step needs `part_of`: the major goal it belongs to".into(),
                    )),
                    Some(major) => match scales.get(major) {
                        None => errors.push(err(
                            &goal.id,
                            "part_of",
                            format!("unknown major goal {major}"),
                        )),
                        Some(Scale::Minor) => errors.push(err(
                            &goal.id,
                            "part_of",
                            format!("{major} is a minor step, not a major goal"),
                        )),
                        Some(Scale::Major) => {}
                    },
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
            }
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
            Some(Condition::FireLit { buildable } | Condition::Built { buildable })
                if !buildables.contains(buildable.as_str()) =>
            {
                errors.push(err(
                    &goal.id,
                    "condition",
                    format!("unknown buildable {buildable}"),
                ));
            }
            Some(Condition::TreeWatered { mine } | Condition::TreeGrown { mine, .. })
                if !mines.contains(mine.as_str()) =>
            {
                errors.push(err(&goal.id, "condition", format!("unknown mine {mine}")));
            }
            Some(Condition::TreeGrown { mine, level })
                if !definition.mines.iter().any(|m| {
                    &m.id == mine
                        && m.growth
                            .as_ref()
                            .is_some_and(|g| g.levels.iter().any(|s| s.level == *level))
                }) =>
            {
                errors.push(err(
                    &goal.id,
                    "condition",
                    format!("{mine} has no growth level {level:?} (mines.json `growth`)"),
                ));
            }
            Some(Condition::ItemFound { item }) if !items.contains(item.as_str()) => {
                errors.push(err(&goal.id, "condition", format!("unknown item {item}")));
            }
            Some(Condition::ItemHeld { item, quantity })
                if *quantity == 0 || !items.contains(item.as_str()) =>
            {
                errors.push(err(
                    &goal.id,
                    "condition",
                    format!("{quantity} x {item} is not a known item and positive quantity"),
                ));
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

/// The player's progress: completed goals, items paid into steps, the chosen major goal,
/// the world conditions holding now, and pins. A snapshot, not an event log.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalProgress {
    completed: BTreeSet<String>,
    paid: BTreeMap<String, BTreeMap<String, u32>>,
    chosen: Option<String>,
    holding: BTreeSet<Condition>,
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

    /// Whether a goal is open: not complete, its `after` complete; a major goal also
    /// waits for the major goals before it in file order unless its `after` says
    /// otherwise; a step waits for its major goal to open and the steps before it.
    pub fn is_open(&self, rules: &Rules, goal: &GoalDefinition) -> bool {
        if self.completed.contains(&goal.id) || !goal.after.iter().all(|id| self.is_complete(id)) {
            return false;
        }
        let earlier = rules.goals.iter().take_while(|other| other.id != goal.id);
        match (goal.scale, &goal.part_of) {
            (Scale::Major, _) => {
                !goal.after.is_empty()
                    || earlier
                        .filter(|other| other.scale == Scale::Major)
                        .all(|other| self.is_complete(&other.id))
            }
            (Scale::Minor, Some(major)) => {
                rules
                    .goals
                    .iter()
                    .find(|other| &other.id == major)
                    .is_some_and(|major| self.is_open(rules, major))
                    && earlier
                        .filter(|other| other.part_of.as_ref() == Some(major))
                        .all(|other| self.is_complete(&other.id))
            }
            (Scale::Minor, None) => false,
        }
    }

    /// The major goal the hub shows: the player's choice while it is open, else the
    /// first open one. `None` after the last authored goal; play goes on.
    pub fn active_major<'a>(&self, rules: &'a Rules) -> Option<&'a GoalDefinition> {
        let open = |goal: &&GoalDefinition| goal.scale == Scale::Major && self.is_open(rules, goal);
        self.chosen
            .as_deref()
            .and_then(|id| rules.goals.iter().find(|goal| goal.id == id).filter(open))
            .or_else(|| rules.goals.iter().find(open))
    }

    /// The major goal's steps in order.
    pub fn steps<'a>(
        &self,
        rules: &'a Rules,
        major: &GoalDefinition,
    ) -> impl Iterator<Item = &'a GoalDefinition> {
        let id = major.id.clone();
        rules
            .goals
            .iter()
            .filter(move |goal| goal.part_of.as_ref() == Some(&id))
    }

    /// The step of a major goal the player works on now: its first open step.
    pub fn active_step<'a>(
        &self,
        rules: &'a Rules,
        major: &GoalDefinition,
    ) -> Option<&'a GoalDefinition> {
        self.steps(rules, major)
            .find(|step| self.is_open(rules, step))
    }

    /// Progress toward one item of a step: paid into its `requires`, or, for an
    /// `item_held` step, simply held.
    pub fn item_progress(
        &self,
        goal: &GoalDefinition,
        item: &str,
        inventory: &Inventory,
    ) -> Option<ItemProgress> {
        if let Some(Condition::ItemHeld {
            item: wanted,
            quantity,
        }) = &goal.condition
        {
            return (wanted == item)
                .then(|| ItemProgress::new(0, inventory.quantity(item), *quantity));
        }
        let required = goal
            .requires
            .iter()
            .filter(|stack| stack.item == item)
            .try_fold(0u32, |sum, stack| sum.checked_add(stack.quantity))?;
        if required == 0 {
            return None;
        }
        Some(ItemProgress::new(
            self.paid(&goal.id, item),
            inventory.quantity(item),
            required,
        ))
    }

    fn holds(&self, condition: &Condition, inventory: &Inventory, book: &RecipeBook) -> bool {
        match condition {
            Condition::ItemFound { item } => book.knows(item),
            Condition::ItemHeld { item, quantity } => inventory.quantity(item) >= *quantity,
            Condition::TreeGrown { mine, level } => self.holding.iter().any(|held| {
                matches!(held, Condition::TreeGrown { mine: m, level: l } if m == mine && l >= level)
            }),
            other => self.holding.contains(other),
        }
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

impl ItemProgress {
    /// Progress toward `required` of an item (a pin's total has nothing paid).
    pub fn new(paid: u32, held: u32, required: u32) -> Self {
        Self {
            paid,
            held,
            required,
            state: progress_state(paid, held, required),
        }
    }
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
    /// Hand in what the player holds toward a step's `requires`.
    Deliver(String),
    /// A world condition started to hold (a machine sounds, a tree is watered or grew).
    ConditionMet(Condition),
    /// A world condition stopped holding. Steps it completed stay complete.
    ConditionCleared(Condition),
    /// Something outside goals changed (the inventory): check the open steps again.
    Refresh,
    Crafted(String),
    /// Choose which open major goal the hub shows; `None` goes back to file order.
    Choose(Option<String>),
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
    #[error("goal does not accept item delivery: {0}")]
    NotDeliverable(String),
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

fn open_set(rules: &Rules, progress: &GoalProgress) -> BTreeSet<String> {
    rules
        .goals
        .iter()
        .filter(|goal| progress.is_open(rules, goal))
        .map(|goal| goal.id.clone())
        .collect()
}

fn apply_staged(
    rules: &Rules,
    progress: &mut GoalProgress,
    inventory: &mut Inventory,
    recipe_book: &mut RecipeBook,
    event: GoalEvent,
) -> Result<Vec<GoalOutcome>, GoalError> {
    let open_before = open_set(rules, progress);
    let mut outcomes = Vec::new();
    match event {
        GoalEvent::Deliver(id) => deliver(rules, progress, inventory, &id, &mut outcomes)?,
        GoalEvent::ConditionMet(condition) => {
            progress.holding.insert(condition);
        }
        GoalEvent::ConditionCleared(condition) => {
            progress.holding.remove(&condition);
        }
        GoalEvent::Refresh => {}
        GoalEvent::Crafted(recipe) => {
            if let Some(count) = progress.pins.0.get_mut(&recipe) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    progress.pins.0.remove(&recipe);
                    outcomes.push(GoalOutcome::PinDone(recipe));
                }
            }
        }
        GoalEvent::Choose(goal) => {
            if let Some(id) = &goal {
                let candidate = rules
                    .goals
                    .iter()
                    .find(|candidate| candidate.id == *id)
                    .ok_or_else(|| GoalError::UnknownGoal(id.clone()))?;
                if candidate.scale != Scale::Major || !progress.is_open(rules, candidate) {
                    return Err(GoalError::GoalNotOpen(id.clone()));
                }
            }
            progress.chosen = goal;
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
    settle(
        rules,
        progress,
        inventory,
        recipe_book,
        open_before,
        &mut outcomes,
    );
    Ok(outcomes)
}

fn deliver(
    rules: &Rules,
    progress: &mut GoalProgress,
    inventory: &mut Inventory,
    id: &str,
    outcomes: &mut Vec<GoalOutcome>,
) -> Result<(), GoalError> {
    let goal = rules
        .goals
        .iter()
        .find(|goal| goal.id == id)
        .ok_or_else(|| GoalError::UnknownGoal(id.into()))?;
    if goal.requires.is_empty() {
        return Err(GoalError::NotDeliverable(id.into()));
    }
    if !progress.is_open(rules, goal) {
        return Ok(());
    }
    let mut required = BTreeMap::<String, u32>::new();
    for stack in &goal.requires {
        let total = required.entry(stack.item.clone()).or_default();
        *total = total
            .checked_add(stack.quantity)
            .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
    }
    let remaining = |progress: &GoalProgress| -> Vec<Stack> {
        required
            .iter()
            .filter_map(|(item, quantity)| {
                let left = quantity.saturating_sub(progress.paid(id, item));
                (left > 0).then(|| Stack {
                    item: item.clone(),
                    quantity: left,
                })
            })
            .collect()
    };
    let delivered: Vec<_> = remaining(progress)
        .into_iter()
        .filter_map(|stack| {
            let accepted = stack.quantity.min(inventory.quantity(&stack.item));
            (accepted > 0).then_some(Stack {
                item: stack.item,
                quantity: accepted,
            })
        })
        .collect();
    if delivered.is_empty() {
        outcomes.push(GoalOutcome::NeedItems {
            goal: id.into(),
            items: remaining(progress),
        });
        return Ok(());
    }
    inventory.take_stacks(&delivered)?;
    let paid = progress.paid.entry(id.into()).or_default();
    for stack in delivered {
        let amount = paid.entry(stack.item.clone()).or_default();
        *amount = amount
            .checked_add(stack.quantity)
            .ok_or_else(|| GoalError::QuantityOverflow(stack.item.clone()))?;
        outcomes.push(GoalOutcome::Paid {
            goal: id.into(),
            item: stack.item,
            quantity: stack.quantity,
        });
    }
    Ok(())
}

/// Complete every open step that is met and every major goal whose steps are all
/// complete, until nothing changes; then report the goals that opened.
fn settle(
    rules: &Rules,
    progress: &mut GoalProgress,
    inventory: &Inventory,
    recipe_book: &mut RecipeBook,
    open_before: BTreeSet<String>,
    outcomes: &mut Vec<GoalOutcome>,
) {
    loop {
        let met = rules.goals.iter().find(|goal| {
            progress.is_open(rules, goal)
                && match goal.scale {
                    Scale::Minor => match &goal.condition {
                        Some(condition) => progress.holds(condition, inventory, recipe_book),
                        None => goal.requires.iter().all(|stack| {
                            let total: u32 = goal
                                .requires
                                .iter()
                                .filter(|other| other.item == stack.item)
                                .map(|other| other.quantity)
                                .sum();
                            progress.paid(&goal.id, &stack.item) >= total
                        }),
                    },
                    Scale::Major => progress
                        .steps(rules, goal)
                        .all(|step| progress.is_complete(&step.id)),
                }
        });
        let Some(goal) = met else { break };
        progress.completed.insert(goal.id.clone());
        outcomes.push(GoalOutcome::GoalCompleted(goal.id.clone()));
        for item in &goal.unlocks {
            recipe_book.discover(item);
            outcomes.push(GoalOutcome::Discovered(item.clone()));
        }
    }
    // Newly open goals, in file order.
    for goal in &rules.goals {
        if !open_before.contains(&goal.id) && progress.is_open(rules, goal) {
            outcomes.push(GoalOutcome::Opened(goal.id.clone()));
        }
    }
    if progress
        .chosen
        .as_ref()
        .is_some_and(|id| progress.is_complete(id))
    {
        progress.chosen = None;
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

    fn major(id: &str) -> GoalDefinition {
        GoalDefinition {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            scale: Scale::Major,
            part_of: None,
            requires: vec![],
            condition: None,
            after: vec![],
            unlocks: vec![],
        }
    }

    fn step(id: &str, major: &str) -> GoalDefinition {
        GoalDefinition {
            scale: Scale::Minor,
            part_of: Some(major.into()),
            ..self::major(id)
        }
    }

    fn when(id: &str, major: &str, condition: Condition) -> GoalDefinition {
        GoalDefinition {
            condition: Some(condition),
            ..step(id, major)
        }
    }

    fn configured() -> Condition {
        Condition::MachineConfigured {
            machine: "mudpit_burbler".into(),
        }
    }

    fn watered() -> Condition {
        Condition::TreeWatered {
            mine: "scrub_oak".into(),
        }
    }

    fn grown(level: GrowthLevel) -> Condition {
        Condition::TreeGrown {
            mine: "scrub_oak".into(),
            level,
        }
    }

    /// The shipped catalog with fixture goals: tests never depend on the shipped goal
    /// list, which designers rewrite while playing.
    ///
    /// `camp`: heartbeat (configured) → reeds (hold 12) → deliver (4 square waves,
    /// 6 reeds, unlocks clock) → bud (oak budding). `later`: `finale` (oak watered).
    fn rules() -> Rules {
        let mut rules = Rules::load(&crate::rules::shipped_files()).unwrap();
        rules.goals = vec![
            major("camp"),
            when("heartbeat", "camp", configured()),
            when(
                "reeds",
                "camp",
                Condition::ItemHeld {
                    item: "reed".into(),
                    quantity: 12,
                },
            ),
            GoalDefinition {
                requires: vec![stack("square_wave", 4), stack("reed", 6)],
                unlocks: vec!["clock".into()],
                ..step("deliver", "camp")
            },
            when("bud", "camp", grown(GrowthLevel::Budding)),
            major("later"),
            when("finale", "later", watered()),
        ];
        rules.definition.recipes.push(RecipeDefinition {
            id: "stream_kit".into(),
            name: "Stream kit".into(),
            inputs: vec![stack("reed", 6)],
            outputs: vec![stack("patch_cable", 1)],
            bpm: None,
        });
        assert_eq!(
            validate_goals(&rules.definition, &BTreeSet::new(), &rules.goals),
            []
        );
        rules
    }

    struct World {
        rules: Rules,
        progress: GoalProgress,
        inventory: Inventory,
        book: RecipeBook,
    }

    impl World {
        fn new() -> Self {
            Self {
                rules: rules(),
                progress: GoalProgress::default(),
                inventory: Inventory::default(),
                book: RecipeBook::default(),
            }
        }
        fn hold(&mut self, item: &str, quantity: u32) {
            self.inventory.add(item, quantity).unwrap();
        }
        fn try_apply(&mut self, event: GoalEvent) -> Result<Vec<GoalOutcome>, GoalError> {
            apply(
                &self.rules,
                &mut self.progress,
                &mut self.inventory,
                &mut self.book,
                event,
            )
        }
        fn apply(&mut self, event: GoalEvent) -> Vec<GoalOutcome> {
            self.try_apply(event).unwrap()
        }
        fn goal(&self, id: &str) -> &GoalDefinition {
            self.rules.goals.iter().find(|g| g.id == id).unwrap()
        }
        fn step(&self) -> Option<&str> {
            let major = self.progress.active_major(&self.rules)?;
            Some(&self.progress.active_step(&self.rules, major)?.id)
        }
        /// Through "heartbeat" and "reeds" to the delivery step, holding nothing.
        fn at_delivery(&mut self) {
            self.apply(GoalEvent::ConditionMet(configured()));
            self.hold("reed", 12);
            self.apply(GoalEvent::Refresh);
            self.inventory.take("reed", 12).unwrap();
            assert_eq!(self.step(), Some("deliver"));
        }
    }

    fn done(id: &str) -> GoalOutcome {
        GoalOutcome::GoalCompleted(id.into())
    }
    fn opened(id: &str) -> GoalOutcome {
        GoalOutcome::Opened(id.into())
    }
    fn paid(item: &str, quantity: u32) -> GoalOutcome {
        GoalOutcome::Paid {
            goal: "deliver".into(),
            item: item.into(),
            quantity,
        }
    }

    #[test]
    fn a_found_item_meets_its_step_after_it_is_used_up() {
        let mut world = World::new();
        world.rules.goals = vec![
            major("water"),
            when(
                "find_filter",
                "water",
                Condition::ItemFound {
                    item: "high_pass_filter".into(),
                },
            ),
        ];
        world.apply(GoalEvent::Refresh);
        assert_eq!(world.step(), Some("find_filter"));
        // Found and already spent (crafted into a kit) before the step is looked at again.
        world.book.discover("high_pass_filter");
        assert_eq!(world.inventory.quantity("high_pass_filter"), 0);
        assert_eq!(
            world.apply(GoalEvent::Refresh),
            [done("find_filter"), done("water")]
        );
    }

    #[test]
    fn steps_open_in_order_and_the_last_completes_the_major() {
        let mut world = World::new();
        assert_eq!(world.step(), Some("heartbeat"));
        assert!(!world.progress.is_open(&world.rules, world.goal("reeds")));
        assert!(!world.progress.is_open(&world.rules, world.goal("later")));
        assert_eq!(
            world.apply(GoalEvent::ConditionMet(configured())),
            [done("heartbeat"), opened("reeds")]
        );
        world.at_delivery();
        world.hold("square_wave", 4);
        world.hold("reed", 6);
        world.apply(GoalEvent::Deliver("deliver".into()));
        assert_eq!(
            world.apply(GoalEvent::ConditionMet(grown(GrowthLevel::Leafed))),
            [done("bud"), done("camp"), opened("later"), opened("finale")]
        );
        assert_eq!(world.step(), Some("finale"));
    }

    #[test]
    fn a_condition_holding_when_its_step_opens_completes_it_once() {
        let mut world = World::new();
        world.hold("reed", 12);
        world.apply(GoalEvent::Refresh);
        assert!(!world.progress.is_complete("reeds"), "not open yet");
        let outcomes = world.apply(GoalEvent::ConditionMet(configured()));
        assert_eq!(
            outcomes,
            [done("heartbeat"), done("reeds"), opened("deliver")]
        );
        // Spending the reeds does not undo it; repeating the events reports nothing.
        world.inventory.take("reed", 12).unwrap();
        assert_eq!(world.apply(GoalEvent::Refresh), []);
        assert_eq!(world.apply(GoalEvent::ConditionMet(configured())), []);
        assert!(world.progress.is_complete("reeds"));
    }

    #[test]
    fn a_cleared_condition_does_not_complete_a_later_step() {
        let mut world = World::new();
        world.apply(GoalEvent::ConditionMet(grown(GrowthLevel::Budding)));
        world.apply(GoalEvent::ConditionCleared(grown(GrowthLevel::Budding)));
        world.apply(GoalEvent::ConditionMet(configured()));
        world.hold("reed", 12);
        world.apply(GoalEvent::Refresh);
        world.hold("square_wave", 4);
        world.apply(GoalEvent::Deliver("deliver".into()));
        assert_eq!(world.step(), Some("bud"));
        world.apply(GoalEvent::ConditionMet(grown(GrowthLevel::Budding)));
        assert!(world.progress.is_complete("bud"));
    }

    #[test]
    fn thresholds_and_progress() {
        for (held, state) in [
            (0, ProgressState::Empty),
            (1, ProgressState::Low),
            (2, ProgressState::High),
            (4, ProgressState::Full),
        ] {
            let mut world = World::new();
            world.hold("square_wave", held);
            let progress = world
                .progress
                .item_progress(world.goal("deliver"), "square_wave", &world.inventory)
                .unwrap();
            assert_eq!((progress.held, progress.required), (held, 4));
            assert_eq!(progress.state, state, "held {held}");
        }
        // An item_held step shows what is held against its quantity.
        let mut world = World::new();
        world.hold("reed", 3);
        let reeds = world
            .progress
            .item_progress(world.goal("reeds"), "reed", &world.inventory)
            .unwrap();
        assert_eq!((reeds.paid, reeds.held, reeds.required), (0, 3, 12));
    }

    #[test]
    fn partial_then_completing_delivery() {
        let mut world = World::new();
        world.at_delivery();
        world.hold("square_wave", 1);
        world.hold("reed", 10);
        assert_eq!(
            world.apply(GoalEvent::Deliver("deliver".into())),
            [paid("reed", 6), paid("square_wave", 1)]
        );
        assert_eq!(world.inventory.quantity("reed"), 4);
        let progress = world
            .progress
            .item_progress(world.goal("deliver"), "square_wave", &world.inventory)
            .unwrap();
        assert_eq!((progress.paid, progress.held), (1, 0));

        world.hold("square_wave", 5);
        assert_eq!(
            world.apply(GoalEvent::Deliver("deliver".into())),
            [
                paid("square_wave", 3),
                done("deliver"),
                GoalOutcome::Discovered("clock".into()),
                opened("bud"),
            ]
        );
        assert_eq!(world.inventory.quantity("square_wave"), 2);
        assert!(world.book.knows("clock"));
        // Repeated event: nothing more consumed or reported.
        assert_eq!(world.apply(GoalEvent::Deliver("deliver".into())), []);
        assert_eq!(world.inventory.quantity("square_wave"), 2);
    }

    #[test]
    fn nothing_to_give_changes_nothing() {
        let mut world = World::new();
        world.at_delivery();
        world.hold("patch_cable", 3);
        let before = (world.inventory.clone(), world.progress.clone());
        assert_eq!(
            world.apply(GoalEvent::Deliver("deliver".into())),
            [GoalOutcome::NeedItems {
                goal: "deliver".into(),
                items: vec![stack("reed", 6), stack("square_wave", 4)],
            }]
        );
        assert_eq!((world.inventory.clone(), world.progress.clone()), before);
        assert_eq!(
            world.try_apply(GoalEvent::Deliver("heartbeat".into())),
            Err(GoalError::NotDeliverable("heartbeat".into()))
        );
    }

    #[test]
    fn choosing_needs_an_open_major_and_keeps_paid_amounts() {
        let mut world = World::new();
        assert_eq!(
            world.try_apply(GoalEvent::Choose(Some("later".into()))),
            Err(GoalError::GoalNotOpen("later".into()))
        );
        assert_eq!(
            world.try_apply(GoalEvent::Choose(Some("heartbeat".into()))),
            Err(GoalError::GoalNotOpen("heartbeat".into()))
        );
        world.at_delivery();
        world.hold("square_wave", 2);
        world.apply(GoalEvent::Deliver("deliver".into()));
        world.apply(GoalEvent::Choose(Some("camp".into())));
        world.apply(GoalEvent::Choose(None));
        assert_eq!(world.progress.paid("deliver", "square_wave"), 2);
    }

    #[test]
    fn after_the_last_goal_nothing_is_active_and_play_continues() {
        let mut world = World::new();
        world.at_delivery();
        world.hold("square_wave", 4);
        world.hold("reed", 6);
        world.apply(GoalEvent::Deliver("deliver".into()));
        world.apply(GoalEvent::ConditionMet(grown(GrowthLevel::FullBloom)));
        world.apply(GoalEvent::ConditionMet(watered()));
        assert!(
            world
                .rules
                .goals
                .iter()
                .all(|g| world.progress.is_complete(&g.id))
        );
        assert!(world.progress.active_major(&world.rules).is_none());
        world.apply(GoalEvent::Pin {
            recipe: "stream_kit".into(),
            count: 1,
        });
    }

    #[test]
    fn pin_totals_and_crafting_lowers_the_pin() {
        let mut world = World::new();
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

    fn errors(goals: Vec<GoalDefinition>) -> Vec<(String, String, String)> {
        let rules = Rules::load(&crate::rules::shipped_files()).unwrap();
        validate_goals(&rules.definition, &BTreeSet::new(), &goals)
            .into_iter()
            .map(|e| (e.id, e.field, e.message))
            .collect()
    }

    #[test]
    fn validation_names_the_goal_and_field() {
        let found = errors(vec![
            major("lonely"),
            when("orphan", "nowhere", configured()),
            when("nested", "orphan", configured()),
            GoalDefinition {
                condition: Some(configured()),
                ..major("bossy")
            },
            when("bossy_step", "bossy", configured()),
            step("empty", "bossy"),
            when(
                "greedy",
                "bossy",
                Condition::ItemHeld {
                    item: "unobtainium".into(),
                    quantity: 1,
                },
            ),
            when(
                "far",
                "bossy",
                Condition::TreeGrown {
                    mine: "baobab".into(),
                    level: GrowthLevel::Budding,
                },
            ),
            when(
                "smoky",
                "bossy",
                Condition::FireLit {
                    buildable: "bonfire".into(),
                },
            ),
            when(
                "builder",
                "bossy",
                Condition::Built {
                    buildable: "tower".into(),
                },
            ),
        ]);
        for (id, field, words) in [
            ("lonely", "scale", "at least one minor step"),
            ("orphan", "part_of", "unknown major goal nowhere"),
            ("nested", "part_of", "not a major goal"),
            ("bossy", "condition", "completes with its steps"),
            ("empty", "requires", "needs `requires` or `condition`"),
            ("greedy", "condition", "unobtainium"),
            ("far", "condition", "unknown mine baobab"),
            ("smoky", "condition", "unknown buildable bonfire"),
            ("builder", "condition", "unknown buildable tower"),
        ] {
            assert!(
                found
                    .iter()
                    .any(|(i, f, m)| i == id && f == field && m.contains(words)),
                "{id}.{field}: {words}\n{found:#?}"
            );
        }
    }

    #[test]
    fn the_opening_starts_by_building_a_campfire_from_the_reeds_you_carry() {
        // User, 2026-11-18: one campfire's reeds to start, and building it the first step.
        let rules = crate::shipped_rules().expect("rules");
        let first = rules
            .goals
            .iter()
            .find(|g| g.part_of.as_deref() == Some("build_a_camp"))
            .expect("Build a camp has steps");
        assert_eq!(
            first.condition,
            Some(Condition::Built {
                buildable: "campfire".into()
            })
        );
        let mut inventory = Inventory::default();
        inventory.grant(&rules.start).expect("start fits");
        let catalog = crate::shipped_catalog().expect("catalog");
        assert!(catalog.build("campfire", 1, &mut inventory).is_ok());
    }

    #[test]
    fn a_prerequisite_cycle_is_rejected() {
        let found = errors(vec![
            GoalDefinition {
                after: vec!["b".into()],
                ..major("a")
            },
            when("a1", "a", configured()),
            GoalDefinition {
                after: vec!["a".into()],
                ..major("b")
            },
            when("b1", "b", configured()),
        ]);
        assert!(
            found
                .iter()
                .any(|(_, f, m)| f == "after" && m.contains("cycle")),
            "{found:?}"
        );
    }

    #[test]
    fn an_unknown_growth_level_fails_to_parse() {
        let row = r#"{"kind": "tree_grown", "mine": "scrub_oak", "level": "towering"}"#;
        assert!(serde_json::from_str::<Condition>(row).is_err());
    }
}
