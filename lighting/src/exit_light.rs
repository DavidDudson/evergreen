use bevy::prelude::*;
use bevy_light_2d::prelude::PointLight2d;
use level::exit::LevelExit;
use models::palette::LIGHT_EXIT;

/// Tunable exit-light parameters. Replace via `app.insert_resource(ExitLightConfig { ... })`
/// to override defaults from a plugin or scene.
#[derive(Resource, Debug, Clone)]
pub struct ExitLightConfig {
    pub color: Color,
    pub intensity: f32,
    pub radius_px: f32,
    pub falloff: f32,
    pub cast_shadows: bool,
}

impl Default for ExitLightConfig {
    fn default() -> Self {
        Self {
            color: LIGHT_EXIT,
            intensity: 4.0,
            radius_px: 96.0,
            falloff: 1.0,
            cast_shadows: true,
        }
    }
}

impl ExitLightConfig {
    pub fn point_light(&self) -> PointLight2d {
        PointLight2d {
            color: self.color,
            intensity: self.intensity,
            radius: self.radius_px,
            falloff: self.falloff,
            cast_shadows: self.cast_shadows,
        }
    }
}

/// Insert a `PointLight2d` on every `LevelExit` entity that does not yet have one.
pub fn attach_level_exit_light(
    mut commands: Commands,
    config: Res<ExitLightConfig>,
    query: Query<Entity, (With<LevelExit>, Without<PointLight2d>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(config.point_light());
    }
}
