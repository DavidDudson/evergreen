use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, RenderLabel};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

use crate::atmosphere::BiomeAtmosphere;

/// Post-processing effect that applies time-of-day lighting.
///
/// Multiplies the screen color by `tint * brightness`.
/// Attach to the camera entity alongside `Camera2d`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct TimeOfDayMaterial {
    /// Overall brightness multiplier (0.0 = black, 1.0 = full bright).
    pub brightness: f32,
    /// Red channel tint.
    pub tint_r: f32,
    /// Green channel tint.
    pub tint_g: f32,
    /// Blue channel tint.
    pub tint_b: f32,
}

impl Default for TimeOfDayMaterial {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            tint_r: 1.0,
            tint_g: 0.98,
            tint_b: 0.95,
        }
    }
}

impl FullscreenMaterial for TimeOfDayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/time_of_day.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            BiomeAtmosphere::node_label().intern(),
            Self::node_label().intern(),
            bevy::core_pipeline::core_2d::graph::Node2d::EndMainPassPostProcessing.intern(),
        ]
    }
}
