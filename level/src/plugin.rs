use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use models::game_states::GameState;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: u32 = 40;
pub const MAP_HEIGHT: u32 = 25;

#[derive(Component)]
pub struct Level;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), spawn_tilemap)
            .add_systems(
                OnExit(GameState::Playing),
                despawn_tilemap.run_if(not(in_state(GameState::Paused))),
            );
    }
}

fn spawn_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize {
        x: MAP_WIDTH,
        y: MAP_HEIGHT,
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size: TilemapGridSize = tile_size.into();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Center the tilemap so (0,0) world is roughly the map center
    let offset_x = -(MAP_WIDTH as f32 * TILE_SIZE) / 2.0;
    let offset_y = -(MAP_HEIGHT as f32 * TILE_SIZE) / 2.0;

    commands.entity(tilemap_entity).insert((
        Level,
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: Transform::from_translation(Vec3::new(offset_x, offset_y, 0.0)),
            ..Default::default()
        },
    ));
}

fn despawn_tilemap(mut commands: Commands, query: Query<Entity, With<Level>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
