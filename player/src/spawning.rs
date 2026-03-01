use bevy::prelude::*;
use level::plugin::tile_size;
use models::layer::Layer;
use models::palette;
use models::speed::Speed;
use models::tile::Tile;

const PLAYER_WIDTH: Tile = Tile(1);
const PLAYER_HEIGHT: Tile = Tile(2);
const PLAYER_SPEED: Speed = Speed(6); // 30ft/s = 6 tiles/s

#[derive(Component)]
#[require(Speed)]
pub struct Player;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Player,
        PLAYER_SPEED,
        Sprite {
            color: palette::PLAYER,
            custom_size: Some(tile_size(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Layer::Player.z_f32()),
    ));
}

pub fn despawn(mut commands: Commands, query: Query<Entity, With<Player>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
