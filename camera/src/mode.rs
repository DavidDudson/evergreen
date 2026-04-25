use bevy::prelude::*;
use models::camera_follow::CameraFollow;

use crate::dialogue_focus;
use crate::smooth::CameraOffset;

/// High-level camera behaviour selector.
///
/// Driven by `GameState` transitions: gameplay sits in [`CameraMode::Follow`],
/// dialogue switches to [`CameraMode::DialogueFocus`].  Systems dispatch off
/// the active mode rather than `GameState` directly so callers can stage
/// custom modes without touching engine state.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CameraMode {
    /// Smoothly follow the [`models::camera_follow::CameraFollow`] target.
    #[default]
    Follow,
    /// Centre the camera on the midpoint of the follow target and dialogue
    /// partner.
    DialogueFocus,
}

/// Switches the camera into [`CameraMode::DialogueFocus`].  Wired to
/// `OnEnter(GameState::Dialogue)` by [`crate::plugin::CameraPlugin`].
pub fn enter_dialogue_focus(mut mode: ResMut<CameraMode>) {
    *mode = CameraMode::DialogueFocus;
}

/// Restores [`CameraMode::Follow`] and seeds the smooth-return offset so the
/// camera lerps back to the follow target after dialogue.
pub fn exit_dialogue_focus(
    mut mode: ResMut<CameraMode>,
    camera_q: Query<&Transform, With<Camera2d>>,
    follow_q: Query<&Transform, (With<CameraFollow>, Without<Camera2d>)>,
    offset: ResMut<CameraOffset>,
) {
    dialogue_focus::reset_camera(camera_q, follow_q, offset);
    *mode = CameraMode::Follow;
}

/// Run-condition factory: returns true when the given mode is active.
pub fn in_camera_mode(expected: CameraMode) -> impl Fn(Res<CameraMode>) -> bool + Clone {
    move |mode: Res<CameraMode>| *mode == expected
}
