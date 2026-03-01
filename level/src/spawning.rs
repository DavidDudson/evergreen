use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::terrain::Terrain;
use crate::tile_map::{self, MAP_HEIGHT, MAP_WIDTH};

/// Sprout Lands tiles are 16x16 pixels.
pub const TILE_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct Level;

pub fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("terrain.png");

    let map_size = TilemapSize {
        x: u32::from(MAP_WIDTH),
        y: u32::from(MAP_HEIGHT),
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size: TilemapGridSize = tile_size.into();

    let offset_x = -(f32::from(MAP_WIDTH) * TILE_SIZE) / 2.0;
    let offset_y = -(f32::from(MAP_HEIGHT) * TILE_SIZE) / 2.0;

    let tilemap_entity = commands.spawn_empty().id();
    let mut storage = TileStorage::empty(map_size);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let row = usize::from(MAP_HEIGHT - 1 - y);
            let terrain: Terrain =
                tile_map::terrain_at_row(row, usize::from(x)).expect("invalid tile map character");
            let x = u32::from(x);
            let y = u32::from(y);
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(tile_map::tile_index(x, y, terrain)),
                    ..Default::default()
                })
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
            transform: Transform::from_translation(Vec3::new(offset_x, offset_y, 0.0)),
            ..Default::default()
        },
    ));
}

pub fn despawn_tilemap(mut commands: Commands, query: Query<Entity, With<Level>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
