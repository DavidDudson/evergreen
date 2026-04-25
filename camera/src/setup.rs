use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, Hdr};
use bevy_light_2d::prelude::{AmbientLight2d, Light2d};
use level::plugin::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE_PX};
use post_processing::atmosphere::BiomeAtmosphere;
use post_processing::bloom_setup::pixel_art_bloom;

/// Spawns the primary 2D camera with HDR + bloom + ambient light pipeline.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        // HDR + MSAA is unsupported on WebGL2 and crashes at runtime.
        // Pixel art also gains nothing from MSAA.
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        pixel_art_bloom(),
        BiomeAtmosphere::default(),
        ColorGrading::default(),
        Light2d {
            ambient_light: AmbientLight2d {
                color: Color::WHITE,
                brightness: 1.0,
            },
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
                min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
