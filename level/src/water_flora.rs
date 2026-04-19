//! Decorative plants around water bodies: lily pads floating on the surface,
//! reeds and cattails at the shoreline, ferns and mushrooms on damp ground
//! adjacent to ponds. Deterministic per-tile placement keyed off the world
//! seed so re-entering an area produces the same layout.

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::layer::Layer;
use models::scenery::Scenery;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::water::{WaterKind, WaterMap};
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LILY_PAD_SPRITE: &str = "sprites/scenery/water_flora/lily_pad.webp";
const LILY_FLOWER_SPRITE: &str = "sprites/scenery/water_flora/lily_flower.webp";
const REED_SPRITE: &str = "sprites/scenery/water_flora/reed.webp";
const CATTAIL_SPRITE: &str = "sprites/scenery/water_flora/cattail.webp";
const FERN_SPRITE: &str = "sprites/scenery/flora_extra/fern.webp";
const MUSHROOM_RED_SPRITE: &str = "sprites/scenery/flora_extra/mushroom_red.webp";

const LILY_PAD_SIZE_PX: f32 = 14.0;
const REED_SIZE_PX: f32 = 20.0;
const FERN_SIZE_PX: f32 = 14.0;
const MUSHROOM_SIZE_PX: f32 = 12.0;

/// Out of 100: per water-tile chance to spawn a lily pad.
const LILY_CHANCE: u32 = 50;
/// Out of 100: chance that lily pad becomes the flowering variant.
const LILY_FLOWER_CHANCE: u32 = 25;
/// Out of 100: per edge-tile chance to spawn a reed clump.
const REED_CHANCE: u32 = 45;
/// Out of 100: per edge-tile chance (if no reed) to spawn a cattail clump.
const CATTAIL_CHANCE: u32 = 35;
/// Out of 100: per grass-neighbour-of-water chance to spawn a fern.
const FERN_CHANCE: u32 = 28;
/// Out of 100: per grass-tile chance in darkwood to spawn a mushroom.
const MUSHROOM_CHANCE: u32 = 6;
/// Minimum biome alignment for mushrooms (darkwood).
const MUSHROOM_ALIGNMENT_MIN: u8 = 70;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Layer z offsets relative to water so flora renders on top of water but
/// below trees / creatures.
const LILY_Z_OFFSET: f32 = 0.6;
const REED_Z_OFFSET: f32 = 0.7;
const GROUND_FLORA_Z_OFFSET: f32 = 0.55;

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct WaterFlora;

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Spawn lily pads, reeds, cattails, and damp-ground flora for one area based
/// on the `WaterMap` already resident in `world.water`.
pub fn spawn_area_water_flora(
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
        .wrapping_add(0xF10_7A);

    for (local, kind) in water.tiles_in_area(area_pos) {
        spawn_surface_flora(
            commands,
            asset_server,
            water,
            area_pos,
            local,
            kind,
            area_seed,
            base_offset_x,
            base_offset_y,
            tile_px,
        );
    }

    spawn_ground_flora(
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

#[allow(clippy::too_many_arguments)]
fn spawn_surface_flora(
    commands: &mut Commands,
    asset_server: &AssetServer,
    water: &WaterMap,
    area_pos: IVec2,
    local: UVec2,
    kind: WaterKind,
    area_seed: u32,
    base_offset_x: f32,
    base_offset_y: f32,
    tile_px: f32,
) {
    let hash = tile_hash(local.x, local.y, area_seed);
    let world_x =
        base_offset_x + f32::from(u16::try_from(local.x).unwrap_or(0)) * tile_px + tile_px / 2.0;
    let world_y =
        base_offset_y + f32::from(u16::try_from(local.y).unwrap_or(0)) * tile_px + tile_px / 2.0;

    let is_edge = water.is_edge_tile(area_pos, local);

    // Lily pads only spawn on non-hot-spring water, interior tiles preferred.
    if kind.spawns_lily_pads() {
        let pad_hash = u32::try_from(hash % 100).unwrap_or(0);
        if !is_edge && pad_hash < LILY_CHANCE {
            let path = if pad_hash.wrapping_mul(7) % 100 < LILY_FLOWER_CHANCE {
                LILY_FLOWER_SPRITE
            } else {
                LILY_PAD_SPRITE
            };
            let jitter_x = jitter(hash.wrapping_add(1), tile_px * 0.25);
            let jitter_y = jitter(hash.wrapping_add(2), tile_px * 0.25);
            commands.spawn((
                WaterFlora,
                Scenery,
                Sprite {
                    image: asset_server.load(path),
                    custom_size: Some(Vec2::splat(LILY_PAD_SIZE_PX)),
                    ..default()
                },
                Transform::from_xyz(
                    world_x + jitter_x,
                    world_y + jitter_y,
                    Layer::Tilemap.z_f32() + LILY_Z_OFFSET,
                ),
            ));
        }
    }

    // Reeds / cattails at shoreline of plain ponds + lakes.
    if kind.spawns_lily_pads() && is_edge {
        let reed_hash = u32::try_from(hash.wrapping_mul(31) % 100).unwrap_or(0);
        let path = if reed_hash < REED_CHANCE {
            Some(REED_SPRITE)
        } else if reed_hash < REED_CHANCE + CATTAIL_CHANCE {
            Some(CATTAIL_SPRITE)
        } else {
            None
        };
        if let Some(path) = path {
            let jitter_x = jitter(hash.wrapping_add(3), tile_px * 0.35);
            commands.spawn((
                WaterFlora,
                Scenery,
                Sprite {
                    image: asset_server.load(path),
                    custom_size: Some(Vec2::splat(REED_SIZE_PX)),
                    ..default()
                },
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(
                    world_x + jitter_x,
                    world_y,
                    Layer::Tilemap.z_f32() + REED_Z_OFFSET,
                ),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_ground_flora(
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
    let darkwood = area.alignment >= MUSHROOM_ALIGNMENT_MIN;

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
            let hash = tile_hash(xu, yu, area_seed.wrapping_add(42));

            // Fern near water edges.
            let near_water = any_neighbour_is_water(water, area_pos, local);
            if near_water {
                let fern_hash = u32::try_from(hash % 100).unwrap_or(0);
                if fern_hash < FERN_CHANCE {
                    let jitter_x = jitter(hash.wrapping_add(10), tile_px * 0.3);
                    let jitter_y = jitter(hash.wrapping_add(11), tile_px * 0.3);
                    let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
                    let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
                    commands.spawn((
                        WaterFlora,
                        Scenery,
                        Sprite {
                            image: asset_server.load(FERN_SPRITE),
                            custom_size: Some(Vec2::splat(FERN_SIZE_PX)),
                            ..default()
                        },
                        Transform::from_xyz(
                            world_x + jitter_x,
                            world_y + jitter_y,
                            Layer::Tilemap.z_f32() + GROUND_FLORA_Z_OFFSET,
                        ),
                    ));
                    continue;
                }
            }

            // Mushrooms sprinkled in darkwood regardless of water proximity.
            if darkwood {
                let mush_hash = u32::try_from(hash.wrapping_mul(7) % 100).unwrap_or(0);
                if mush_hash < MUSHROOM_CHANCE {
                    let jitter_x = jitter(hash.wrapping_add(20), tile_px * 0.3);
                    let jitter_y = jitter(hash.wrapping_add(21), tile_px * 0.3);
                    let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
                    let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
                    commands.spawn((
                        WaterFlora,
                        Scenery,
                        Sprite {
                            image: asset_server.load(MUSHROOM_RED_SPRITE),
                            custom_size: Some(Vec2::splat(MUSHROOM_SIZE_PX)),
                            ..default()
                        },
                        Anchor::BOTTOM_CENTER,
                        Transform::from_xyz(
                            world_x + jitter_x,
                            world_y + jitter_y,
                            Layer::Tilemap.z_f32() + GROUND_FLORA_Z_OFFSET,
                        ),
                    ));
                }
            }
        }
    }
}

fn any_neighbour_is_water(water: &WaterMap, area_pos: IVec2, local: UVec2) -> bool {
    const DELTAS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    DELTAS.iter().any(|&(dx, dy)| {
        let nx = i32::try_from(local.x).unwrap_or(0) + dx;
        let ny = i32::try_from(local.y).unwrap_or(0) + dy;
        let w = i32::from(MAP_WIDTH);
        let h = i32::from(MAP_HEIGHT);
        if !(0..w).contains(&nx) || !(0..h).contains(&ny) {
            return false;
        }
        #[allow(clippy::cast_sign_loss, clippy::as_conversions)]
        let n_local = UVec2::new(nx as u32, ny as u32);
        water.get(area_pos, n_local).is_some()
    })
}

fn jitter(seed: usize, range_px: f32) -> f32 {
    let h = seed.wrapping_mul(2_654_435_761) ^ (seed >> 13);
    #[allow(clippy::as_conversions)]
    let norm = (h % 1000) as f32 / 1000.0 - 0.5;
    norm * 2.0 * range_px
}

/// Despawn every water-flora entity on teardown.
pub fn despawn_water_flora(mut commands: Commands, q: Query<Entity, With<WaterFlora>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
