#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BiomeAtmosphere {
    // 0.0 = city (bright, no vignette), 1.0 = darkwood (dark, heavy vignette)
    darkness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(2) var<uniform> settings: BiomeAtmosphere;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    // -- Scene darkening --
    // Darken by up to 65% in full darkwood.
    let darken = 1.0 - settings.darkness * 0.65;

    // -- Vignette --
    // Distance from screen centre (0..~0.7).
    let center = in.uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);
    // Vignette strength scales with darkness: 0 in city, strong in darkwood.
    let vignette_radius = 0.3;
    let vignette_soft = 0.4;
    let raw_vignette = smoothstep(vignette_radius, vignette_radius + vignette_soft, dist);
    let vignette = 1.0 - raw_vignette * settings.darkness * 0.9;

    let tint = darken * vignette;
    return vec4<f32>(color.rgb * tint, color.a);
}
