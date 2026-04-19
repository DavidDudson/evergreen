use bevy::prelude::*;
use bevy_light_2d::prelude::PointLight2d;
use level::exit::LevelExit;
use models::palette::LIGHT_EXIT;

/// Intensity of the level-exit point light (HDR scale).
const LIGHT_EXIT_INTENSITY: f32 = 4.0;
/// Radius of the level-exit point light, in world pixels.
const LIGHT_EXIT_RADIUS_PX: f32 = 96.0;
/// Falloff curve exponent (1.0 = linear).
const LIGHT_EXIT_FALLOFF: f32 = 1.0;

/// Insert a `PointLight2d` on every `LevelExit` entity that does not yet have one.
pub fn attach_level_exit_light(
    mut commands: Commands,
    query: Query<Entity, (With<LevelExit>, Without<PointLight2d>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(PointLight2d {
            color: LIGHT_EXIT,
            intensity: LIGHT_EXIT_INTENSITY,
            radius: LIGHT_EXIT_RADIUS_PX,
            falloff: LIGHT_EXIT_FALLOFF,
            ..default()
        });
    }
}
