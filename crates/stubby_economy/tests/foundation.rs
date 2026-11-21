use stubby_economy::{
    ledger::{LabourLedger, LedgerError},
    rules::{BoundRule, RuleKind, Stock, resolve},
    time::{Checkpoint, CheckpointCursor, Schedule},
};

fn stock(entries: &[(&str, f64)]) -> Stock {
    entries
        .iter()
        .map(|&(id, amount)| (id.into(), amount))
        .collect()
}

fn rule(
    id: &str,
    kind: RuleKind,
    units: u32,
    inputs: &[(&str, f64)],
    outputs: &[(&str, f64)],
) -> BoundRule {
    BoundRule {
        id: id.into(),
        kind,
        units,
        inputs: stock(inputs),
        outputs: stock(outputs),
    }
}

#[test]
fn producers_and_converters_run_before_consumers_and_upkeep() {
    let rules = [
        rule("maintenance", RuleKind::Upkeep, 1, &[("clean", 1.)], &[]),
        rule("rations", RuleKind::Consumer, 2, &[("clean", 1.)], &[]),
        rule(
            "filter",
            RuleKind::Converter,
            2,
            &[("mud", 1.)],
            &[("clean", 1.)],
        ),
        rule("burbler", RuleKind::Producer, 2, &[], &[("mud", 1.)]),
    ];
    let outcome = resolve(&Stock::new(), &rules).unwrap();
    assert_eq!(
        outcome
            .applied
            .iter()
            .map(|rule| rule.id.as_str())
            .collect::<Vec<_>>(),
        ["burbler", "filter", "rations", "maintenance"]
    );
    assert_eq!(outcome.deficits, stock(&[("clean", 1.)]));
}

#[test]
fn converters_use_only_whole_affordable_batches() {
    let rules = [rule(
        "cook",
        RuleKind::Converter,
        6,
        &[("crop", 1.5), ("water", 0.5)],
        &[("food", 2.)],
    )];
    let outcome = resolve(&stock(&[("crop", 4.), ("water", 0.75)]), &rules).unwrap();
    assert_eq!(outcome.applied[0].units, 1);
    assert_eq!(
        outcome.stock,
        stock(&[("crop", 2.5), ("water", 0.25), ("food", 2.)])
    );
    assert!(outcome.deficits.is_empty());
}

#[test]
fn ration_shortfalls_record_unmet_demand_and_clamp_stock() {
    let rules = [rule("dawn", RuleKind::Consumer, 12, &[("water", 1.)], &[])];
    let starting = stock(&[("water", 8.)]);
    let outcome = resolve(&starting, &rules).unwrap();
    assert_eq!(outcome.stock, stock(&[("water", 0.)]));
    assert_eq!(outcome.deficits, stock(&[("water", 4.)]));
    assert_eq!(starting, stock(&[("water", 8.)]));
}

#[test]
fn jumping_across_checkpoints_matches_steps_and_resumes_from_save() {
    let schedule = Schedule::new(3_600, 1_350, 2_700, 3_150).unwrap();
    let resolve_through =
        |cursor: &mut CheckpointCursor, seconds, stock: &mut Stock, events: &mut Vec<_>| {
            cursor
                .advance(schedule, seconds, |due| {
                    let ration = match due.checkpoint {
                        Checkpoint::Dawn | Checkpoint::Dusk => Some("water"),
                        Checkpoint::Midday => Some("food"),
                        Checkpoint::Night => None,
                    };
                    let rules: Vec<_> = ration
                        .into_iter()
                        .map(|ration| rule(ration, RuleKind::Consumer, 1, &[(ration, 1.)], &[]))
                        .collect();
                    let outcome = resolve(stock, &rules)?;
                    *stock = outcome.stock;
                    events.push((due, outcome.deficits));
                    Ok::<_, stubby_economy::rules::RuleError>(())
                })
                .unwrap();
        };
    let starting = stock(&[("water", 2.), ("food", 1.)]);
    let (mut jump_cursor, mut jump_stock, mut jump_events) =
        (CheckpointCursor::default(), starting.clone(), Vec::new());
    resolve_through(&mut jump_cursor, 6_300, &mut jump_stock, &mut jump_events);

    let (mut cursor, mut stepped_stock, mut stepped_events) =
        (CheckpointCursor::default(), starting, Vec::new());
    for second in [0, 1_350, 2_700, 3_150, 3_600, 4_950, 6_300] {
        resolve_through(&mut cursor, second, &mut stepped_stock, &mut stepped_events);
        cursor = serde_json::from_str(&serde_json::to_string(&cursor).unwrap()).unwrap();
    }
    resolve_through(&mut cursor, 2_700, &mut stepped_stock, &mut stepped_events);
    assert_eq!(jump_events.len(), 7);
    assert_eq!(
        (jump_cursor, jump_stock, jump_events),
        (cursor, stepped_stock, stepped_events)
    );

    let mut failed = CheckpointCursor::default();
    assert_eq!(
        failed.advance(schedule, 0, |_| Err("blocked")),
        Err("blocked")
    );
    assert_eq!(failed.last_processed(), None);
    failed.advance(schedule, 0, |_| Ok::<_, ()>(())).unwrap();
    assert_eq!(failed.last_processed(), Some(0));
}

#[test]
fn labour_reservations_are_atomic_idempotent_and_restorable() {
    let mut ledger = LabourLedger::new(5);
    ledger.reserve("build", 3).unwrap();
    ledger.reserve("build", 3).unwrap();
    ledger.reserve("tend", 2).unwrap();
    assert_eq!(
        ledger.reserve("tend", 3),
        Err(LedgerError::Shortfall { missing: 1 })
    );
    assert_eq!(ledger.reservation("tend"), 2);
    ledger.set_capacity(2);
    ledger.reserve("build", 3).unwrap();
    ledger.reserve("build", 2).unwrap();
    assert_eq!(ledger.available(), 0);

    let snapshot = serde_json::to_string(&ledger.snapshot()).unwrap();
    let mut restored = LabourLedger::restore(serde_json::from_str(&snapshot).unwrap()).unwrap();
    assert_eq!(restored, ledger);
    assert_eq!(restored.release("tend"), 2);
    restored.reserve("build", 0).unwrap();
    assert_eq!(restored.available(), 2);
}
