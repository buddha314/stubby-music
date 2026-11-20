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
    // Every shipped module loads (the stream blank too: a panel with no parts or jacks).
    assert_eq!(modules.0.len(), crate::shipped_sources().len());
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

#[test]
fn targets_match_notes_and_values() {
    let note = |n: &str| Target {
        place: "vco_low".into(),
        knob: "freq".into(),
        note: Some(n.into()),
        value: None,
    };
    assert!(note("C2").met(65.406));
    assert!(!note("C2").met(69.296), "C#2 is not C2");
    assert_eq!(note("C2").problem(), None);
    assert!(note("H2").problem().is_some());
    let gate = Target {
        place: "seq".into(),
        knob: "gate_2".into(),
        note: None,
        value: Some(1.),
    };
    assert!(gate.met(1.) && !gate.met(0.));
    assert!((note_hz("A4").unwrap() - 440.).abs() < 1e-3);
}

#[test]
fn purifier_profile_renders_its_clock_gated_patch_at_the_outlet() {
    use stubby_audio::{Engine, EngineConfig, ModuleKind, PortRef};

    #[derive(Deserialize)]
    struct Profile {
        configure: Configuration,
    }
    #[derive(Deserialize)]
    struct Configuration {
        places: Vec<Place>,
        cables: Vec<Cable>,
        normalled_output: Jack,
    }
    #[derive(Deserialize)]
    struct Place {
        place: String,
        module: String,
        #[serde(default)]
        borrowed: bool,
    }
    #[derive(Deserialize)]
    struct Cable {
        from: Jack,
        to: Jack,
    }

    let profile: Profile = serde_json::from_str(include_str!(
        "../../../assets/machines/reed_water_purifier/asset.json"
    ))
    .unwrap();
    let config = profile.configure;
    let clock = ModuleId(1);
    let modules = config
        .places
        .iter()
        .map(|place| (place.place.clone(), place.module.clone()))
        .collect();
    let cables: Vec<_> = config
        .cables
        .into_iter()
        .map(|cable| (cable.from, cable.to))
        .collect();
    let registry = shipped();
    let provided: HashMap<_, _> = config
        .places
        .iter()
        .filter(|place| place.borrowed)
        .map(|place| {
            let part = &registry.get(&place.module).unwrap().parts[0];
            assert_eq!(part.kind, "Clock");
            ((place.place.clone(), part.part.clone()), clock)
        })
        .collect();
    assert_eq!(provided.len(), 1);
    let mut ids = (2..).map(ModuleId);
    let plan = registry.plan(
        &RackPatch {
            modules: &modules,
            cables: &cables,
            normalled_output: Some(&config.normalled_output),
            provided: &provided,
        },
        |place, knob| match (place, knob) {
            ("vco_low", "freq") => note_hz("C2"),
            ("vco_high", "freq") => note_hz("C4"),
            ("filter", "cutoff") => note_hz("C5"),
            _ => None,
        },
        || ids.next(),
    );
    let scope = plan.parts[&("scope".into(), "scope".into())];
    assert_eq!(plan.voice, Some(PortRef::new(scope, "out")));
    assert_eq!(plan.owned.len(), 6);

    let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
    engine
        .send(Command::AddModule {
            id: clock,
            kind: ModuleKind::Clock { bpm: 120.0 },
        })
        .unwrap();
    for command in plan.commands {
        engine.send(command).unwrap();
    }
    engine.flush().unwrap();

    let mut output = [0.0_f32; 512];
    let mut trace = [0.0_f32; 128];
    let (mut heard_gate, mut heard_rest) = (false, false);
    for _ in 0..190 {
        renderer.render(&mut output);
        if output.iter().any(|sample| sample.abs() > 0.01)
            && engine.read_scope(scope, &mut trace)
            && trace.iter().any(|sample| sample.abs() > 0.01)
        {
            heard_gate = true;
        }
        if heard_gate && output.iter().all(|sample| sample.abs() < 1e-6) {
            heard_rest = true;
        }
    }
    assert!(
        heard_gate,
        "the outlet and trace should receive the filtered mix"
    );
    assert!(
        heard_rest,
        "the borrowed clock should close the VCA between gates"
    );
}

