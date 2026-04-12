use bevy::math::IVec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use models::layer::Layer;
use models::tile::Tile;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::terrain::{self, Terrain};
use crate::world::{AreaChanged, WorldMap};

/// Sprout Lands tiles are 16×16 pixels.
pub const TILE_SIZE_PX: u16 = 16;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Convert a tile-based size (width × height in tiles) to a pixel `Vec2`.
pub fn tile_size(width: Tile, height: Tile) -> Vec2 {
    Vec2::new(
        f32::from(width.0) * f32::from(TILE_SIZE_PX),
        f32::from(height.0) * f32::from(TILE_SIZE_PX),
    )
}

/// Marker for the tilemap entity of the current area.
#[derive(Component)]
pub struct Level;

/// Marker for individual tile entities so they can be bulk-despawned.
#[derive(Component)]
pub struct LevelTile;

/// Marker for neighbor tilemap entities (visual border areas).
#[derive(Component)]
pub struct NeighborLevel;

/// Marker for individual neighbor tile entities.
#[derive(Component)]
pub struct NeighborTile;

/// Cardinal offsets for the 4 neighbor areas.
const NEIGHBOR_OFFSETS: [IVec2; 4] = [
    IVec2::new(0, 1),  // North
    IVec2::new(0, -1), // South
    IVec2::new(1, 0),  // East
    IVec2::new(-1, 0), // West
];

pub fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    existing: Query<(), With<Level>>,
) {
    if !existing.is_empty() {
        return;
    }
    spawn_center_area(&mut commands, &asset_server, &world);
    spawn_neighbors(&mut commands, &asset_server, &world);
}

pub fn despawn_tilemap(
    mut commands: Commands,
    level_q: Query<Entity, With<Level>>,
    tile_q: Query<Entity, With<LevelTile>>,
    neighbor_level_q: Query<Entity, With<NeighborLevel>>,
    neighbor_tile_q: Query<Entity, With<NeighborTile>>,
) {
    despawn_all_of(&mut commands, &level_q);
    despawn_all_of(&mut commands, &tile_q);
    despawn_all_of(&mut commands, &neighbor_level_q);
    despawn_all_of(&mut commands, &neighbor_tile_q);
}

/// Despawn the current tilemap and spawn one from the new `WorldMap` current area.
pub fn respawn_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    level_q: Query<Entity, With<Level>>,
    tile_q: Query<Entity, With<LevelTile>>,
    neighbor_level_q: Query<Entity, With<NeighborLevel>>,
    neighbor_tile_q: Query<Entity, With<NeighborTile>>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }

    despawn_all_of(&mut commands, &level_q);
    despawn_all_of(&mut commands, &tile_q);
    despawn_all_of(&mut commands, &neighbor_level_q);
    despawn_all_of(&mut commands, &neighbor_tile_q);
    spawn_center_area(&mut commands, &asset_server, &world);
    spawn_neighbors(&mut commands, &asset_server, &world);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn despawn_all_of<T: Component>(commands: &mut Commands, query: &Query<Entity, With<T>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn spawn_neighbors(commands: &mut Commands, asset_server: &AssetServer, world: &WorldMap) {
    let dense_forest = Area::dense_forest();

    for offset in &NEIGHBOR_OFFSETS {
        let area_pos = world.current + *offset;
        let area = world.get_area(area_pos).unwrap_or(&dense_forest);
        spawn_neighbor_area(commands, asset_server, world, area, area_pos, *offset);
    }
}

fn spawn_center_area(commands: &mut Commands, asset_server: &AssetServer, world: &WorldMap) {
    let texture: Handle<Image> = asset_server.load("sprites/terrain/terrain_wang.webp");
    let area_pos = world.current;

    let map_size = TilemapSize {
        x: u32::from(MAP_WIDTH),
        y: u32::from(MAP_HEIGHT),
    };
    let tile_size_f32 = f32::from(TILE_SIZE_PX);
    let ts = TilemapTileSize {
        x: tile_size_f32,
        y: tile_size_f32,
    };
    let grid_size: TilemapGridSize = ts.into();

    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);
            let tile_pos = TilePos { x: xu, y: yu };
            let tile_entity = commands
                .spawn((
                    LevelTile,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(wang_tile_index(
                            xu, yu, area_pos, world,
                        )),
                        ..Default::default()
                    },
                ))
                .id();
            storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert((
        Level,
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(texture),
            tile_size: ts,
            transform: Transform::from_translation(Vec3::new(
                -(MAP_W_PX / 2.0),
                -(MAP_H_PX / 2.0),
                Layer::Tilemap.z_f32(),
            )),
            ..Default::default()
        },
    ));
}

