//! Beaches, piers, and crab fauna that live around ocean tiles.
//!
//! Beaches: sand tiles stored on `WaterMap::sand` render as simple decorative
//! Sprites, no collider (player walks on sand).
//!
//! Piers: tile-based runs of plank tiles extending from a dead-end coastal
//! area's land out into the ocean. Generated into `WaterMap::piers`; this
//! file just spawns sprites for those tiles. The pier tiles are walkable
//! (no collider) and overlay any underlying ocean.
//!
//! Crabs: per-tile chance of spawning on a sand tile; scuttle sideways via a
//! sine phase.

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;
use models::scenery::Scenery;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::tile_hash;
use crate::water::WaterKind;
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

const CRAB_SPRITE: &str = "sprites/creatures/water/crab.webp";

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

const CRAB_SIZE_PX: f32 = 10.0;

/// Per sand-tile chance (out of 100) to spawn a crab.
const CRAB_CHANCE: u32 = 20;

/// Crab horizontal scuttle amplitude + frequency.
const CRAB_SCUTTLE_AMPLITUDE_PX: f32 = 2.5;
const CRAB_SCUTTLE_FREQ_HZ: f32 = 1.1;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct SandTile;

#[derive(Component)]
pub struct PierPiece;

#[derive(Component)]
pub struct Crab {
    pub phase: f32,
    pub base_x: f32,
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn spawn_area_beach(
    commands: &mut Commands,
    asset_server: &AssetServer,
    wang: &crate::wang::WangTilesets,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);

    // Sand tiles (wang-tiled) + crabs.
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223))
        .wrapping_add(0xBE_AC_30);

    let sand_tileset = wang.get(crate::wang::SAND_GRASS);
    for y in 0..u32::from(crate::area::MAP_HEIGHT) {
        for x in 0..u32::from(crate::area::MAP_WIDTH) {
            let mask = crate::water::sand_mask(&world.water, area_pos, x, y);
            if mask == 0 {
                continue;
            }
            let local = bevy::math::UVec2::new(x, y);
            let world_x =
                base_offset_x + f32::from(u16::try_from(x).unwrap_or(0)) * tile_px + tile_px / 2.0;
            let world_y =
                base_offset_y + f32::from(u16::try_from(y).unwrap_or(0)) * tile_px + tile_px / 2.0;
            commands.spawn((
                SandTile,
                Scenery,
                Sprite {
                    image: sand_tileset.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: sand_tileset.layout.clone(),
                        index: sand_tileset.lut[usize::from(mask)],
                    }),
                    custom_size: Some(Vec2::splat(tile_px)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.3),
            ));

            // Crabs only spawn on fully-sand tiles (mask = 0b1111).
            if mask == 0b1111 && world.water.has_sand(area_pos, local) {
                let hash = tile_hash(x, y, area_seed);
                #[allow(clippy::as_conversions)]
                let crab_roll = u32::try_from(hash % 100).unwrap_or(0);
                if crab_roll < CRAB_CHANCE {
                    #[allow(clippy::as_conversions)]
                    let phase = (hash.wrapping_mul(31) % 628) as f32 / 100.0;
                    commands.spawn((
                        SandTile,
                        Crab {
                            phase,
                            base_x: world_x,
                        },
                        Sprite {
                            image: asset_server.load(CRAB_SPRITE),
                            custom_size: Some(Vec2::splat(CRAB_SIZE_PX)),
                            ..default()
                        },
                        Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.8),
                    ));
                }
            }
        }
    }

    // Pier tiles (data generated by `water::pier::generate_piers`).
    spawn_pier_tiles(
        commands,
        wang,
        world,
        area_pos,
        base_offset_x,
        base_offset_y,
        tile_px,
    );
}

// ---------------------------------------------------------------------------
// Pier rendering -- PIER_OCEAN wang tileset (lower=ocean, upper=plank).
// ---------------------------------------------------------------------------

fn spawn_pier_tiles(
    commands: &mut Commands,
    wang: &crate::wang::WangTilesets,
    world: &WorldMap,
    area_pos: IVec2,
    base_offset_x: f32,
    base_offset_y: f32,
    tile_px: f32,
) {
    let pier_ocean = wang.get(crate::wang::PIER_OCEAN);
    let pier_sand = wang.get(crate::wang::PIER_SAND);
    for y in 0..u32::from(MAP_HEIGHT) {
        for x in 0..u32::from(MAP_WIDTH) {
            let mask = pier_mask(world, area_pos, x, y);
            if mask == 0 {
                continue;
            }
            // Pick PIER_SAND over land/sand cells, PIER_OCEAN over ocean.
            // Decision per-tile by what sits under the pier here.
            let local = UVec2::new(x, y);
            let over_sand = world.water.has_sand(area_pos, local)
                || !matches!(
                    world.water.get(area_pos, local),
                    Some(crate::water::WaterKind::Ocean)
                );
            let pier_set = if over_sand { pier_sand } else { pier_ocean };
            let world_x =
                base_offset_x + f32::from(u16::try_from(x).unwrap_or(0)) * tile_px + tile_px / 2.0;
            let world_y =
                base_offset_y + f32::from(u16::try_from(y).unwrap_or(0)) * tile_px + tile_px / 2.0;
            commands.spawn((
                PierPiece,
                Scenery,
                Sprite {
                    image: pier_set.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: pier_set.layout.clone(),
                        index: pier_set.lut[usize::from(mask)],
                    }),
                    custom_size: Some(Vec2::splat(tile_px)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.9),
            ));
        }
    }
}

fn pier_mask(world: &WorldMap, area_pos: IVec2, x: u32, y: u32) -> u8 {
    let nw = pier_vertex(world, area_pos, x, y + 1);
    let ne = pier_vertex(world, area_pos, x + 1, y + 1);
    let sw = pier_vertex(world, area_pos, x, y);
    let se = pier_vertex(world, area_pos, x + 1, y);
    crate::wang::wang_mask(nw, ne, sw, se)
}

fn pier_vertex(world: &WorldMap, area_pos: IVec2, vx: u32, vy: u32) -> bool {
    for (dx, dy) in [(-1i32, -1i32), (0, -1), (-1, 0), (0, 0)] {
        let lx = i32::try_from(vx).unwrap_or(0) + dx;
        let ly = i32::try_from(vy).unwrap_or(0) + dy;
        if lx < 0 || ly < 0 {
            continue;
        }
        if lx >= i32::from(MAP_WIDTH) || ly >= i32::from(MAP_HEIGHT) {
            continue;
        }
        let local =
            UVec2::new(u32::try_from(lx).unwrap_or(0), u32::try_from(ly).unwrap_or(0));
        if world.water.has_pier(area_pos, local) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

pub fn animate_crabs(time: Res<Time>, mut crabs: Query<(&Crab, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (crab, mut tf) in &mut crabs {
        tf.translation.x =
            crab.base_x + (t * CRAB_SCUTTLE_FREQ_HZ + crab.phase).sin() * CRAB_SCUTTLE_AMPLITUDE_PX;
    }
}

// ---------------------------------------------------------------------------
// Teardown
// ---------------------------------------------------------------------------

pub fn despawn_sand(mut commands: Commands, q: Query<Entity, With<SandTile>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn despawn_piers(mut commands: Commands, q: Query<Entity, With<PierPiece>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

// Suppress dead-code warnings on the ocean-only WaterKind imported to keep
// compile-time confidence across future refactors (the variant stays used
// via pattern matches elsewhere, but not referenced directly in this file).
const _: WaterKind = WaterKind::Ocean;
