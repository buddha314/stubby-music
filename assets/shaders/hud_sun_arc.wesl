// The sky card's arc (stubby_game::hud): the sun's path from sunrise to sunset over a
// ground line, drawn solid where the sun has been and dim ahead, with the sun (or, at
// night, the moon on its own path from dusk to dawn) riding it, and a dot at each end.
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct SunArc {
    // x: how far along (0 at the left end, 1 at the right), y: line width in pixels,
    // z: disc radius in pixels, w: unused
    params: vec4<f32>,
    ink: vec4<f32>,
    dim: vec4<f32>,
    disc: vec4<f32>,
    ground: vec4<f32>,
}

@group(1) @binding(0) var<uniform> arc: SunArc;

fn over(base: vec4<f32>, top: vec4<f32>, a: f32) -> vec4<f32> {
    let alpha = top.a * clamp(a, 0.0, 1.0);
    return vec4<f32>(mix(base.rgb, top.rgb, alpha), base.a + alpha * (1.0 - base.a));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size;
    let p = in.uv * size;
    let pad = 6.0;
    let base = size.y * 0.82;
    let centre = vec2<f32>(size.x * 0.5, base);
    let radii = vec2<f32>(size.x * 0.5 - pad, size.y * 0.66);
    let progress = clamp(arc.params.x, 0.0, 1.0);
    let line = arc.params.y;
    var colour = vec4<f32>(0.0);

    // The ground: a faint band under the line the path rises from.
    if p.y > base {
        colour = over(colour, arc.ground, 1.0);
    }
    // The path: an upper half ellipse. Distance by its normalised radius.
    let q = vec2<f32>((p.x - centre.x) / radii.x, (centre.y - p.y) / radii.y);
    let r = length(q);
    let d = abs(r - 1.0) * min(radii.x, radii.y);
    if q.y >= 0.0 {
        let angle = atan2(q.y, q.x); // pi at the left end, 0 at the right
        let passed = angle >= 3.14159265 * (1.0 - progress);
        let ink = select(arc.dim, arc.ink, passed);
        colour = over(colour, ink, 1.0 - smoothstep(line * 0.5 - 0.6, line * 0.5 + 0.6, d));
    }
    // The ends.
    let left = vec2<f32>(centre.x - radii.x, base);
    let right = vec2<f32>(centre.x + radii.x, base);
    let dot = min(distance(p, left), distance(p, right));
    colour = over(colour, arc.ink, 1.0 - smoothstep(2.0, 3.0, dot));
    // The sun (or moon): a halo and a disc where it has got to.
    let theta = 3.14159265 * (1.0 - progress);
    let at = centre + vec2<f32>(radii.x * cos(theta), -radii.y * sin(theta));
    let from_disc = distance(p, at);
    let radius = arc.params.z;
    colour = over(colour, vec4<f32>(arc.disc.rgb, 0.28), 1.0 - smoothstep(radius, radius * 1.55, from_disc));
    colour = over(colour, arc.disc, 1.0 - smoothstep(radius - 0.8, radius + 0.6, from_disc));
    return colour;
}
