use super::*;
use std::collections::HashMap;
use stubby_audio::{Command, ModuleId};

fn shipped() -> Modules {
    shipped_modules().unwrap_or_else(|e| panic!("{e}"))
}

/// The shipped sources with one module's definition replaced.
fn load_with(id: &str, definition: &str) -> Result<Modules, RackErrors> {
    let sources: Vec<Source> = shipped_sources()
        .into_iter()
        .map(|s| {
            if s.file.contains(&format!("/{id}/")) {
                Source { definition, ..s }
            } else {
                s
            }
        })
        .collect();
    Modules::load(&sources)
}

#[test]
fn shipped_modules_load_and_match_their_panels() {
    let modules = shipped();
    assert_eq!(modules.0.len(), 6);
    // The snail sequencer's step buttons bind like knobs, 0..1; its CV knobs 0..2 V.
    let gate = modules.knob("snail_sequencer", "gate_2").unwrap();
    assert_eq!((gate.min, gate.max, gate.param), (0., 1., "gate_2"));
    assert_eq!(modules.knob("snail_sequencer", "cv_1").unwrap().max, 2.);
    assert_eq!(
        modules.jack_is_output("snail_sequencer", "clock"),
        Some(false)
    );
    assert_eq!(modules.jack_is_output("snail_sequencer", "cv"), Some(true));
    assert_eq!(modules.jack_is_output("volume", "trig"), Some(false));
    assert_eq!(modules.jack_is_output("volume", "out"), Some(true));
    assert_eq!(
        modules.knob("volume", "gain").map(|k| k.default),
        Some(0.25)
    );
}

#[test]
fn every_error_is_reported_at_once() {
    let definition = r#"{ "id": "volume", "parts": [{ "part": "vca", "kind": "Vca" }],
        "jacks": { "trig": "vca.cv", "in": "vca.in", "out": "vca.outt" },
        "knobs": { "gain": { "param": "vca.gain", "default": 5.0 } } }"#;
    let errors = load_with("volume", definition).expect_err("two mistakes").0;
    let fields: Vec<_> = errors
        .iter()
        .map(|e| (e.module.as_str(), e.field.as_str()))
        .collect();
    assert!(fields.contains(&("volume", "jacks.out")), "{errors:?}");
    assert!(
        fields.contains(&("volume", "knobs.gain.default")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .all(|e| e.file.ends_with("volume/module.json"))
    );
}

#[test]
fn an_unbound_panel_port_is_drift() {
    let definition = r#"{ "id": "square_vco", "parts": [{ "part": "vco", "kind": "SquareVco" }],
        "jacks": { "out": "vco.out" },
        "knobs": { "freq": { "param": "vco.freq", "default": 261.63 } } }"#;
    let errors = load_with("square_vco", definition)
        .expect_err("voct unbound")
        .0;
    assert!(
        errors
            .iter()
            .any(|e| e.field == "jacks" && e.message.contains("voct")),
        "{errors:?}"
    );
}

#[test]
fn an_output_cannot_be_bound_to_an_input() {
    let definition = r#"{ "id": "volume", "parts": [{ "part": "vca", "kind": "Vca" }],
        "jacks": { "trig": "vca.cv", "in": "vca.in", "out": "vca.in" },
        "knobs": { "gain": { "param": "vca.gain", "default": 0.25 } } }"#;
    let errors = load_with("volume", definition).expect_err("direction").0;
    assert!(
        errors
            .iter()
            .any(|e| e.field == "jacks.out" && e.message.contains("output")),
        "{errors:?}"
    );
}

#[test]
fn unknown_fields_and_kinds_fail() {
    let definition = r#"{ "id": "volume", "parts": [{ "part": "vca", "kind": "Vca" }],
        "jacks": {}, "envelope": true }"#;
    assert!(load_with("volume", definition).is_err());
    let definition = r#"{ "id": "volume", "parts": [{ "part": "vca", "kind": "Wobbler" }],
        "jacks": { "trig": "vca.cv", "in": "vca.in", "out": "vca.out" } }"#;
    let errors = load_with("volume", definition).expect_err("kind").0;
    assert!(
        errors.iter().any(|e| e.field == "parts[0].kind"),
        "{errors:?}"
    );
}

#[test]
fn rack_data_names_no_game_concepts() {
    // The rack is an independent system: its data names engine parts and panel
    // jacks, never items, creatures or machines (a module's own ID is its name).
    for source in shipped_sources() {
        let body: String = source
            .definition
            .lines()
            .filter(|l| !l.contains("\"id\""))
            .collect();
        for word in [
            "lightning",
            "bug",
            "burbler",
            "square_wave",
            "reed",
            "item",
            "inventory",
        ] {
            assert!(!body.contains(word), "{} names `{word}`", source.file);
        }
    }
}

// ------------------------------------------------------------------ planning

fn jack(place: &str, port: &str) -> Jack {
    Jack {
        place: place.into(),
        port: port.into(),
    }
}

