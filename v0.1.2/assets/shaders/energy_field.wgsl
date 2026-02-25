// Energy field effect shader for Bevy 0.17
// Creates a flowing energy effect for power cards and buffs

// Define our own VertexOutput to avoid PBR pipeline conflicts
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct EnergyFieldMaterial {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    speed: f32,
    time: f32,
    scale: f32,
    intensity: f32,
}

@group(2) @binding(0) var<uniform> material: EnergyFieldMaterial;

// Simple hash function for noise
fn hash(p: vec2<f32>) -> f32 {
    let k = vec2<f32>(0.3183099, 0.3678794);
    var p2 = p * k + k.yx;
    return fract(16.0 * k.x * fract(p2.x * p2.y * (p2.x + p2.y)));
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// Fractal Brownian Motion for more organic noise
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var p2 = p;

    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(p2);
        p2 *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = material.time * material.speed;

    // Create flowing energy pattern
    let p = uv * material.scale;
    let n1 = fbm(p + vec2<f32>(time * 0.5, time * 0.3));
    let n2 = fbm(p * 1.5 - vec2<f32>(time * 0.4, time * 0.6));

    // Combine noise patterns
    let energy = (n1 + n2) * 0.5;

    // Color gradient based on energy
    let color = mix(material.color_a, material.color_b, energy);

    // Edge fade for smooth blending
    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let edge_fade = smoothstep(0.0, 0.2, edge_dist);

    var final_color = color;
    final_color.a = energy * material.intensity * edge_fade;

    return final_color;
}
