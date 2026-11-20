// The air quality card's gauge (stubby_game::hud): a half ring over the top, a grey
// track, filled from the left as far as the value in the value's band colour.
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Gauge {
    // x: how full (0..1), y: ring thickness in pixels
    params: vec4<f32>,
    track: vec4<f32>,
    fill: vec4<f32>,
}

@group(1) @binding(0) var<uniform> gauge: Gauge;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size;
    let p = in.uv * size;
    let thickness = gauge.params.y;
    let centre = vec2<f32>(size.x * 0.5, size.y - thickness * 0.5 - 2.0);
    let radius = min(size.x * 0.5, size.y) - thickness * 0.5 - 2.0;
    let v = vec2<f32>(p.x - centre.x, centre.y - p.y);
    let half_t = thickness * 0.5;
    // The ring's ends are rounded: measure to the arc's centre line, capped at its ends.
    var angle = atan2(v.y, v.x);
    if v.y < 0.0 {
        angle = select(0.0, 3.14159265, v.x < 0.0);
    }
    let on_line = centre + radius * vec2<f32>(cos(angle), -sin(angle));
    let d = distance(p, on_line);
    let alpha = 1.0 - smoothstep(half_t - 0.8, half_t + 0.6, d);
    let filled = angle >= 3.14159265 * (1.0 - clamp(gauge.params.x, 0.0, 1.0));
    let colour = select(gauge.track, gauge.fill, filled);
    return vec4<f32>(colour.rgb, colour.a * alpha);
}
