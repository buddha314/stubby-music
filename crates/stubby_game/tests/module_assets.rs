//! Exported module assets carry the nodes and extras the game relies on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn model(name: &str) -> gltf::Gltf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/models")
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
fn square_vco_exposes_ports_knob_and_light() {
    let gltf = model("square_vco.glb");
    let node = |name: &str| {
        gltf.nodes()
            .find(|n| n.name() == Some(name))
            .unwrap_or_else(|| panic!("missing node {name}"))
    };

    let root = extras(&node("SquareVco_Root"));
    assert_eq!(root["asset_id"], "square_vco");
    assert_eq!(root["module_kind"], "SquareVco");
    for (name, port, direction) in [("Port_voct", "voct", "in"), ("Port_out", "out", "out")] {
        let e = extras(&node(name));
        assert_eq!(e["port"], port);
        assert_eq!(e["direction"], direction);
        // The faceplate faces glTF +Z, so sockets sit in front of the body.
        let [_, _, z] = node(name).transform().decomposed().0;
        assert!(z > 0.15, "{name} at z = {z}, behind the faceplate");
    }
    assert_eq!(extras(&node("Knob_freq"))["param"], "freq");
    assert_eq!(extras(&node("Light_out"))["light"], "out");

    // The LED and glyph glow; nothing else does.
    let mut emissive: Vec<_> = gltf
        .materials()
        .filter(|m| m.emissive_factor() != [0.0, 0.0, 0.0])
        .filter_map(|m| m.name().map(str::to_owned))
        .collect();
    emissive.sort();
    assert_eq!(emissive, ["SquareVco_Glyph", "SquareVco_LedOut"]);
}