/// The purifier's patch, rendered offline for two seconds, with its filter-to-VCA cable
/// taken from `filter_out` (`hpf` as shipped). The outlet's left channel.
fn render_purifier(filter_out: &str) -> Vec<f32> {
    use stubby_audio::{Engine, EngineConfig, ModuleKind};

    #[derive(Deserialize)]
    struct Profile {
        configure: Configuration,
    }
    #[derive(Deserialize)]
    struct Configuration {
        places: Vec<Place>,
        cables: Vec<Cable>,
        normalled_output: Jack,
    }
    #[derive(Deserialize)]
    struct Place {
        place: String,
        module: String,
        #[serde(default)]
        borrowed: bool,
    }
    #[derive(Deserialize)]
    struct Cable {
        from: Jack,
        to: Jack,
    }
    let profile: Profile = serde_json::from_str(include_str!(
        "../../../assets/machines/reed_water_purifier/asset.json"
    ))
    .unwrap();
    let config = profile.configure;
    let clock = ModuleId(1);
    let modules = config
        .places
        .iter()
        .map(|place| (place.place.clone(), place.module.clone()))
        .collect();
    let cables: Vec<_> = config
        .cables
        .into_iter()
        .map(|mut cable| {
            if cable.from.place == "filter" {
                cable.from.port = filter_out.into();
            }
            (cable.from, cable.to)
        })
        .collect();
    let registry = shipped();
    let provided: HashMap<_, _> = config
        .places
        .iter()
        .filter(|place| place.borrowed)
        .map(|place| {
            let part = &registry.get(&place.module).unwrap().parts[0];
            ((place.place.clone(), part.part.clone()), clock)
        })
        .collect();
    let mut ids = (2..).map(ModuleId);
    let plan = registry.plan(
        &RackPatch {
            modules: &modules,
            cables: &cables,
            normalled_output: Some(&config.normalled_output),
            provided: &provided,
        },
        |place, knob| match (place, knob) {
            ("vco_low", "freq") => note_hz("C2"),
            ("vco_high", "freq") => note_hz("C4"),
            ("filter", "cutoff") => note_hz("C5"),
            _ => None,
        },
        || ids.next(),
    );
    let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
    engine
        .send(Command::AddModule {
            id: clock,
            kind: ModuleKind::Clock { bpm: 120.0 },
        })
        .unwrap();
    for command in plan.commands {
        engine.send(command).unwrap();
    }
    engine.flush().unwrap();
    let mut left = Vec::new();
    let mut block = [0.0_f32; 512];
    for _ in 0..(2 * 48_000 / 256) {
        renderer.render(&mut block);
        left.extend(block.iter().step_by(2));
    }
    left
}

/// The power of `samples` at `hz` (Goertzel), at 48 kHz.
fn power_at(samples: &[f32], hz: f32) -> f32 {
    let w = std::f32::consts::TAU * hz / 48_000.;
    let coeff = 2. * w.cos();
    let (mut s1, mut s2) = (0.0_f32, 0.0_f32);
    for &x in samples {
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2) / samples.len() as f32
}

/// The purifier's job is the high-pass: with the filter's HPF patched (as shipped) the
/// low saw's C2 fundamental is all but gone and its bright upper harmonics pass, the
/// opposite of the same patch through the LPF. The patch test above only hears sound.
#[test]
fn the_purifier_high_passes_its_mix() {
    let hpf = render_purifier("hpf");
    let lpf = render_purifier("lpf");
    let c2 = note_hz("C2").unwrap();
    let c6 = note_hz("C6").unwrap(); // the saw's 16th harmonic, an octave over the cutoff
    let (hpf_low, lpf_low) = (power_at(&hpf, c2), power_at(&lpf, c2));
    let (hpf_high, lpf_high) = (power_at(&hpf, c6), power_at(&lpf, c6));
    assert!(lpf_low > 0., "the low-pass passes C2");
    assert!(
        hpf_low < lpf_low * 0.01,
        "C2 at least 20 dB down through the HPF: {hpf_low} vs {lpf_low}"
    );
    assert!(
        hpf_high > lpf_high * 4.,
        "C6 passes the HPF better than the LPF: {hpf_high} vs {lpf_high}"
    );
}
