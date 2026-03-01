use bevy::camera::ScalingMode;
use bevy::prelude::*;

/// Half the 1280 × 720 render resolution: 2 Bevy pixels per world unit, so
/// a 1 × 2 tile character (16 × 32 world units) occupies 32 × 64 Bevy pixels.
const VIEWPORT_WIDTH_WORLD: f32 = 640.0;
const VIEWPORT_HEIGHT_WORLD: f32 = 360.0;

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
                min_width: VIEWPORT_WIDTH_WORLD,
                min_height: VIEWPORT_HEIGHT_WORLD,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
