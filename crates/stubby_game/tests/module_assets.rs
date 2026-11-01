//! Exported module assets carry the nodes and extras the game relies on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn model(name: &str) -> gltf::Gltf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/models/modules")
        .join(name);
    gltf::Gltf::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn extras(node: &gltf::Node) -> serde_json::Value {
    node.extras()
        .as_ref()
        .map(|raw| serde_json::from_str(raw.get()).unwrap())
        .unwrap_or(serde_json::Value::Null)
}

#[test]
fn vco_modules_expose_ports_knob_and_light() {
    for (waveform, prefix) in [
        ("square", "SquareVco"),
        ("saw", "SawVco"),
        ("triangle", "TriangleVco"),
        ("sine", "SineVco"),
    ] {
        let gltf = model(&format!("{waveform}_vco.glb"));
        let node = |name: &str| {
            gltf.nodes()
                .find(|n| n.name() == Some(name))
                .unwrap_or_else(|| panic!("{waveform}: missing node {name}"))
        };

        let root = extras(&node(&format!("{prefix}_Root")));
        assert_eq!(root["asset_id"], format!("{waveform}_vco"));
        assert_eq!(root["module_kind"], prefix);
        assert_eq!(root["waveform"], waveform);
        for (name, port, direction) in [("Port_voct", "voct", "in"), ("Port_out", "out", "out")] {
            let e = extras(&node(name));
            assert_eq!(e["port"], port, "{waveform} {name}");
            assert_eq!(e["direction"], direction, "{waveform} {name}");
            // The faceplate faces glTF +Z, so sockets sit in front of the body.
            let [_, _, z] = node(name).transform().decomposed().0;
            assert!(
                z > 0.15,
                "{waveform}: {name} at z = {z}, behind the faceplate"
            );
        }
        assert_eq!(extras(&node("Knob_freq"))["param"], "freq");
        assert_eq!(extras(&node("Light_out"))["light"], "out");

        // The LED and the waveform glyphs glow; nothing else does.
        let mut emissive: Vec<_> = gltf
            .materials()
            .filter(|m| m.emissive_factor() != [0.0, 0.0, 0.0])
            .filter_map(|m| m.name().map(str::to_owned))
            .collect();
        emissive.sort();
        assert_eq!(
            emissive,
            [format!("{prefix}_Glyph"), format!("{prefix}_LedOut")],
            "{waveform}"
        );
    }
}

#[test]
fn clock_exposes_outputs_encoder_leds_lamp_and_bpm_display() {
    let gltf = model("clock_lamp.glb");
    let node = |name: &str| {
        gltf.nodes()
            .find(|n| n.name() == Some(name))
            .unwrap_or_else(|| panic!("clock_lamp: missing node {name}"))
    };

    let root = extras(&node("Clock_Root"));
    assert_eq!(root["asset_id"], "clock_lamp");
    assert_eq!(root["module_kind"], "Clock");
    for (name, port) in [("Port_gate", "gate"), ("Port_phase", "phase")] {
        let e = extras(&node(name));
        assert_eq!(e["port"], port);
        assert_eq!(e["direction"], "out", "the engine's Clock has no inputs");
        let [_, _, z] = node(name).transform().decomposed().0;
        assert!(z > 0.15, "{name} at z = {z}, behind the faceplate");
    }
    assert_eq!(extras(&node("Knob_bpm"))["param"], "bpm");
    assert_eq!(extras(&node("Light_gate"))["light"], "gate");
    assert_eq!(extras(&node("Light_phase"))["light"], "phase");

    // The flashing clock light carries the envelope the game drives from ticks.
    let lamp = extras(&node("Lamp"));
    assert_eq!(lamp["light"], "gate");
    assert_eq!(lamp["pulse_driver"], "audio_sample_timestamp");
    assert!(lamp["peak_strength"].as_f64().unwrap() > lamp["rest_strength"].as_f64().unwrap());

    // Three seven-segment digits, every segment addressable, so any BPM can show.
    assert_eq!(extras(&node("Display"))["display"], "bpm");
    for digit in 0..3 {
        for segment in ["a", "b", "c", "d", "e", "f", "g"] {
            let e = extras(&node(&format!("Display_d{digit}_{segment}")));
            assert_eq!(e["digit"], digit);
            assert_eq!(e["segment"], segment);
        }
    }

    let mut emissive: Vec<_> = gltf
        .materials()
        .filter(|m| m.emissive_factor() != [0.0, 0.0, 0.0])
        .filter_map(|m| m.name().map(str::to_owned))
        .collect();
    emissive.sort();
    assert_eq!(emissive, ["Clock_Lamp", "Clock_LedGate", "Clock_Segment"]);
}
