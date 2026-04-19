use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapPlugin;
use models::alignment::PlayerAlignment;
use models::game_states::{should_despawn_world, GameState};
use models::time::GameClock;

use models::weather::WeatherState;
use models::wind::{WindDirection, WindStrength};

use crate::bark_bubbles;
use crate::creatures;
use crate::decorations;
use crate::exit;
use crate::galen;
use crate::grass;
use crate::npc_anim;
use crate::npc_labels::{self, InteractIconState};
use crate::npc_wander;
use crate::npcs;
use crate::reveal;
use crate::scenery;
use crate::shadows;
use crate::spawning::{self, SpawnedAreas};
use crate::beach;
use crate::puddles;
use crate::water;
use crate::water_fauna;
use crate::water_flora;
use crate::weather;
use crate::world::{AreaChanged, WorldMap};

pub use crate::area::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::spawning::{tile_size, TILE_SIZE_PX};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractIconState>()
            .init_resource::<SpawnedAreas>()
            .init_resource::<WindStrength>()
            .init_resource::<WindDirection>()
            .init_resource::<WeatherState>()
            .init_resource::<puddles::PuddleSpawnTimer>()
            .init_resource::<puddles::SteamAccumulator>()
            .add_plugins(TilemapPlugin)
            .add_systems(Startup, shadows::init_shadow_assets)
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
                    npcs::update_npc_z,
                    galen::update_galen_z,
                    bark_bubbles::spawn_bark_bubble,
                    bark_bubbles::tick_bark_bubbles,
                    reveal::detect_reveals,
                    reveal::animate_reveals,
                    weather::weather_state_machine,
                    weather::sync_wind_strength,
                    weather::spawn_weather_particles,
                    weather::spawn_fireflies,
                    weather::animate_fireflies,
                    weather::update_weather_particles,
                    grass::animate_grass_sway,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    shadows::animate_shadow_sun,
                    water_fauna::animate_water_fauna,
                    puddles::spawn_puddles,
                    puddles::fade_puddles_when_clear,
                    puddles::spawn_hotspring_steam,
                    puddles::update_steam,
                    water::animate_water_surface,
                    beach::animate_crabs,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (weather::spawn_dust_motes, weather::spawn_fog_patches)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    creatures::creature_state_transitions,
                    creatures::creature_movement,
                    creatures::creature_animation,
                    creatures::creature_flying_bob,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    spawning::despawn_all_areas,
                    scenery::despawn_scenery,
                    decorations::despawn_decorations,
                    npcs::despawn_npcs,
                    galen::despawn_galen,
                    exit::despawn_exit,
                    weather::despawn_weather_particles,
                    grass::despawn_grass,
                    creatures::despawn_creatures,
                    water::despawn_water,
                    water::despawn_stones,
                    water_flora::despawn_water_flora,
                    water_fauna::despawn_water_fauna,
                    puddles::despawn_puddles,
                    puddles::despawn_steam,
                    beach::despawn_sand,
                    beach::despawn_piers,
                )
                    .run_if(should_despawn_world),
            );
    }
}

/// Maximum hour value (exclusive) when randomising a new game's start time.
const HOURS_PER_DAY: f32 = 24.0;

/// Regenerate the world with a fresh seed, biased toward the player's
/// dominant faction alignment.  Skips if a world is already loaded
/// (e.g. returning from Dialogue or Paused).
fn regenerate_world(
    mut world: ResMut<WorldMap>,
    mut clock: ResMut<GameClock>,
    alignment: Res<PlayerAlignment>,
    spawned: Res<SpawnedAreas>,
) {
    if !spawned.0.is_empty() {
        return;
    }
    let dominant = alignment.dominant_area_alignment();
    *world = WorldMap::new(rand::random(), dominant);
    clock.hour = rand::random::<f32>() * HOURS_PER_DAY;
}
