use bevy::prelude::*;
use models::game_states::{should_despawn_world, GameState};

use crate::animation;
use crate::collision;
use crate::exit_check;
use crate::hop;
use crate::movement;
use crate::portal_trigger;
use crate::rustle;
use crate::spawning;
use crate::water_state::{
    spawn_splashes, tick_splashes, update_player_water_state, PlayerWaterState, SplashTimer,
};
use crate::y_sort;

pub use crate::spawning::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerWaterState>()
            .init_resource::<SplashTimer>()
            .add_systems(OnEnter(GameState::Playing), spawning::spawn)
            .add_systems(
                Update,
                (
                    update_player_water_state,
                    animation::update_animation_state,
                    animation::advance_frame,
                    movement::move_player,
                    collision::resolve_scenery_collisions,
                    rustle::trigger_rustle,
                    movement::check_area_transition,
                    exit_check::check_exit_overlap,
                    y_sort::update_player_z,
                    hop::apply_hop_bob,
                    spawn_splashes,
                    tick_splashes,
                    portal_trigger::detect_portal_overlap,
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
