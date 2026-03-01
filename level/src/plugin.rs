use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::game_states::GameState;

use crate::scenery;
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
            .add_systems(
                OnEnter(GameState::Playing),
                (spawning::spawn_tilemap, scenery::spawn_scenery),
            )
            .add_systems(
                Update,
                (
                    spawning::respawn_on_area_change,
                    scenery::respawn_scenery_on_area_change,
                    scenery::animate_rustle,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (spawning::despawn_tilemap, scenery::despawn_scenery)
                    .run_if(not(in_state(GameState::Paused))),
            );
    }
}
