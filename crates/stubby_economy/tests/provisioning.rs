use std::{collections::BTreeMap, num::NonZeroU64};

use stubby_economy::{
    housing::{Dwelling, DwellingId},
    provision::{
        AirSupply, ProvisionError, QuarterNote, RationStock, RationStockError, Rations,
        STANDARD_BPM, Supply,
    },
    roster::{AppearanceCounts, DancerId},
    state::Settlement,
};

const SAMPLE_RATE: u64 = 48_000;
const DAY_SECONDS: u64 = 3_600;
const SITE_CLOCK: u64 = 1;

const FOOD: &str = "edible_crop";
const WATER: &str = "filtered_water";

fn stock_items() -> RationStock<'static> {
    RationStock::bind(FOOD, WATER, |id| matches!(id, FOOD | WATER)).unwrap()
}

/// One shared air patch. The fixture replenishes by elapsed samples and
/// therefore has the same cleaning rate at both worksite tempos.
struct Grove {
    air: f64,
    capacity: f64,
    refresh_per_second: f64,
    last_sample: u64,
    locations: BTreeMap<DancerId, bool>,
}

impl Grove {
    fn new(capacity: f64, refresh_per_second: f64) -> Self {
        Self {
            air: capacity,
            capacity,
            refresh_per_second,
            last_sample: 0,
            locations: BTreeMap::new(),
        }
    }

    fn refresh(&mut self, sample: u64) {
        let elapsed = (sample - self.last_sample) as f64 / SAMPLE_RATE as f64;
        self.air = (self.air + elapsed * self.refresh_per_second).min(self.capacity);
        self.last_sample = sample;
    }
}

impl AirSupply for Grove {
    fn draw(&mut self, dancer: DancerId, note: QuarterNote, demand: f64) -> f64 {
        self.refresh(note.sample);
        if self.locations.get(&dancer) != Some(&true) {
            return 0.0;
        }
        let met = self.air.min(demand);
        self.air -= met;
        met
    }
}

fn resident_fixture(count: u32, grove: &mut Grove) -> Settlement {
    let mut settlement = Settlement::new(42);
    settlement.stock.insert(FOOD.into(), 4.0);
    settlement.stock.insert(WATER.into(), 8.0);
    settlement.add_dwelling(
        DwellingId(1),
        Dwelling {
            kind: "cottage".into(),
            capacity: 4.try_into().unwrap(),
        },
    );
    for _ in 0..count {
        let dancer = settlement
            .admit(
                AppearanceCounts {
                    body_shapes: 4.try_into().unwrap(),
                    skin_tones: 4.try_into().unwrap(),
                },
                SITE_CLOCK,
            )
            .unwrap();
        grove.locations.insert(dancer, true);
    }
    settlement
}

fn rates() -> Rations {
    let reference_notes = NonZeroU64::new(STANDARD_BPM * DAY_SECONDS / 60).unwrap();
    Rations {
        food: 1.0,
        clean_water: 2.0,
        fresh_air: 1.0,
    }
    .per_note(reference_notes)
}

