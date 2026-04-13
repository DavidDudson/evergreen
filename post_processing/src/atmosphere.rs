use bevy::core_pipeline::core_2d::graph::Node2d;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

/// Combined post-processing effect for biome atmosphere and time of day.
///
/// Biome fields darken the scene and add a vignette based on area alignment.
/// Time-of-day fields tint and dim the scene based on the game clock.
/// Both effects are applied in a single shader pass.
///
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct BiomeAtmosphere {
    /// Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark + vignette).
    pub darkness: f32,
    /// Time-of-day brightness (0.3 = night, 1.0 = midday).
    pub tod_brightness: f32,
    /// Time-of-day red tint.
    pub tod_tint_r: f32,
    /// Time-of-day green tint.
    pub tod_tint_g: f32,
    /// Time-of-day blue tint.
    pub tod_tint_b: f32,
    // Padding to reach 32-byte alignment (required by WebGL).
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Default for BiomeAtmosphere {
    fn default() -> Self {
        Self {
            darkness: 0.0,
            tod_brightness: 1.0,
            tod_tint_r: 1.0,
            tod_tint_g: 0.98,
            tod_tint_b: 0.95,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

impl FullscreenMaterial for BiomeAtmosphere {
    fn fragment_shader() -> ShaderRef {
        "shaders/biome_atmosphere.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }
}
