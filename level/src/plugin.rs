use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::game_states::GameState;

use crate::spawning;

pub use crate::spawning::TILE_SIZE;
pub use crate::tile_map::{MAP_HEIGHT, MAP_WIDTH};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), spawning::spawn_tilemap)
            .add_systems(
                OnExit(GameState::Playing),
                spawning::despawn_tilemap.run_if(not(in_state(GameState::Paused))),
            );
    }
}
