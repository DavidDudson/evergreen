use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::game_states::{GameState, should_despawn_world};

use crate::galen;
use crate::npc_anim;
use crate::npc_homes::NpcHomes;
use crate::npc_labels::{self, InteractIconState};
use crate::npc_wander;
use crate::npcs;
use crate::scenery;
use crate::spawning;
use crate::world::{AreaChanged, WorldMap};

pub use crate::area::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::spawning::{tile_size, TILE_SIZE_PX};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let world = WorldMap::new(rand::random());
        let npc_homes = NpcHomes::assign(&world.area_positions(), world.seed());

        app.init_resource::<InteractIconState>()
            .add_plugins(TilemapPlugin)
            .add_message::<AreaChanged>()
            .insert_resource(npc_homes)
            .insert_resource(world)
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawning::spawn_tilemap,
                    scenery::spawn_scenery,
                    npcs::spawn_npcs,
                    galen::spawn_galen,
                ),
            )
            .add_systems(
                Update,
                (
                    spawning::respawn_on_area_change,
                    scenery::respawn_scenery_on_area_change,
                    npcs::respawn_npcs_on_area_change,
                    galen::respawn_galen_on_area_change,
                    scenery::animate_rustle,
                    npc_labels::attach_labels,
                    npc_labels::sync_interact_icon,
                    galen::randomise_question,
                    npc_anim::advance_npc_frame,
                    npc_anim::reset_npc_anim_on_change,
                    npc_wander::wander_npcs,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    spawning::despawn_tilemap,
                    scenery::despawn_scenery,
                    npcs::despawn_npcs,
                    galen::despawn_galen,
                )
                    .run_if(should_despawn_world),
            );
    }
}
