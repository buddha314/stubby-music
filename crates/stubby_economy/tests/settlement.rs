use stubby_economy::{
    housing::{Dwelling, DwellingId},
    roster::{AppearanceCounts, Task},
    state::Settlement,
};

fn cottage() -> Dwelling {
    Dwelling {
        kind: "cottage".into(),
        capacity: 4.try_into().unwrap(),
    }
}

fn appearances() -> AppearanceCounts {
    AppearanceCounts {
        body_shapes: 4.try_into().unwrap(),
        skin_tones: 4.try_into().unwrap(),
    }
}

#[test]
fn seeded_roster_and_homes_survive_save_and_continue() {
    let build = |order: [u64; 2]| {
        let mut state = Settlement::new(42);
        for id in order {
            assert!(state.add_dwelling(DwellingId(id), cottage()));
        }
        for clock in 100..105 {
            state.admit(appearances(), clock).unwrap();
        }
        state
    };
    let mut state = build([1, 2]);
    assert_eq!(state, build([2, 1]));
    assert_eq!(state.roster().population(), 5);
    assert_eq!(state.roster().profession_count("laborer"), 5);
    assert_eq!(state.roster().idle_count(), 5);
    for id in [1, 2] {
        let housed = state
            .roster()
            .dancers()
            .filter(|dancer| dancer.home == Some(DwellingId(id)))
            .count();
        assert!((1..=4).contains(&housed));
    }
    for (dancer, clock) in state.roster().dancers().zip(100..105) {
        assert!(dancer.appearance.body_shape < 4);
        assert!(dancer.appearance.skin_tone < 4);
        assert_eq!(dancer.task, Task::Idle);
        assert_eq!(dancer.clock_id, clock);
    }

    let saved = serde_json::to_string(&state).unwrap();
    let mut loaded: Settlement = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded, state);
    assert_eq!(
        loaded.admit(appearances(), 200),
        state.admit(appearances(), 200)
    );
    assert_eq!(loaded, state);
}

#[test]
fn displaced_residents_get_first_claim_on_new_vacancies() {
    let mut state = Settlement::new(42);
    let first = DwellingId(1);
    let second = DwellingId(2);
    assert!(state.add_dwelling(first, cottage()));
    assert!(!state.add_dwelling(first, cottage()));

    let oldest = state.admit(appearances(), 100).unwrap();
    for clock in 101..104 {
        state.admit(appearances(), clock).unwrap();
    }
    assert!(state.admit(appearances(), 200).is_none());
    state.remove_dwelling(first).unwrap();
    assert_eq!(state.roster().homeless_count(), 4);

    let mut single = cottage();
    single.kind = "single".into();
    single.capacity = 1.try_into().unwrap();
    assert!(state.add_dwelling(second, single));
    assert_eq!(state.roster().homeless_count(), 3);
    assert_eq!(state.roster().dancers().next().unwrap().home, Some(second));

    state.remove_dancer(oldest).unwrap();
    assert_eq!(state.roster().homeless_count(), 2);
    assert_eq!(state.roster().dancers().next().unwrap().home, Some(second));

    assert!(state.add_dwelling(first, cottage()));
    assert_eq!(state.roster().homeless_count(), 0);
    assert_eq!(state.admit(appearances(), 200).unwrap().0, 4);
}
