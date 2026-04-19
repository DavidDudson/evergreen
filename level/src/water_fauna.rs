//! Small creatures around water bodies: frogs sitting on plain-pond /
//! lake tiles, water striders skating on the surface, and dragonflies
//! hovering near shorelines. All are decorative -- they don't wander
//! through the full `creatures` AI; instead a cheap animation system
//! bobs them in place.

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::water::WaterMap;
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

const FROG_SPRITE: &str = "sprites/creatures/greenwood/frog.webp";
const STRIDER_SPRITE: &str = "sprites/creatures/water/water_strider.webp";
const DRAGONFLY_SPRITE: &str = "sprites/creatures/water/dragonfly.webp";

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

/// Per-frog-eligible-tile chance (out of 100) to spawn a frog.
const FROG_CHANCE: u32 = 35;
/// Per water-tile chance (out of 100) to spawn a water strider.
const STRIDER_CHANCE: u32 = 40;
/// Per edge-adjacent water tile chance (out of 100) to spawn a dragonfly.
const DRAGONFLY_CHANCE: u32 = 30;

/// Visual sizes (square sprites, scaled to this pixel size).
const FROG_SIZE_PX: f32 = 8.0;
const STRIDER_SIZE_PX: f32 = 6.0;
const DRAGONFLY_SIZE_PX: f32 = 10.0;

/// Dragonfly hover amplitude + frequency.
const DRAGONFLY_HOVER_AMPLITUDE_PX: f32 = 4.0;
const DRAGONFLY_HOVER_FREQ_HZ: f32 = 1.2;

/// Water strider horizontal skate amplitude + frequency.
const STRIDER_SKATE_AMPLITUDE_PX: f32 = 3.0;
const STRIDER_SKATE_FREQ_HZ: f32 = 0.8;

/// Frog sit-bob amplitude + frequency (subtle -- frogs are mostly still).
const FROG_BOB_AMPLITUDE_PX: f32 = 0.5;
const FROG_BOB_FREQ_HZ: f32 = 0.6;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Frog {
    pub phase: f32,
    pub base_y: f32,
}

#[derive(Component)]
pub struct WaterStrider {
    pub phase: f32,
    pub base_x: f32,
}

#[derive(Component)]
pub struct Dragonfly {
    pub phase: f32,
    pub base_y: f32,
}

/// Generic marker so teardown can despawn all water creatures in one query.
#[derive(Component)]
pub struct WaterCreature;

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

pub fn spawn_area_water_fauna(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);
    let water = &world.water;
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223))
        .wrapping_add(0xFA_AA);

    for (local, kind) in water.tiles_in_area(area_pos) {
        let world_x = base_offset_x
            + f32::from(u16::try_from(local.x).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(local.y).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let hash = tile_hash(local.x, local.y, area_seed);

        // Water striders skate only on still water (not rivers).
        if kind.is_still() && (hash.wrapping_mul(3) % 100) < usize_of(STRIDER_CHANCE) {
            spawn_strider(commands, asset_server, world_x, world_y, hash);
        }

        // Frogs on plain + lake tiles only, edge preferred (visually on lily pads).
        if kind.spawns_frogs()
            && water.is_edge_tile(area_pos, local)
            && (hash.wrapping_mul(5) % 100) < usize_of(FROG_CHANCE)
        {
            spawn_frog(commands, asset_server, world_x, world_y, hash);
        }

        // Dragonflies hover over edge water (visually at shoreline).
        if water.is_edge_tile(area_pos, local)
            && (hash.wrapping_mul(11) % 100) < usize_of(DRAGONFLY_CHANCE)
        {
            spawn_dragonfly(commands, asset_server, world_x, world_y, hash);
        }
    }

    // Occasional dragonfly over grass adjacent to water.
    spawn_pondside_dragonflies(
        commands,
        asset_server,
        world,
        area_pos,
        area_seed,
        base_offset_x,
        base_offset_y,
        tile_px,
    );
}

fn usize_of(val: u32) -> usize {
    usize::try_from(val).unwrap_or(0)
}

