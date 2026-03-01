use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use models::layer::Layer;
use models::tile::Tile;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::terrain::{self, Terrain};
use crate::world::{AreaChanged, WorldMap};

/// Sprout Lands tiles are 16×16 pixels.
pub const TILE_SIZE_PX: u16 = 16;

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

pub fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
) {
    spawn_area(&mut commands, &asset_server, &world);
}

pub fn despawn_tilemap(
    mut commands: Commands,
    level_q: Query<Entity, With<Level>>,
    tile_q: Query<Entity, With<LevelTile>>,
) {
    despawn_area(&mut commands, &level_q, &tile_q);
}

/// Despawn the current tilemap and spawn one from the new `WorldMap` current area.
pub fn respawn_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    level_q: Query<Entity, With<Level>>,
    tile_q: Query<Entity, With<LevelTile>>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }

    despawn_area(&mut commands, &level_q, &tile_q);
    spawn_area(&mut commands, &asset_server, &world);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn despawn_area(
    commands: &mut Commands,
    level_q: &Query<Entity, With<Level>>,
    tile_q: &Query<Entity, With<LevelTile>>,
) {
    for entity in level_q {
        commands.entity(entity).despawn();
    }
    for entity in tile_q {
        commands.entity(entity).despawn();
    }
}

fn spawn_area(commands: &mut Commands, asset_server: &AssetServer, world: &WorldMap) {
    let texture: Handle<Image> = asset_server.load("terrain_wang.png");
    let area_pos = world.current;

    let map_size = TilemapSize {
        x: u32::from(MAP_WIDTH),
        y: u32::from(MAP_HEIGHT),
    };
    let tile_size_f32 = f32::from(TILE_SIZE_PX);
    let tile_size = TilemapTileSize {
        x: tile_size_f32,
        y: tile_size_f32,
    };
    let grid_size: TilemapGridSize = tile_size.into();

    let offset_x = -(f32::from(MAP_WIDTH) * tile_size_f32) / 2.0;
    let offset_y = -(f32::from(MAP_HEIGHT) * tile_size_f32) / 2.0;

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
            tile_size,
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
///
/// Each corner is grass if ≥2 of the 4 tiles sharing that vertex are grass.
/// Grass wins on a 2-vs-2 tie. Bit ordering: NW=8, NE=4, SW=2, SE=1.
fn wang_tile_index(x: u32, y: u32, area_pos: bevy::math::IVec2, world: &WorldMap) -> u32 {
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
    terrain::WANG_TO_ATLAS[wang as usize]
}
