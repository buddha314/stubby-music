// The forensic flashlight's hologram (add-forensic-flashlight): a cyan-to-violet rim
// (Fresnel), scanlines scrolling up in world space and a slight flicker, added to the
// scene so it glows. Drawn without a depth test, through grass and reeds. The scene's
// distance fog dims it as it dims everything else (user, 2026-11-20), so a far pickup
// fades into the haze.
import bevy_pbr::render::{
    mesh_view_bindings::{globals, view, fog},
    mesh_view_types,
    fog::{linear_fog, exponential_fog, exponential_squared_fog, atmospheric_fog},
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

// The scene's fog, as Bevy applies it to lit surfaces (without in-scattering).
fn fogged(color: vec4<f32>, distance: f32) -> vec4<f32> {
    let none = vec3<f32>(0.0);
    if fog.mode == mesh_view_types::FOG_MODE_LINEAR {
        return linear_fog(fog, color, distance, none);
    } else if fog.mode == mesh_view_types::FOG_MODE_EXPONENTIAL {
        return exponential_fog(fog, color, distance, none);
    } else if fog.mode == mesh_view_types::FOG_MODE_EXPONENTIAL_SQUARED {
        return exponential_squared_fog(fog, color, distance, none);
    } else if fog.mode == mesh_view_types::FOG_MODE_ATMOSPHERIC {
        return atmospheric_fog(fog, color, distance, none);
    }
    return color;
}

// How much of a light at this distance survives the fog: fogging white and black
// differ by exactly that, whatever the fog's colour. An additive glow must fade toward
// nothing, not toward the fog's colour, or the haze itself would glow.
fn fog_visibility(distance: f32) -> vec3<f32> {
    let white = fogged(vec4<f32>(1.0), distance).rgb;
    let black = fogged(vec4<f32>(0.0, 0.0, 0.0, 1.0), distance).rgb;
    return clamp(white - black, vec3<f32>(0.0), vec3<f32>(1.0));
}

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
    let seen = fog_visibility(length(view.world_position - in.world_position.xyz));
    // Additive: the colour is added in proportion to how strongly it shows.
    return vec4<f32>(color * alpha * seen, alpha * seen.g);
}