fn notes(bpm: u64) -> impl Iterator<Item = QuarterNote> {
    (1..=bpm * DAY_SECONDS / 60).map(move |index| QuarterNote {
        clock_id: SITE_CLOCK,
        index,
        sample: index * 60 * SAMPLE_RATE / bpm,
    })
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn reference_day_and_double_tempo_scale_work_and_all_three_needs() {
    for (bpm, scale) in [(120, 1_u32), (240, 2)] {
        let mut grove = Grove::new(4.0, 0.0);
        let mut state = resident_fixture(1, &mut grove);
        let mut work_notes = 0;
        let mut fresh_air_demand = 0.0;

        for note in notes(bpm) {
            let reports = state
                .provision_note(note, stock_items(), rates(), &mut grove)
                .unwrap()
                .unwrap();
            let [ration] = reports.as_slice() else {
                panic!("expected one resident on the worksite clock");
            };
            // The headless worksite advances one unit on each accepted note.
            work_notes += 1;
            fresh_air_demand += ration.fresh_air.demand;
            close(ration.food.shortfall(), 0.0);
            close(ration.clean_water.shortfall(), 0.0);
            close(ration.fresh_air.shortfall(), 0.0);
        }

        assert_eq!(work_notes, 7_200 * scale);
        let scale = f64::from(scale);
        close(4.0 - state.stock[FOOD], scale);
        close(8.0 - state.stock[WATER], 2.0 * scale);
        close(fresh_air_demand, scale);
        close(4.0 - grove.air, scale);
    }
}

#[test]
fn shared_air_depletes_and_recovers_independently_of_clock() {
    let mut grove = Grove::new(0.01, 0.0001);
    let mut state = resident_fixture(2, &mut grove);
    state.stock.insert(FOOD.into(), 0.0);
    let mut shortfall = 0.0;
    let mut delivered = 0.0;
    let active_notes: Vec<_> = notes(240).take(100).collect();
    for note in &active_notes {
        let reports = state
            .provision_note(*note, stock_items(), rates(), &mut grove)
            .unwrap()
            .unwrap();
        for ration in reports {
            delivered += ration.fresh_air.met;
            shortfall += ration.fresh_air.shortfall();
            close(ration.food.shortfall(), rates().food);
        }
    }
    assert!(shortfall > 0.0);
    assert!(grove.air < grove.capacity);
    let last_sample = active_notes.last().unwrap().sample;
    assert!(delivered <= grove.capacity + 25.0 * grove.refresh_per_second + 1e-9);

    for dancer in state
        .roster()
        .dancers()
        .map(|dancer| dancer.id)
        .collect::<Vec<_>>()
    {
        state.remove_dancer(dancer).unwrap();
    }
    grove.refresh(last_sample + 120 * SAMPLE_RATE);
    close(grove.air, grove.capacity);
}

#[test]
fn provisioning_resumes_after_reload_without_double_charging() {
    let mut grove = Grove::new(1.0, 0.0);
    let mut state = resident_fixture(1, &mut grove);
    let mut events = notes(120);
    let first = events.next().unwrap();
    let next = events.next().unwrap();
    state
        .provision_note(first, stock_items(), rates(), &mut grove)
        .unwrap()
        .unwrap();
    let saved = serde_json::to_string(&state).unwrap();
    let mut resumed: Settlement = serde_json::from_str(&saved).unwrap();
    let before = grove.air;

    assert!(
        resumed
            .provision_note(first, stock_items(), rates(), &mut grove)
            .unwrap()
            .is_none()
    );
    assert_eq!(state, resumed);
    assert_eq!(grove.air, before);
    let later = events.next().unwrap();
    assert_eq!(
        resumed.provision_note(later, stock_items(), rates(), &mut grove),
        Err(ProvisionError::NoteGap {
            clock_id: SITE_CLOCK,
            expected: next.index,
            received: later.index,
        }),
    );
    assert_eq!(state, resumed);
    assert_eq!(grove.air, before);
    resumed
        .provision_note(next, stock_items(), rates(), &mut grove)
        .unwrap()
        .unwrap();
    close(4.0 - resumed.stock[FOOD], 2.0 * rates().food);
    close(1.0 - grove.air, 2.0 * rates().fresh_air);
}

#[test]
fn stock_bindings_require_distinct_catalog_items() {
    assert_eq!(
        RationStock::bind(FOOD, "missing", |id| id == FOOD),
        Err(RationStockError::UnknownItem("missing".into())),
    );
    assert_eq!(
        RationStock::bind(FOOD, FOOD, |id| id == FOOD),
        Err(RationStockError::DuplicateItem(FOOD.into())),
    );
}

#[test]
fn exactly_one_daily_budget_does_not_report_shortfall_from_rounding() {
    let mut grove = Grove::new(1.0, 0.0);
    let mut state = resident_fixture(1, &mut grove);
    state.stock.insert(FOOD.into(), 1.0);
    state.stock.insert(WATER.into(), 2.0);

    for note in notes(STANDARD_BPM) {
        let report = state
            .provision_note(note, stock_items(), rates(), &mut grove)
            .unwrap()
            .unwrap();
        let ration = report[0];
        assert_eq!(ration.food.shortfall(), 0.0);
        assert_eq!(ration.clean_water.shortfall(), 0.0);
        assert_eq!(ration.fresh_air.shortfall(), 0.0);
    }
    let tiny_ration = Supply {
        demand: 1e-15,
        met: 0.0,
    };
    assert_eq!(tiny_ration.shortfall(), tiny_ration.demand);
    let partial = Supply {
        demand: 1.0,
        met: 0.999,
    };
    close(partial.shortfall(), 0.001);
}
