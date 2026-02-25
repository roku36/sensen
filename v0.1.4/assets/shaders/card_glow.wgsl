// Card glow effect shader for Bevy 0.17
// Creates a pulsing glow effect around card edges

#import bevy_pbr::forward_io::VertexOutput

struct CardGlowMaterial {
    glow_color: vec4<f32>,
    glow_intensity: f32,
    time: f32,
    _padding: vec2<f32>,
}

@group(2) @binding(0) var<uniform> material: CardGlowMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Calculate distance from edge for glow effect
    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));

    // Pulsing effect using time
    let pulse = sin(material.time * 3.0) * 0.5 + 0.5;

    // Glow intensity based on edge distance
    let glow_falloff = 0.15;
    let glow = smoothstep(0.0, glow_falloff, edge_dist);
    let edge_glow = (1.0 - glow) * material.glow_intensity * (0.7 + 0.3 * pulse);

    // Final color with glow
    var final_color = material.glow_color * edge_glow;
    final_color.a = edge_glow * material.glow_color.a;

    return final_color;
}
