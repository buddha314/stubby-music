// The forensic flashlight's hologram (add-forensic-flashlight): a cyan-to-violet rim
// (Fresnel), scanlines scrolling up in world space and a slight flicker, added to the
// scene so it glows. Drawn without a depth test, through grass and reeds.
import bevy_pbr::render::{
    mesh_view_bindings::{globals, view},
    forward_io::VertexOutput,
};

struct Hologram {
    rim: vec4<f32>,
    core: vec4<f32>,
    // x: how revealed (0..1), y: scanlines per metre, z: scroll metres per second,
    // w: flicker (0..1)
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> hologram: Hologram;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let to_eye = normalize(view.world_position - in.world_position.xyz);
    let rim = pow(1.0 - abs(dot(normal, to_eye)), 2.0);
    let phase = in.world_position.y * hologram.params.y - globals.time * hologram.params.z;
    let lines = smoothstep(0.35, 1.0, 0.5 + 0.5 * sin(phase * 6.2831853));
    let flicker = 1.0 - hologram.params.w * (0.5 + 0.5 * sin(globals.time * 37.0 + in.world_position.x * 3.0));
    let color = mix(hologram.core.rgb, hologram.rim.rgb, rim);
    let alpha = (0.2 + 0.8 * rim) * (0.5 + 0.5 * lines) * flicker * hologram.params.x;
    // Additive: the colour is added in proportion to how strongly it shows.
    return vec4<f32>(color * alpha, alpha);
}
