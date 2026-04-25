// level/src/grass.rs

use bevy::math::IVec2;
use bevy::prelude::*;
use models::grass::{GrassTuft, WindSway};
use models::layer::Layer;
use models::wind::WindStrength;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::biome_registry::BiomeRegistry;
use crate::blending;
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::world::WorldMap;
use models::shadow::{GRASS_SHADOW_HALF_PX, GRASS_SHADOW_OFFSET_Y_PX};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Y-sort scale factor for z-ordering within the World layer.
const Y_SORT_SCALE: f32 = 0.001;

/// Minimum grass tufts per area.
const MIN_GRASS_PER_AREA: usize = 20;
/// Maximum grass tufts per area.
const MAX_GRASS_PER_AREA: usize = 30;

/// Inset from area edges (same as decorations).
const EDGE_INSET: u16 = 2;

/// Unique salt added to area seed to avoid overlapping decoration/tree positions.
const GRASS_SEED_SALT: u32 = 55_555;

/// Sway oscillation frequency in Hz.
const SWAY_FREQUENCY_HZ: f32 = 3.0;
/// Maximum sway angle in radians.
const SWAY_MAX_ANGLE_RAD: f32 = 0.1;

/// Hash divisor used to derive a per-tuft sway phase (~ 2 PI * 1000).
const PHASE_HASH_DIVISOR: usize = 6283;
/// Phase divisor (matching the divisor above) so phases land in 0..2 PI.
const PHASE_HASH_SCALE: f32 = 1000.0;

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn 20-30 grass tufts for a single area on grass tiles.
pub fn spawn_area_grass(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shadow_assets: &DropShadowAssets,
    registry: &BiomeRegistry,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223));

    let grass_seed = area_seed.wrapping_add(GRASS_SEED_SALT);

    // Collect candidate grass tiles (inset from edges).
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in EDGE_INSET..(MAP_WIDTH - EDGE_INSET) {
        for y in EDGE_INSET..(MAP_HEIGHT - EDGE_INSET) {
            let xu = u32::from(x);
            let yu = u32::from(y);
            if area.terrain_at(xu, yu) == Some(Terrain::Grass) {
                candidates.push((xu, yu));
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    // Deterministically shuffle candidates.
    let len = candidates.len();
    let mut rng = u64::from(grass_seed);
    for i in (1..len).rev() {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let j = (rng % u64::try_from(i + 1).expect("i+1 fits u64")) as usize;
        candidates.swap(i, j);
    }

    // Pick tuft count (20-30) deterministically.
    rng = lcg(rng);
    #[allow(clippy::as_conversions)]
    let range = (MAX_GRASS_PER_AREA - MIN_GRASS_PER_AREA + 1) as u64;
    #[allow(clippy::as_conversions)]
    let count = MIN_GRASS_PER_AREA + (rng % range) as usize;
    let count = count.min(candidates.len());

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        let effective_alignment =
            blending::blended_alignment(area.alignment, xu, yu, area_pos, world);
        let pool = registry.grass(effective_alignment);

        let variant = tile_hash(
            xu,
            yu,
            grass_seed.wrapping_add(u32::try_from(i).expect("i fits u32")),
        ) % pool.len();
        let path = pool[variant];

        let world_x = base_offset_x
            + f32::from(u16::try_from(xu).expect("xu fits u16")) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(yu).expect("yu fits u16")) * tile_px
            + tile_px / 2.0;

        let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;

        // Derive phase from tile hash so each tuft sways independently.
        #[allow(clippy::as_conversions)]
        let phase = (tile_hash(xu, yu, grass_seed) % PHASE_HASH_DIVISOR) as f32 / PHASE_HASH_SCALE;

        let parent = commands
            .spawn((
                GrassTuft,
                WindSway { phase },
                Sprite {
                    image: asset_server.load(path),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, z),
            ))
            .id();

        spawn_drop_shadow(
            commands,
            shadow_assets,
            parent,
            GRASS_SHADOW_HALF_PX,
            GRASS_SHADOW_OFFSET_Y_PX,
        );
    }
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Oscillate grass tufts based on wind strength and per-entity phase.
pub fn animate_grass_sway(
    time: Res<Time>,
    wind: Res<WindStrength>,
    mut query: Query<(&WindSway, &mut Transform), With<GrassTuft>>,
) {
    let elapsed = time.elapsed_secs();
    for (sway, mut tf) in &mut query {
        let angle = (elapsed * SWAY_FREQUENCY_HZ + sway.phase).sin() * SWAY_MAX_ANGLE_RAD * wind.0;
        tf.rotation = Quat::from_rotation_z(angle);
    }
}

// ---------------------------------------------------------------------------
// Despawn
// ---------------------------------------------------------------------------

/// Despawn all grass tufts on game exit.
pub fn despawn_grass(mut commands: Commands, query: Query<Entity, With<GrassTuft>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// LCG
// ---------------------------------------------------------------------------

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
