use bevy::core_pipeline::core_2d::graph::Node2d;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

/// Post-processing effect that darkens the scene and adds a vignette based on
/// area alignment (0 = city/bright, 1 = darkwood/dark).
///
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, Default, ShaderType)]
pub struct BiomeAtmosphere {
    /// 0.0 = no effect (city), 1.0 = full darkwood darkness + vignette.
    pub darkness: f32,
    // Padding to reach 16-byte alignment (required by WebGL).
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
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
