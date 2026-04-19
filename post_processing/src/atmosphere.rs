use bevy::core_pipeline::core_2d::graph::Node2d;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

/// Combined post-processing effect for biome atmosphere.
///
/// Biome darkness fades the scene and adds a vignette based on area alignment.
/// Time-of-day lighting is handled by `lighting::ambient::sync_ambient_light`
/// against `Light2d.ambient_light`, not this shader.
///
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct BiomeAtmosphere {
    /// Biome darkness: 0.0 = city (bright), 1.0 = darkwood (dark + vignette).
    pub darkness: f32,
    // Padding to reach 16-byte alignment (required by WebGL).
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Default for BiomeAtmosphere {
    fn default() -> Self {
        Self {
            darkness: 0.0,
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
