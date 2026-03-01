use bevy::prelude::*;
use models::scenery::SceneryCollider;

use crate::spawning::Player;

/// Half-size of the player's AABB for collision (pixels).
const PLAYER_HALF_PX: f32 = 6.0;

/// Push the player out of any overlapping scenery colliders (AABB push-back).
///
/// Runs after `move_player` so the player's new position is already applied.
pub fn resolve_scenery_collisions(
    mut player_q: Query<&mut Transform, With<Player>>,
    colliders: Query<(&Transform, &SceneryCollider), Without<Player>>,
) {
    let Ok(mut player_tf) = player_q.single_mut() else {
        return;
    };

    let ph = Vec2::splat(PLAYER_HALF_PX);

    for (obj_tf, col) in &colliders {
        let pp = player_tf.translation.truncate();
        let cp = obj_tf.translation.truncate() + col.center_offset;

        let overlap_x = ph.x + col.half_extents.x - (pp.x - cp.x).abs();
        let overlap_y = ph.y + col.half_extents.y - (pp.y - cp.y).abs();

        if overlap_x <= 0.0 || overlap_y <= 0.0 {
            continue;
        }

        // Resolve on the axis of minimum penetration.
        if overlap_x < overlap_y {
            player_tf.translation.x += overlap_x * (pp.x - cp.x).signum();
        } else {
            player_tf.translation.y += overlap_y * (pp.y - cp.y).signum();
        }
    }
}
