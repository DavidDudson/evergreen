use bevy::prelude::*;
use models::game_states::GameState;

use crate::animation;
use crate::collision;
use crate::movement;
use crate::rustle;
use crate::spawning;

pub use crate::spawning::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawning::spawn)
            .add_systems(
                Update,
                (
                    animation::update_animation_state,
                    animation::advance_frame,
                    movement::move_player,
                    collision::resolve_scenery_collisions,
                    rustle::trigger_rustle,
                    movement::check_area_transition,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                spawning::despawn.run_if(not(in_state(GameState::Paused))),
            );
    }
}
