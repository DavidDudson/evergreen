#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct TimeOfDay {
    brightness: f32,
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
}

@group(0) @binding(2) var<uniform> settings: TimeOfDay;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);
    let tint = vec3<f32>(settings.tint_r, settings.tint_g, settings.tint_b) * settings.brightness;
    return vec4<f32>(color.rgb * tint, color.a);
}
