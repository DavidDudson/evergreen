use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, Hdr};
use level::plugin::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE_PX};
use models::game_states::GameState;

use post_processing::atmosphere::BiomeAtmosphere;
use post_processing::bloom_setup::pixel_art_bloom;

use crate::dialogue_focus;
use crate::smooth;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<smooth::CameraOffset>();

        app.add_systems(Startup, setup);

        app.add_systems(
            Update,
            dialogue_focus::focus_on_dialogue.run_if(in_state(GameState::Dialogue)),
        );

        app.add_systems(OnExit(GameState::Dialogue), dialogue_focus::reset_camera);

        app.add_systems(
            PostUpdate,
            smooth::follow_player.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup(mut commands: Commands) {
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
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
                min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