fn spawn_neighbor_area(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area: &Area,
    area_pos: IVec2,
    grid_offset: IVec2,
) {
    let texture: Handle<Image> = asset_server.load("sprites/terrain/terrain_wang.webp");

    let map_size = TilemapSize {
        x: u32::from(MAP_WIDTH),
        y: u32::from(MAP_HEIGHT),
    };
    let tile_size_f32 = f32::from(TILE_SIZE_PX);
    let ts = TilemapTileSize {
        x: tile_size_f32,
        y: tile_size_f32,
    };
    let grid_size: TilemapGridSize = ts.into();

    #[allow(clippy::as_conversions)]
    let offset_x = -(MAP_W_PX / 2.0) + (grid_offset.x as f32) * MAP_W_PX;
    #[allow(clippy::as_conversions)]
    let offset_y = -(MAP_H_PX / 2.0) + (grid_offset.y as f32) * MAP_H_PX;

    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    let in_world = world.get_area(area_pos).is_some();

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);
            let tile_pos = TilePos { x: xu, y: yu };
            let idx = if in_world {
                wang_tile_index(xu, yu, area_pos, world)
            } else {
                wang_tile_index_local(xu, yu, area, area_pos, world)
            };
            let tile_entity = commands
                .spawn((
                    NeighborTile,
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(idx),
                        ..Default::default()
                    },
                ))
                .id();
            storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert((
        NeighborLevel,
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(texture),
            tile_size: ts,
            transform: Transform::from_translation(Vec3::new(
                offset_x,
                offset_y,
                Layer::Tilemap.z_f32(),
            )),
            ..Default::default()
        },
    ));
}

/// Wang corner tile index for a cell, consulting adjacent areas across boundaries.
fn wang_tile_index(x: u32, y: u32, area_pos: IVec2, world: &WorldMap) -> u32 {
    let lx = i32::try_from(x).expect("x fits i32");
    let ly = i32::try_from(y).expect("y fits i32");

    let at = |dx: i32, dy: i32| world.terrain_at_extended(area_pos, lx + dx, ly + dy);

    let corner = |a, b, c, d: Option<Terrain>| -> bool {
        [a, b, c, d]
            .iter()
            .filter(|t| **t == Some(Terrain::Grass))
            .count()
            >= 2
    };

    let nw = corner(at(0, 0), at(-1, 0), at(0, 1), at(-1, 1));
    let ne = corner(at(0, 0), at(1, 0), at(0, 1), at(1, 1));
    let sw = corner(at(0, 0), at(-1, 0), at(0, -1), at(-1, -1));
    let se = corner(at(0, 0), at(1, 0), at(0, -1), at(1, -1));

    let wang = terrain::wang_index(nw, ne, sw, se);
    #[allow(clippy::as_conversions)]
    terrain::WANG_TO_ATLAS[wang as usize]
}

/// Wang tile index for an area not in the world map (dense forest fallback).
/// Uses local terrain lookups and falls back to grass for out-of-bounds.
fn wang_tile_index_local(
    x: u32,
    y: u32,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) -> u32 {
    let lx = i32::try_from(x).expect("x fits i32");
    let ly = i32::try_from(y).expect("y fits i32");

    let at = |dx: i32, dy: i32| -> Option<Terrain> {
        let nx = lx + dx;
        let ny = ly + dy;
        if let (Ok(ux), Ok(uy)) = (u32::try_from(nx), u32::try_from(ny)) {
            if let Some(t) = area.terrain_at(ux, uy) {
                return Some(t);
            }
        }
        // Border tiles: check if the real neighbor has terrain there.
        world.terrain_at_extended(area_pos, nx, ny)
            .or(Some(Terrain::Grass))
    };

    let corner = |a, b, c, d: Option<Terrain>| -> bool {
        [a, b, c, d]
            .iter()
            .filter(|t| **t == Some(Terrain::Grass))
            .count()
            >= 2
    };

    let nw = corner(at(0, 0), at(-1, 0), at(0, 1), at(-1, 1));
    let ne = corner(at(0, 0), at(1, 0), at(0, 1), at(1, 1));
    let sw = corner(at(0, 0), at(-1, 0), at(0, -1), at(-1, -1));
    let se = corner(at(0, 0), at(1, 0), at(0, -1), at(1, -1));

    let wang = terrain::wang_index(nw, ne, sw, se);
    #[allow(clippy::as_conversions)]
    terrain::WANG_TO_ATLAS[wang as usize]
}
