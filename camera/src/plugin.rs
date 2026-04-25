use bevy::prelude::*;
use models::game_states::GameState;

use crate::config::CameraConfig;
use crate::dialogue_focus;
use crate::mode::{enter_dialogue_focus, exit_dialogue_focus, in_camera_mode, CameraMode};
use crate::setup::spawn_camera;
use crate::smooth;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<smooth::CameraOffset>()
            .init_resource::<CameraConfig>()
            .init_resource::<CameraMode>();

        app.add_systems(Startup, spawn_camera);

        app.add_systems(OnEnter(GameState::Dialogue), enter_dialogue_focus);
        app.add_systems(OnExit(GameState::Dialogue), exit_dialogue_focus);

        app.add_systems(
            Update,
            dialogue_focus::focus_on_dialogue
                .run_if(in_state(GameState::Dialogue))
                .run_if(in_camera_mode(CameraMode::DialogueFocus)),
        );

        app.add_systems(
            PostUpdate,
            smooth::follow_player
                .run_if(in_state(GameState::Playing))
                .run_if(in_camera_mode(CameraMode::Follow)),
        );
    }
}
