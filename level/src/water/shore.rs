//! Per-area water-tile and stepping-stone spawning + wang masks.

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;
use models::player::HopTrigger;
use models::scenery::{Scenery, SceneryCollider};

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::wang;
use crate::world::WorldMap;

use super::animation::AnimatedWater;
use super::tiles::{neighbour_key, WaterKind, WaterMap, WaterTile};

/// Stepping-stone sprite (placed over walkable river tiles).
const STONE_SPRITE: &str = "sprites/scenery/ponds/stepping_stone.webp";

/// Rendered sprite size in pixels (wider than a tile so neighbours overlap
/// and the pond outline looks organic).
const WATER_SPRITE_SIZE_PX: f32 = 20.0;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Collider half-extent (slightly smaller than the tile so the player can
/// squeeze along shorelines without getting stuck).
const WATER_COLLIDER_HALF_PX: f32 = 7.0;

/// Z-offset above tilemap layer for water sprites.
const WATER_Z_OFFSET: f32 = 0.4;
/// Per-mask z-bias so deeper masks render above shallower neighbours.
const WATER_Z_PER_MASK: f32 = 0.001;
/// Z-offset for stepping stones above the water.
const STONE_Z_OFFSET: f32 = 0.7;

/// Marker for stepping-stone sprites. Player systems use this to trigger the
/// hop-bob animation while a player is standing on a stone.
#[derive(Component)]
pub struct SteppingStone;

/// Spawn wang-tiled water sprites for every tile position in `area_pos`
/// that has at least one water-owning vertex. Transition tiles on land
/// adjacent to water render automatically.
pub fn spawn_area_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    wang_sets: &wang::WangTilesets,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);

    // Each kind family uses one wang tileset and one same-kind predicate.
    type Family<'a> = (&'a wang::WangTileset, fn(WaterKind) -> bool, WaterKind);
    let families: [Family; 5] = [
        (
            wang_sets.get(wang::POND_GRASS),
            |k| matches!(k, WaterKind::Plain | WaterKind::Lake),
            WaterKind::Plain,
        ),
        (
            wang_sets.get(wang::HOTSPRING_GRASS),
            |k| k == WaterKind::HotSpring,
            WaterKind::HotSpring,
        ),
        (
            wang_sets.get(wang::RIVER_GRASS),
            |k| matches!(k, WaterKind::RiverNS | WaterKind::RiverEW),
            WaterKind::RiverNS,
        ),
        (
            wang_sets.get(wang::WATERFALL_GRASS),
            |k| k == WaterKind::Waterfall,
            WaterKind::Waterfall,
        ),
        (
            wang_sets.get(wang::OCEAN_SAND),
            |k| k == WaterKind::Ocean,
            WaterKind::Ocean,
        ),
    ];

    for y in 0..u32::from(MAP_HEIGHT) {
        for x in 0..u32::from(MAP_WIDTH) {
            for &(tileset, predicate, marker_kind) in &families {
                let mask = kind_mask(&world.water, area_pos, x, y, predicate);
                if mask == 0 {
                    continue;
                }
                let world_x = base_offset_x
                    + f32::from(u16::try_from(x).unwrap_or(0)) * tile_px
                    + tile_px / 2.0;
                let world_y = base_offset_y
                    + f32::from(u16::try_from(y).unwrap_or(0)) * tile_px
                    + tile_px / 2.0;
                let atlas_idx = tileset.lut[usize::from(mask)];
                let local = UVec2::new(x, y);
                let is_center = mask == 0b1111;
                let has_stone = world.water.has_stone(area_pos, local);
                let z = Layer::Tilemap.z_f32()
                    + WATER_Z_OFFSET
                    + f32::from(u16::from(mask)) * WATER_Z_PER_MASK;
                let mut entity = commands.spawn((
                    WaterTile { kind: marker_kind },
                    AnimatedWater,
                    Scenery,
                    Sprite {
                        image: tileset.texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: tileset.layout.clone(),
                            index: atlas_idx,
                        }),
                        custom_size: Some(Vec2::splat(tile_px)),
                        ..default()
                    },
                    Transform::from_xyz(world_x, world_y, z),
                ));
                // Only fully-water tiles block the player. Transition (partial)
                // tiles are walkable so players can step onto shore edges.
                if is_center && !has_stone {
                    entity.insert(SceneryCollider {
                        half_extents: Vec2::splat(WATER_COLLIDER_HALF_PX),
                        center_offset: Vec2::ZERO,
                    });
                }
            }
        }
    }

    // Stepping stones rendered on top of the river water at crossings.
    for local in world.water.stones_in_area(area_pos) {
        let world_x = base_offset_x
            + f32::from(u16::try_from(local.x).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(local.y).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        commands.spawn((
            SteppingStone,
            HopTrigger,
            Scenery,
            Sprite {
                image: asset_server.load(STONE_SPRITE),
                custom_size: Some(Vec2::splat(WATER_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + STONE_Z_OFFSET),
        ));
    }
}

/// Wang corner mask for a tile at `(x, y)` given a same-kind predicate.
/// Returns 0 when no vertex of this tile is adjacent to a matching water
/// tile. Bit order matches `wang::wang_mask`.
fn kind_mask(water: &WaterMap, area_pos: IVec2, x: u32, y: u32, same: fn(WaterKind) -> bool) -> u8 {
    let nw = vertex_is_kind(water, area_pos, x, y + 1, same);
    let ne = vertex_is_kind(water, area_pos, x + 1, y + 1, same);
    let sw = vertex_is_kind(water, area_pos, x, y, same);
    let se = vertex_is_kind(water, area_pos, x + 1, y, same);
    wang::wang_mask(nw, ne, sw, se)
}

/// True if any of the (up to 4) tiles touching vertex `(vx, vy)` matches
/// `same`. Vertices land on the integer grid; a vertex at `(vx, vy)` is
/// shared by tiles `(vx-1, vy-1)`, `(vx, vy-1)`, `(vx-1, vy)`, `(vx, vy)`.
fn vertex_is_kind(
    water: &WaterMap,
    area_pos: IVec2,
    vx: u32,
    vy: u32,
    same: fn(WaterKind) -> bool,
) -> bool {
    for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let tile_x = i32::try_from(vx).unwrap_or(0) + dx;
        let tile_y = i32::try_from(vy).unwrap_or(0) + dy;
        if let Some(tile_key) = neighbour_key(area_pos, UVec2::new(0, 0), tile_x, tile_y) {
            if let Some(kind) = water.get(tile_key.0, tile_key.1) {
                if same(kind) {
                    return true;
                }
            }
        }
    }
    false
}

/// Wang mask for sand tiles at `(x, y)` in `area_pos`.
pub fn sand_mask(water: &WaterMap, area_pos: IVec2, x: u32, y: u32) -> u8 {
    let nw = sand_vertex(water, area_pos, x, y + 1);
    let ne = sand_vertex(water, area_pos, x + 1, y + 1);
    let sw = sand_vertex(water, area_pos, x, y);
    let se = sand_vertex(water, area_pos, x + 1, y);
    wang::wang_mask(nw, ne, sw, se)
}

fn sand_vertex(water: &WaterMap, area_pos: IVec2, vx: u32, vy: u32) -> bool {
    for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let tile_x = i32::try_from(vx).unwrap_or(0) + dx;
        let tile_y = i32::try_from(vy).unwrap_or(0) + dy;
        if let Some(tile_key) = neighbour_key(area_pos, UVec2::new(0, 0), tile_x, tile_y) {
            if water.has_sand(tile_key.0, tile_key.1) {
                return true;
            }
        }
    }
    false
}

/// Despawn every water tile on world teardown.
pub fn despawn_water(mut commands: Commands, q: Query<Entity, With<WaterTile>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Despawn every stepping stone on world teardown.
pub fn despawn_stones(mut commands: Commands, q: Query<Entity, With<SteppingStone>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
