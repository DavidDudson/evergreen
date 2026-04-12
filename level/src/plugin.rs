use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::alignment::PlayerAlignment;
use models::game_states::{GameState, should_despawn_world};

use crate::bark_bubbles;
use crate::exit;
use crate::galen;
use crate::npc_anim;
use crate::npc_labels::{self, InteractIconState};
use crate::npc_wander;
use crate::npcs;
use crate::scenery;
use crate::spawning::{self, SpawnedAreas};
use crate::world::{AreaChanged, WorldMap};

pub use crate::area::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::spawning::{tile_size, TILE_SIZE_PX};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractIconState>()
            .init_resource::<SpawnedAreas>()
            .add_plugins(TilemapPlugin)
            .add_message::<AreaChanged>()
            .insert_resource(WorldMap::new(rand::random(), 50))
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    regenerate_world,
                    spawning::spawn_initial_areas,
                    galen::spawn_galen,
                    exit::spawn_exit,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    spawning::ensure_neighbors_on_area_change,
                    scenery::animate_rustle,
                    npc_labels::attach_labels,
                    npc_labels::sync_interact_icon,
                    npc_anim::advance_npc_frame,
                    npc_anim::reset_npc_anim_on_change,
                    npc_wander::wander_npcs,
                    bark_bubbles::spawn_bark_bubble,
                    bark_bubbles::tick_bark_bubbles,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    spawning::despawn_all_areas,
                    scenery::despawn_scenery,
                    npcs::despawn_npcs,
                    galen::despawn_galen,
                    exit::despawn_exit,
                )
                    .run_if(should_despawn_world),
            );
    }
}

/// Regenerate the world with a fresh seed, biased toward the player's
/// dominant faction alignment.  Skips if a world is already loaded
/// (e.g. returning from Dialogue or Paused).
fn regenerate_world(
    mut world: ResMut<WorldMap>,
    alignment: Res<PlayerAlignment>,
    spawned: Res<SpawnedAreas>,
) {
    if !spawned.0.is_empty() {
        return;
    }
    let dominant = alignment.dominant_area_alignment();
    *world = WorldMap::new(rand::random(), dominant);
}
