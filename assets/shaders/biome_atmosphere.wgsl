#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BiomeAtmosphere {
    // Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark)
    darkness: f32,
    // Time-of-day brightness multiplier (0.3 = night, 1.0 = midday)
    tod_brightness: f32,
    // Time-of-day RGB tint
    tod_tint_r: f32,
    tod_tint_g: f32,
    tod_tint_b: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(2) var<uniform> settings: BiomeAtmosphere;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    // -- Biome darkening --
    let darken = 1.0 - settings.darkness * 0.80;

    // -- Vignette --
    let center = in.uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);
    let vignette_radius = 0.15;
    let vignette_soft = 0.35;
    let raw_vignette = smoothstep(vignette_radius, vignette_radius + vignette_soft, dist);
    let vignette = 1.0 - raw_vignette * settings.darkness;

    let biome_tint = darken * vignette;

    // -- Time of day --
    let tod_tint = vec3<f32>(settings.tod_tint_r, settings.tod_tint_g, settings.tod_tint_b) * settings.tod_brightness;

    // Combine both effects
    return vec4<f32>(color.rgb * biome_tint * tod_tint, color.a);
}
