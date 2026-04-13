use std::collections::HashSet;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use models::decoration::Biome;
use models::layer::Layer;
use models::tile::Tile;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::blending;
use crate::creatures;
use crate::decorations;
use crate::grass;
use crate::npcs;
use crate::scenery;
use crate::terrain::{self, Terrain};
use crate::world::{AreaChanged, WorldMap};

/// Sprout Lands tiles are 16x16 pixels.
pub const TILE_SIZE_PX: u16 = 16;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Convert a tile-based size (width x height in tiles) to a pixel `Vec2`.
pub fn tile_size(width: Tile, height: Tile) -> Vec2 {
    Vec2::new(
        f32::from(width.0) * f32::from(TILE_SIZE_PX),
        f32::from(height.0) * f32::from(TILE_SIZE_PX),
    )
}

/// World-space pixel offset for the centre of an area at `grid_pos`.
pub fn area_world_offset(grid_pos: IVec2) -> Vec2 {
    #[allow(clippy::as_conversions)]
    Vec2::new(grid_pos.x as f32 * MAP_W_PX, grid_pos.y as f32 * MAP_H_PX)
}

// ---------------------------------------------------------------------------
// Components & resources
// ---------------------------------------------------------------------------

/// Marker for any area tilemap entity.
#[derive(Component)]
pub struct AreaTilemap;

/// Marker for individual tile entities (for bulk despawn).
#[derive(Component)]
pub struct AreaTile;

/// Tracks which areas have had their entities spawned.
#[derive(Resource, Default)]
pub struct SpawnedAreas(pub HashSet<IVec2>);

/// Cardinal offsets for the 4 neighbor areas.
const NEIGHBOR_OFFSETS: [IVec2; 4] = [
    IVec2::new(0, 1),  // North
    IVec2::new(0, -1), // South
    IVec2::new(1, 0),  // East
    IVec2::new(-1, 0), // West
];

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawn the current area and its neighbors on game start.
pub fn spawn_initial_areas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    mut spawned: ResMut<SpawnedAreas>,
) {
    let current = world.current;
    ensure_area_spawned(
        &mut commands,
        &asset_server,
        &mut atlas_layouts,
        &world,
        current,
        &mut spawned,
    );
    for offset in &NEIGHBOR_OFFSETS {
        let pos = current + *offset;
        ensure_area_spawned(
            &mut commands,
            &asset_server,
            &mut atlas_layouts,
            &world,
            pos,
            &mut spawned,
        );
    }
}

/// On area change, spawn any new neighbor areas that haven't been spawned yet.
pub fn ensure_neighbors_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<WorldMap>,
    mut spawned: ResMut<SpawnedAreas>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }
    let current = world.current;
    ensure_area_spawned(
        &mut commands,
        &asset_server,
        &mut atlas_layouts,
        &world,
        current,
        &mut spawned,
    );
    for offset in &NEIGHBOR_OFFSETS {
        let pos = current + *offset;
        ensure_area_spawned(
            &mut commands,
            &asset_server,
            &mut atlas_layouts,
            &world,
            pos,
            &mut spawned,
        );
    }
}

/// Despawn all area entities on game exit.
pub fn despawn_all_areas(
    mut commands: Commands,
    tilemaps: Query<Entity, With<AreaTilemap>>,
    tiles: Query<Entity, With<AreaTile>>,
    mut spawned: ResMut<SpawnedAreas>,
) {
    for entity in &tilemaps {
        commands.entity(entity).despawn();
    }
    for entity in &tiles {
        commands.entity(entity).despawn();
    }
    spawned.0.clear();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ensure_area_spawned(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    world: &WorldMap,
    area_pos: IVec2,
    spawned: &mut SpawnedAreas,
) {
    if spawned.0.contains(&area_pos) {
        return;
    }
    let dense_forest = Area::dense_forest();
    let area = world.get_area(area_pos).unwrap_or(&dense_forest);
    spawn_area_tilemap(commands, asset_server, world, area, area_pos);
    scenery::spawn_area_scenery_at(commands, asset_server, area, area_pos, world);
    decorations::spawn_area_decorations(commands, asset_server, area, area_pos, world);
    grass::spawn_area_grass(commands, asset_server, area, area_pos, world);
    creatures::spawn_area_creatures(commands, asset_server, area, area_pos, world);
    npcs::spawn_npc_for_area(commands, asset_server, atlas_layouts, area, area_pos);
    spawned.0.insert(area_pos);
}

fn spawn_area_tilemap(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldMap,
    area: &Area,
    area_pos: IVec2,
) {
    let center_x = u32::from(MAP_WIDTH) / 2;
    let center_y = u32::from(MAP_HEIGHT) / 2;
    let effective_alignment =
        blending::blended_alignment(area.alignment, center_x, center_y, area_pos, world);
    let texture: Handle<Image> = asset_server.load(terrain_tileset_path(effective_alignment));
    let base = area_world_offset(area_pos);

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
                    AreaTile,
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
        AreaTilemap,
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(texture),
            tile_size: ts,
            transform: Transform::from_translation(Vec3::new(
                base.x - (MAP_W_PX / 2.0),
                base.y - (MAP_H_PX / 2.0),
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
fn wang_tile_index_local(x: u32, y: u32, area: &Area, area_pos: IVec2, world: &WorldMap) -> u32 {
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
        world
            .terrain_at_extended(area_pos, nx, ny)
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

/// Returns the Wang tileset asset path for the given area alignment.
fn terrain_tileset_path(alignment: u8) -> &'static str {
    match Biome::from_alignment(alignment) {
        Biome::City => "sprites/terrain/terrain_wang_city.webp",
        Biome::Greenwood => "sprites/terrain/terrain_wang.webp",
        Biome::Darkwood => "sprites/terrain/terrain_wang_darkwood.webp",
    }
}
