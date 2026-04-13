use bevy::prelude::*;
use level::world::WorldMap;

use crate::atmosphere::BiomeAtmosphere;

/// Lerp speed for atmosphere transitions between areas.
const ATMOSPHERE_LERP_SPEED: f32 = 3.0;

/// Update the atmosphere darkness based on the current area's alignment.
pub fn sync_atmosphere(
    world: Res<WorldMap>,
    time: Res<Time>,
    mut query: Query<&mut BiomeAtmosphere>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);

    // Map alignment 1-100 to darkness 0.0-1.0.
    let target = f32::from(alignment.clamp(1, 100) - 1) / 99.0;

    for mut atmo in &mut query {
        let alpha = (ATMOSPHERE_LERP_SPEED * time.delta_secs()).min(1.0);
        atmo.darkness += (target - atmo.darkness) * alpha;
    }
}
