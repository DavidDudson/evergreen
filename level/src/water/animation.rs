//! Per-frame water surface alpha pulse + the marker that opts a sprite in.

use bevy::prelude::*;

/// Marker added to every water sprite. A shared animation system alpha-pulses
/// all water in lockstep so wang-tiled edges stay seamless.
#[derive(Component)]
pub struct AnimatedWater;

/// Surface alpha pulse frequency.
const FREQ_HZ: f32 = 0.55;
/// Surface alpha pulse amplitude.
const ALPHA_AMPLITUDE: f32 = 0.06;

/// Per-frame system: pulse every water sprite's alpha in lockstep so wang-tiled
/// water bodies keep seamless edges. No per-tile phase (would reveal seams)
/// and no scale animation (would gap between neighbours).
pub fn animate_water_surface(time: Res<Time>, mut query: Query<&mut Sprite, With<AnimatedWater>>) {
    let s = (time.elapsed_secs() * FREQ_HZ * std::f32::consts::TAU).sin();
    let alpha = 1.0 - ALPHA_AMPLITUDE + s.abs() * ALPHA_AMPLITUDE;
    for mut sprite in &mut query {
        sprite.color = sprite.color.with_alpha(alpha);
    }
}