fn spawn_strider(commands: &mut Commands, asset_server: &AssetServer, x: f32, y: f32, hash: usize) {
    #[allow(clippy::as_conversions)]
    let phase = (hash % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        WaterStrider { phase, base_x: x },
        Sprite {
            image: asset_server.load(STRIDER_SPRITE),
            custom_size: Some(Vec2::splat(STRIDER_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Tilemap.z_f32() + 0.8),
    ));
}

fn spawn_frog(commands: &mut Commands, asset_server: &AssetServer, x: f32, y: f32, hash: usize) {
    #[allow(clippy::as_conversions)]
    let phase = (hash.wrapping_mul(31) % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        Frog { phase, base_y: y },
        Sprite {
            image: asset_server.load(FROG_SPRITE),
            custom_size: Some(Vec2::splat(FROG_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Tilemap.z_f32() + 0.9),
    ));
}

fn spawn_dragonfly(
    commands: &mut Commands,
    asset_server: &AssetServer,
    x: f32,
    y: f32,
    hash: usize,
) {
    #[allow(clippy::as_conversions)]
    let phase = (hash.wrapping_mul(17) % 628) as f32 / 100.0;
    commands.spawn((
        WaterCreature,
        Dragonfly { phase, base_y: y },
        Sprite {
            image: asset_server.load(DRAGONFLY_SPRITE),
            custom_size: Some(Vec2::splat(DRAGONFLY_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(x, y, Layer::Weather.z_f32() - 1.0),
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_pondside_dragonflies(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area_pos: IVec2,
    area_seed: u32,
    base_offset_x: f32,
    base_offset_y: f32,
    tile_px: f32,
) {
    let Some(area) = world.get_area(area_pos) else {
        return;
    };
    let water = &world.water;
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let xu = u32::from(x);
            let yu = u32::from(y);
            let local = UVec2::new(xu, yu);
            if water.get(area_pos, local).is_some() {
                continue;
            }
            if area.terrain_at(xu, yu) != Some(Terrain::Grass) {
                continue;
            }
            if !neighbour_has_water(water, area_pos, local) {
                continue;
            }
            let hash = tile_hash(xu, yu, area_seed.wrapping_add(7));
            if (hash.wrapping_mul(13) % 100) >= usize_of(DRAGONFLY_CHANCE / 2) {
                continue;
            }
            let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
            let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
            spawn_dragonfly(commands, asset_server, world_x, world_y, hash);
        }
    }
}

fn neighbour_has_water(water: &WaterMap, area_pos: IVec2, local: UVec2) -> bool {
    const DELTAS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    DELTAS.iter().any(|&(dx, dy)| {
        let nx = i32::try_from(local.x).unwrap_or(0) + dx;
        let ny = i32::try_from(local.y).unwrap_or(0) + dy;
        if !(0..i32::from(MAP_WIDTH)).contains(&nx) || !(0..i32::from(MAP_HEIGHT)).contains(&ny) {
            return false;
        }
        #[allow(clippy::cast_sign_loss, clippy::as_conversions)]
        let n_local = UVec2::new(nx as u32, ny as u32);
        water.get(area_pos, n_local).is_some()
    })
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

type FrogQuery<'w, 's> =
    Query<'w, 's, (&'static Frog, &'static mut Transform), (Without<WaterStrider>, Without<Dragonfly>)>;
type StriderQuery<'w, 's> =
    Query<'w, 's, (&'static WaterStrider, &'static mut Transform), (Without<Frog>, Without<Dragonfly>)>;
type DragonflyQuery<'w, 's> =
    Query<'w, 's, (&'static Dragonfly, &'static mut Transform), (Without<Frog>, Without<WaterStrider>)>;

pub fn animate_water_fauna(
    time: Res<Time>,
    mut frogs: FrogQuery,
    mut striders: StriderQuery,
    mut dragonflies: DragonflyQuery,
) {
    let t = time.elapsed_secs();
    for (frog, mut tf) in &mut frogs {
        tf.translation.y =
            frog.base_y + (t * FROG_BOB_FREQ_HZ + frog.phase).sin() * FROG_BOB_AMPLITUDE_PX;
    }
    for (strider, mut tf) in &mut striders {
        tf.translation.x =
            strider.base_x + (t * STRIDER_SKATE_FREQ_HZ + strider.phase).sin() * STRIDER_SKATE_AMPLITUDE_PX;
    }
    for (fly, mut tf) in &mut dragonflies {
        let offset = (t * DRAGONFLY_HOVER_FREQ_HZ + fly.phase).sin() * DRAGONFLY_HOVER_AMPLITUDE_PX;
        tf.translation.y = fly.base_y + offset;
    }
}

pub fn despawn_water_fauna(mut commands: Commands, q: Query<Entity, With<WaterCreature>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
