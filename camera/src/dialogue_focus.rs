use bevy::prelude::*;
use dialog::runner::DialogueTarget;
use player::Player;

/// Lerp speed for camera movement toward the dialogue midpoint.
const CAMERA_LERP_SPEED: f32 = 5.0;

/// Centers the camera between the player and the NPC they are talking to.
pub fn focus_on_dialogue(
    target: Res<DialogueTarget>,
    player_q: Query<&GlobalTransform, With<Player>>,
    npc_q: Query<&GlobalTransform, Without<Player>>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let Some(npc_entity) = target.0 else {
        return;
    };
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(npc_tf) = npc_q.get(npc_entity) else {
        return;
    };
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };

    let player_pos = player_tf.translation().truncate();
    let npc_pos = npc_tf.translation().truncate();
    let midpoint = (player_pos + npc_pos) / 2.0;

    let alpha = (CAMERA_LERP_SPEED * time.delta_secs()).min(1.0);
    cam_tf.translation.x += (midpoint.x - cam_tf.translation.x) * alpha;
    cam_tf.translation.y += (midpoint.y - cam_tf.translation.y) * alpha;
}

/// Stores the current camera position so the smooth-return system can
/// lerp it back to the origin over time.
pub fn reset_camera(
    camera_q: Query<&Transform, With<Camera2d>>,
    mut offset: ResMut<crate::smooth::CameraOffset>,
) {
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    offset.dialogue_return = cam_tf.translation.truncate();
}
