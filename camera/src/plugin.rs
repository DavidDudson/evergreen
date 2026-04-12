use bevy::camera::ScalingMode;
use bevy::prelude::*;
use level::plugin::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE_PX};
use models::game_states::GameState;

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

        // PostUpdate so the offset always sees AreaChanged written in Update.
        app.add_systems(
            PostUpdate,
            smooth::apply_camera_offset.run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX),
                min_height: f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX),
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
