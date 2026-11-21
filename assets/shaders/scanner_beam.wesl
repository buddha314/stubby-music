// The scanner's beams scattering in the haze (stubby_player::flashlight): a thin open cone
// along each beam, glowing in its colour. The glow is brightest at the ball and fades along
// the beam and toward its edges, and its strength follows the fog's density, set each
// frame: thick, bleak air shows the shafts strongly, clean air barely at all. Additive,
// depth-tested, two-sided; dimmed by distance fog as other glows are.
import bevy_pbr::render::{
    mesh_view_bindings::{view, fog},
    mesh_view_types,
    fog::{linear_fog, exponential_fog, exponential_squared_fog, atmospheric_fog},
    forward_io::VertexOutput,
};

struct Beam {
    colour: vec4<f32>,
    // x: strength (the fog's density times the style's gain), y: edge sharpness
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> beam: Beam;

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

fn fog_visibility(distance: f32) -> vec3<f32> {
    let white = fogged(vec4<f32>(1.0), distance).rgb;
    let black = fogged(vec4<f32>(0.0, 0.0, 0.0, 1.0), distance).rgb;
    return clamp(white - black, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // uv.y runs 0 at the ball to 1 at the beam's reach.
    let t = clamp(in.uv.y, 0.0, 1.0);
    let along = smoothstep(0.02, 0.14, t) * pow(1.0 - t, 1.4);
    let to_eye = normalize(view.world_position - in.world_position.xyz);
    let edge = pow(abs(dot(normalize(in.world_normal), to_eye)), beam.params.y);
    let glow = beam.params.x * along * edge;
    let seen = fog_visibility(length(view.world_position - in.world_position.xyz));
    return vec4<f32>(beam.colour.rgb * glow * seen, glow * seen.g);
}
