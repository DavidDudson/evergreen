use bevy::prelude::*;
use level::area::{MAP_HEIGHT, MAP_WIDTH};
use level::spawning::{area_world_offset, TILE_SIZE_PX};
use level::world::WorldMap;
use models::camera_follow::CameraFollow;

use crate::config::CameraConfig;

/// Squared distance below which offsets snap to zero.
const SNAP_THRESHOLD_SQ: f32 = 0.25;

/// Half of the tile-to-pixel divisor used to derive the area half-extents.
const HALF_DIVISOR: f32 = 2.0;

/// Residual offset for the dialogue camera return.
#[derive(Resource, Default)]
pub struct CameraOffset {
    /// Time-based residual for the dialogue camera return.
    pub dialogue_return: Vec2,
}

/// Camera blends between the active area centre and the follow target.
///
/// When the target is near the centre of the area the camera sits on the
/// area centre.  As they approach an edge the camera shifts toward the
/// target, reaching full tracking at the boundary.  This creates a
/// "tug of war" feel: area-locked in the middle, target-locked at the edges.
pub fn follow_player(
    mut state: ResMut<CameraOffset>,
    target_q: Query<&Transform, With<CameraFollow>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<CameraFollow>)>,
    world: Res<WorldMap>,
    time: Res<Time>,
    config: Res<CameraConfig>,
) {
    let Ok(target_tf) = target_q.single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };

    let half_w = f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX) / HALF_DIVISOR;
    let half_h = f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX) / HALF_DIVISOR;

    let target_pos = target_tf.translation.truncate();
    let area_center = area_world_offset(world.current);

    // How far the target is from the area centre, normalised per axis.
    // The camera only starts moving once past `dead_zone_frac` of the
    // half-extent, so the centre band is fully area-locked.
    let local = target_pos - area_center;
    let dead_zone = config.dead_zone_frac;
    let tx = ((local.x.abs() / half_w - dead_zone) / (1.0 - dead_zone)).clamp(0.0, 1.0);
    let ty = ((local.y.abs() / half_h - dead_zone) / (1.0 - dead_zone)).clamp(0.0, 1.0);

    // Smoothstep for a gentle start and strong pull near edges.
    let wx = smoothstep(tx);
    let wy = smoothstep(ty);

    // Blend: area centre (w=0) -> target position (w=1).
    let target_x = area_center.x + (target_pos.x - area_center.x) * wx;
    let target_y = area_center.y + (target_pos.y - area_center.y) * wy;

    // Dialogue return (time-based decay).
    if state.dialogue_return != Vec2::ZERO {
        let alpha = (config.dialogue_return_speed_per_sec * time.delta_secs()).min(1.0);
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
