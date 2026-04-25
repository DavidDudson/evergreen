use bevy::prelude::*;
use bevy_light_2d::prelude::PointLight2d;
use level::area::AreaAlignment;
use level::world::WorldMap;
use models::palette::LIGHT_TORCH;
use models::player::Player;
use models::time::GameClock;

/// Alignment threshold above which the torch turns on regardless of hour.
const DARKWOOD_TORCH_THRESHOLD: AreaAlignment = 75;
/// Hour-of-day before which the torch is on (early morning / pre-dawn).
const TORCH_HOUR_START: f32 = 5.0;
/// Hour-of-day after which the torch is on (post-dusk).
const TORCH_HOUR_END: f32 = 19.0;

/// Fallback alignment when the current area is missing (greenwood -- safe neutral).
const DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50;

/// Tunable torch parameters. Replace via `app.insert_resource(TorchConfig { ... })`
/// to override defaults from a plugin or scene.
#[derive(Resource, Debug, Clone)]
pub struct TorchConfig {
    pub color: Color,
    pub intensity: f32,
    pub radius_px: f32,
    pub falloff: f32,
    pub cast_shadows: bool,
    pub darkwood_alignment_threshold: AreaAlignment,
    pub off_hour_start: f32,
    pub off_hour_end: f32,
}

impl Default for TorchConfig {
    fn default() -> Self {
        Self {
            color: LIGHT_TORCH,
            intensity: 2.5,
            radius_px: 80.0,
            falloff: 1.0,
            cast_shadows: true,
            darkwood_alignment_threshold: DARKWOOD_TORCH_THRESHOLD,
            off_hour_start: TORCH_HOUR_START,
            off_hour_end: TORCH_HOUR_END,
        }
    }
}

impl TorchConfig {
    pub fn point_light(&self) -> PointLight2d {
        PointLight2d {
            color: self.color,
            intensity: self.intensity,
            radius: self.radius_px,
            falloff: self.falloff,
            cast_shadows: self.cast_shadows,
        }
    }

    pub fn should_be_on(&self, alignment: AreaAlignment, hour: f32) -> bool {
        alignment > self.darkwood_alignment_threshold
            || !(self.off_hour_start..=self.off_hour_end).contains(&hour)
    }
}

/// Pure predicate exposed for tests / external policy callers using default config.
pub fn should_torch_be_on(alignment: AreaAlignment, hour: f32) -> bool {
    TorchConfig::default().should_be_on(alignment, hour)
}

/// Per-frame system: insert/remove the torch on the player based on
/// `TorchConfig::should_be_on(area.alignment, clock.hour)`.
pub fn update_player_torch(
    mut commands: Commands,
    config: Res<TorchConfig>,
    world: Res<WorldMap>,
    clock: Res<GameClock>,
    query: Query<(Entity, Option<&PointLight2d>), With<Player>>,
) {
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment);
    let on = config.should_be_on(alignment, clock.hour);

    for (entity, existing) in &query {
        match (on, existing.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(config.point_light());
            }
            (false, true) => {
                commands.entity(entity).remove::<PointLight2d>();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torch_off_in_daylight_city() {
        assert!(!should_torch_be_on(10, 12.0));
    }

    #[test]
    fn torch_on_at_night_anywhere() {
        assert!(should_torch_be_on(50, 22.0));
    }

    #[test]
    fn torch_on_in_darkwood_anytime() {
        assert!(should_torch_be_on(90, 12.0));
    }

    #[test]
    fn torch_off_at_dawn_greenwood() {
        assert!(!should_torch_be_on(50, 8.0));
    }

    #[test]
    fn torch_threshold_boundary_strict() {
        // alignment == threshold => false (strict greater-than)
        assert!(!should_torch_be_on(75, 12.0));
    }

    #[test]
    fn torch_threshold_above_boundary() {
        assert!(should_torch_be_on(76, 12.0));
    }
}
