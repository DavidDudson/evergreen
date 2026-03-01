use bevy::camera::ScalingMode;
use bevy::prelude::*;
use level::plugin::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: f32::from(MAP_WIDTH) * TILE_SIZE,
                min_height: f32::from(MAP_HEIGHT) * TILE_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
