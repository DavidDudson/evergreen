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

/// Player torch intensity (HDR scale).
const LIGHT_TORCH_INTENSITY: f32 = 2.5;
/// Player torch radius (world pixels).
const LIGHT_TORCH_RADIUS_PX: f32 = 80.0;
/// Player torch falloff.
const LIGHT_TORCH_FALLOFF: f32 = 1.0;
/// Torch casts shadows on the multi-rect occluders added in later tasks.
const LIGHT_TORCH_CAST_SHADOWS: bool = true;

/// Fallback alignment when the current area is missing (greenwood -- safe neutral).
const DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50;

/// Pure predicate: should the torch be on for this alignment + hour?
pub fn should_torch_be_on(alignment: AreaAlignment, hour: f32) -> bool {
    alignment > DARKWOOD_TORCH_THRESHOLD || !(TORCH_HOUR_START..=TORCH_HOUR_END).contains(&hour)
}

fn torch_component() -> PointLight2d {
    PointLight2d {
        color: LIGHT_TORCH,
        intensity: LIGHT_TORCH_INTENSITY,
        radius: LIGHT_TORCH_RADIUS_PX,
        falloff: LIGHT_TORCH_FALLOFF,
        cast_shadows: LIGHT_TORCH_CAST_SHADOWS,
    }
}

/// Per-frame system: insert/remove the torch on the player based on
/// `should_torch_be_on(area.alignment, clock.hour)`.
pub fn update_player_torch(
    mut commands: Commands,
    world: Res<WorldMap>,
    clock: Res<GameClock>,
    query: Query<(Entity, Option<&PointLight2d>), With<Player>>,
) {
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment);
    let on = should_torch_be_on(alignment, clock.hour);

    for (entity, existing) in &query {
        match (on, existing.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(torch_component());
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
