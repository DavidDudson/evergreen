use bevy::prelude::*;
use level::area::{MAP_HEIGHT, MAP_WIDTH};
use level::spawning::{area_world_offset, TILE_SIZE_PX};
use level::world::WorldMap;
use player::Player;

/// Time-based lerp speed for the dialogue-return camera slide.
const DIALOGUE_RETURN_SPEED: f32 = 5.0;

/// Squared distance below which offsets snap to zero.
const SNAP_THRESHOLD_SQ: f32 = 0.25;

// Area half-extents in pixels.
#[allow(clippy::as_conversions)]
const HALF_W: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32 / 2.0;
#[allow(clippy::as_conversions)]
const HALF_H: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32 / 2.0;

/// Residual offset for the dialogue camera return.
#[derive(Resource, Default)]
pub struct CameraOffset {
    /// Time-based residual for the dialogue camera return.
    pub dialogue_return: Vec2,
}

/// Camera blends between the active area centre and the player position.
///
/// When the player is near the centre of the area the camera sits on the
/// area centre.  As they approach an edge the camera shifts toward the
/// player, reaching full player-tracking at the boundary.  This creates a
/// "tug of war" feel: area-locked in the middle, player-locked at the edges.
pub fn follow_player(
    mut state: ResMut<CameraOffset>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    world: Res<WorldMap>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };

    let player_pos = player_tf.translation.truncate();
    let area_center = area_world_offset(world.current);

    // How far the player is from the area centre, normalised per axis.
    // The camera only starts moving once past 60% of the half-extent,
    // so the centre 60% of the area is fully area-locked.
    let local = player_pos - area_center;
    let dead_zone = 0.6;
    let tx = ((local.x.abs() / HALF_W - dead_zone) / (1.0 - dead_zone)).clamp(0.0, 1.0);
    let ty = ((local.y.abs() / HALF_H - dead_zone) / (1.0 - dead_zone)).clamp(0.0, 1.0);

    // Smoothstep for a gentle start and strong pull near edges.
    let wx = smoothstep(tx);
    let wy = smoothstep(ty);

    // Blend: area centre (w=0) → player position (w=1).
    let target_x = area_center.x + (player_pos.x - area_center.x) * wx;
    let target_y = area_center.y + (player_pos.y - area_center.y) * wy;

    // Dialogue return (time-based decay).
    if state.dialogue_return != Vec2::ZERO {
        let alpha = (DIALOGUE_RETURN_SPEED * time.delta_secs()).min(1.0);
        state.dialogue_return = state.dialogue_return.lerp(Vec2::ZERO, alpha);
        if state.dialogue_return.length_squared() < SNAP_THRESHOLD_SQ {
            state.dialogue_return = Vec2::ZERO;
        }
    }

    cam_tf.translation.x = target_x + state.dialogue_return.x;
    cam_tf.translation.y = target_y + state.dialogue_return.y;
}

/// Classic smoothstep: 3t^2 - 2t^3.  Zero derivative at 0 and 1.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