fn burbler_rack() -> HashMap<String, String> {
    [
        ("clock", "burbler_clock"),
        ("vco", "square_vco"),
        ("volume", "volume"),
    ]
    .into_iter()
    .map(|(p, m)| (p.to_owned(), m.to_owned()))
    .collect()
}

fn plan_with(cables: &[(Jack, Jack)], knob: impl Fn(&str, &str) -> Option<f32>) -> Plan {
    let modules = shipped();
    let rack = burbler_rack();
    let provided = HashMap::from([(("clock".to_owned(), "clock".to_owned()), ModuleId(1))]);
    let out = jack("volume", "out");
    let mut next = 100;
    modules.plan(
        &RackPatch {
            modules: &rack,
            cables,
            normalled_output: Some(&out),
            provided: &provided,
        },
        knob,
        || {
            next += 1;
            Some(ModuleId(next))
        },
    )
}

fn connects(plan: &Plan) -> usize {
    plan.commands
        .iter()
        .filter(|c| matches!(c, Command::Connect { .. }))
        .count()
}

#[test]
fn the_plan_is_exactly_the_rack() {
    // Unpatched: the VCO and VCA exist, only the normalled output is wired, the
    // machine's own clock is used rather than a new one.
    let plan = plan_with(&[], |_, _| None);
    assert_eq!(plan.owned.len(), 2);
    assert_eq!(connects(&plan), 1);
    assert_eq!(
        plan.parts[&("clock".to_owned(), "clock".to_owned())],
        ModuleId(1)
    );

    let gate = (jack("clock", "gate"), jack("volume", "trig"));
    assert_eq!(
        connects(&plan_with(std::slice::from_ref(&gate), |_, _| None)),
        2
    );

    let both = [gate, (jack("vco", "out"), jack("volume", "in"))];
    let plan = plan_with(&both, |_, _| None);
    assert_eq!(connects(&plan), 3);
    assert_eq!(plan.owned.len(), 2, "no hidden modules");
}

#[test]
fn knobs_reach_the_parts_they_turn() {
    let kind = |plan: &Plan, place: &str, part: &str| {
        let id = plan.parts[&(place.to_owned(), part.to_owned())];
        plan.commands.iter().find_map(|c| match c {
            Command::AddModule { id: i, kind } if *i == id => Some(*kind),
            _ => None,
        })
    };
    // Defaults from the definitions.
    let plan = plan_with(&[], |_, _| None);
    assert_eq!(
        kind(&plan, "volume", "vca"),
        Some(ModuleKind::Vca { gain: 0.25 })
    );
    // Values from the caller: a turned knob, or a starting value it supplies.
    let plan = plan_with(&[], |place, knob| match (place, knob) {
        ("vco", "freq") => Some(440.),
        ("volume", "gain") => Some(0.8),
        _ => None,
    });
    assert_eq!(
        kind(&plan, "vco", "vco"),
        Some(ModuleKind::SquareVco { freq_hz: 440. })
    );
    assert_eq!(
        kind(&plan, "volume", "vca"),
        Some(ModuleKind::Vca { gain: 0.8 })
    );
    // A caller's value past the knob's travel is held at its end.
    let plan = plan_with(&[], |_, knob| (knob == "gain").then_some(3.0));
    assert_eq!(
        kind(&plan, "volume", "vca"),
        Some(ModuleKind::Vca { gain: 1.0 })
    );
}

#[test]
fn the_same_rack_gets_the_same_ids() {
    let a = plan_with(&[], |_, _| None);
    let b = plan_with(&[], |_, _| None);
    assert_eq!(a.parts, b.parts);
}

#[test]
fn a_stepped_knob_clicks_through_the_24_bpm_grid_inside_its_travel() {
    let knob = Knob {
        part: "clock".into(),
        param: "bpm",
        default: 120.,
        min: 30.,
        max: 240.,
        step: Some(24.),
        quantize: None,
    };
    assert_eq!(knob.snap(130.), 120.);
    assert_eq!(knob.snap(137.), 144.);
    assert_eq!(knob.snap(30.), 48., "the lowest multiple inside the travel");
    assert_eq!(knob.snap(250.), 240.);
}

#[test]
fn a_semitone_knob_clicks_to_notes_inside_its_travel() {
    let knob = Knob {
        part: "vco".into(),
        param: "freq",
        default: 261.63,
        min: 1.,
        max: 20000.,
        step: None,
        quantize: Some(Quantize::Semitone),
    };
    // The reed water purifier's patch (add-reed-water-purifier): C2, C4, and the cutoff C5.
    for (hz, note, exact) in [
        (65.625, "C2", 65.406),
        (261.63, "C4", 261.626),
        (531.5, "C5", 523.251),
    ] {
        let snapped = knob.snap(hz);
        assert!((snapped - exact).abs() < 0.01, "{hz} -> {snapped}");
        assert_eq!(note_name(snapped), note);
    }
    assert_eq!(note_name(knob.nudge(261.63, 1)), "C#4");
    assert_eq!(note_name(knob.nudge(261.63, -12)), "C3");
    assert!(knob.nudge(19000., 24) <= 20000., "stays inside the travel");
}
