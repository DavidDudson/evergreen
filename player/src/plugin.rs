use bevy::prelude::*;
use models::game_states::{should_despawn_world, GameState};

use crate::animation;
use crate::collision;
use crate::exit_check;
use crate::movement;
use crate::rustle;
use crate::spawning;
use crate::y_sort;

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
                    exit_check::check_exit_overlap,
                    y_sort::update_player_z,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                spawning::despawn.run_if(should_despawn_world),
            );
    }
}
