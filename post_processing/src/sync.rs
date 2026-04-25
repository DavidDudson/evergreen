use bevy::prelude::*;
use level::world::WorldMap;

use crate::atmosphere::BiomeAtmosphere;
use crate::lerp_config::PostProcessLerpConfig;
use crate::toggles::DisableAtmosphere;

/// Fallback alignment when the current area is missing.
const DEFAULT_AREA_ALIGNMENT: u8 = 50;
/// Maximum darkness denominator: alignment range is 1..=100, so 99 steps map to 0..=1.
const DARKNESS_RANGE_STEPS: f32 = 99.0;

/// Update the atmosphere darkness based on the current area's alignment.
pub fn sync_atmosphere(
    world: Res<WorldMap>,
    time: Res<Time>,
    lerp_config: Res<PostProcessLerpConfig>,
    mut query: Query<&mut BiomeAtmosphere, Without<DisableAtmosphere>>,
) {
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment);

    // Map alignment 1-100 to darkness 0.0-1.0.
    let target = f32::from(alignment.clamp(1, 100) - 1) / DARKNESS_RANGE_STEPS;

    for mut atmo in &mut query {
        let alpha = (lerp_config.atmosphere_speed_per_sec * time.delta_secs()).min(1.0);
        atmo.darkness += (target - atmo.darkness) * alpha;
    }
}
