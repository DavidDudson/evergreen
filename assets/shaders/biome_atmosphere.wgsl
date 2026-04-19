#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct BiomeAtmosphere {
    // Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark)
    darkness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(2) var<uniform> settings: BiomeAtmosphere;

// 4x4 Bayer ordered-dither matrix, normalized to [0, 1).
fn bayer_4x4(coord: vec2<u32>) -> f32 {
    let i = (coord.y % 4u) * 4u + (coord.x % 4u);
    var m: array<u32, 16> = array<u32, 16>(
        0u,  8u,  2u,  10u,
        12u, 4u,  14u, 6u,
        3u,  11u, 1u,  9u,
        15u, 7u,  13u, 5u,
    );
    return f32(m[i]) / 16.0;
}

const DITHER_STEP: f32 = 1.0 / 255.0;

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

    var rgb = color.rgb * darken * vignette;

    // -- Bayer dither, scaled by darkness so city = no dither, darkwood = full --
    let frag_coord = vec2<u32>(in.position.xy);
    let dither = (bayer_4x4(frag_coord) - 0.5) * DITHER_STEP * settings.darkness;
    rgb = rgb + vec3<f32>(dither, dither, dither);

    return vec4<f32>(rgb, color.a);
}
