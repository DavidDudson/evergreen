use bevy::prelude::*;
use dialog::runner::DialogueTarget;
use models::camera_follow::CameraFollow;

use crate::config::CameraConfig;
use crate::smooth::CameraOffset;

/// Centres the camera between the follow target and the NPC they are talking to.
pub fn focus_on_dialogue(
    target: Res<DialogueTarget>,
    follow_q: Query<&GlobalTransform, With<CameraFollow>>,
    npc_q: Query<&GlobalTransform, Without<CameraFollow>>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    config: Res<CameraConfig>,
) {
    let Some(npc_entity) = target.0 else {
        return;
    };
    let Ok(follow_tf) = follow_q.single() else {
        return;
    };
    let Ok(npc_tf) = npc_q.get(npc_entity) else {
        return;
    };
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };

    let follow_pos = follow_tf.translation().truncate();
    let npc_pos = npc_tf.translation().truncate();
    let midpoint = (follow_pos + npc_pos) / 2.0;

    let alpha = (config.lerp_speed_per_sec * time.delta_secs()).min(1.0);
    cam_tf.translation.x += (midpoint.x - cam_tf.translation.x) * alpha;
    cam_tf.translation.y += (midpoint.y - cam_tf.translation.y) * alpha;
}

/// Stores the camera's offset from the follow target so the smooth-return
/// system can lerp it back to zero over time.
pub fn reset_camera(
    camera_q: Query<&Transform, With<Camera2d>>,
    follow_q: Query<&Transform, (With<CameraFollow>, Without<Camera2d>)>,
    mut offset: ResMut<CameraOffset>,
) {
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let Ok(follow_tf) = follow_q.single() else {
        return;
    };
    offset.dialogue_return = cam_tf.translation.truncate() - follow_tf.translation.truncate();
}
