use bevy::prelude::*;
use bevy_light_2d::plugin::Light2dPlugin;
use models::game_states::GameState;

use crate::ambient::{reset_ambient_light, sync_ambient_light};
use crate::exit_light::attach_level_exit_light;
use crate::torch::update_player_torch;

/// Top-level lighting plugin -- composes `bevy_light_2d` + project systems.
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
        app.add_systems(
            Update,
            (
                sync_ambient_light,
                attach_level_exit_light,
                update_player_torch,
            )
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnExit(GameState::Playing), reset_ambient_light);
    }
}
