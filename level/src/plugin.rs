use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::game_states::GameState;

use crate::spawning;
use crate::world::{AreaChanged, WorldMap};

pub use crate::area::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::spawning::{tile_size, TILE_SIZE_PX};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_message::<AreaChanged>()
            .insert_resource(WorldMap::new(42))
            .add_systems(OnEnter(GameState::Playing), spawning::spawn_tilemap)
            .add_systems(
                Update,
                spawning::respawn_on_area_change.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                spawning::despawn_tilemap.run_if(not(in_state(GameState::Paused))),
            );
    }
}
