use bevy::prelude::*;
use models::game_states::GameState;

use crate::movement;
use crate::spawning;

pub use crate::spawning::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawning::spawn)
            .add_systems(
                Update,
                movement::move_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                spawning::despawn.run_if(not(in_state(GameState::Paused))),
            );
    }
}
