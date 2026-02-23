// Damage flash effect shader for Bevy 0.17
// Creates a red flash overlay when taking damage

// Define our own VertexOutput to avoid PBR pipeline conflicts
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct DamageFlashMaterial {
    flash_color: vec4<f32>,
    flash_intensity: f32,
    time: f32,
    _padding: vec2<f32>,
}

@group(2) @binding(0) var<uniform> material: DamageFlashMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Flash decays over time exponentially
    let decay = exp(-material.time * 5.0);
    let intensity = material.flash_intensity * decay;

    // Apply flash color with decay
    var color = material.flash_color;
    color.a = intensity * material.flash_color.a;

    return color;
}
