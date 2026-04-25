//! Player hop-bob while standing on a stepping stone. Applies an additive
//! vertical sine offset to the player's Transform without new animation
//! frames -- re-uses existing walk/run sprites. Runs last in the player
//! system chain so it sees the frame's final logical position.

use bevy::prelude::*;
use level::plugin::TILE_SIZE_PX;
use models::player::HopTrigger;

use crate::spawning::Player;

/// Sine frequency for the hop bob (one full hop arc per ~0.36s).
const HOP_FREQ_HZ: f32 = 2.8;
/// Vertical amplitude in pixels.
const HOP_AMPLITUDE_PX: f32 = 3.5;
/// Half a tile, for stone-proximity detection.
#[allow(clippy::as_conversions)] // u16 -> f32 in const context; safe (TILE_SIZE_PX << f32 range).
const TILE_HALF_PX: f32 = TILE_SIZE_PX as f32 * 0.5;

/// Tracks the bob offset currently added to the player's Transform so it can
/// be reversed next frame before applying a new bob.
#[derive(Component, Default)]
pub struct HopBob {
    last_applied: f32,
}

#[allow(clippy::type_complexity)]
pub fn apply_hop_bob(
    time: Res<Time>,
    mut player: Query<(&mut Transform, &mut HopBob), With<Player>>,
    stones: Query<&Transform, (With<HopTrigger>, Without<Player>)>,
) {
    let Ok((mut tf, mut bob)) = player.single_mut() else {
        return;
    };

    // 1. Reverse last frame's bob so we see the true logical Y.
    tf.translation.y -= bob.last_applied;

    // 2. Are we standing on a stone?
    let player_pos = tf.translation.truncate();
    let on_stone = stones.iter().any(|stone_tf| {
        let delta = stone_tf.translation.truncate() - player_pos;
        delta.x.abs() < TILE_HALF_PX && delta.y.abs() < TILE_HALF_PX
    });

    // 3. Compute new bob.
    let new_bob = if on_stone {
        (time.elapsed_secs() * HOP_FREQ_HZ * std::f32::consts::TAU).sin().abs() * HOP_AMPLITUDE_PX
    } else {
        0.0
    };

    // 4. Apply and remember.
    tf.translation.y += new_bob;
    bob.last_applied = new_bob;
}
