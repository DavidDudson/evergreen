use bevy::prelude::*;
use models::speed::Speed;

#[derive(Component)]
#[require(Speed)]
pub struct Player;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Player,
        Speed(150.0),
        Sprite {
            color: Color::srgb(0.6, 0.2, 0.8),
            custom_size: Some(Vec2::new(16.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
}

pub fn despawn(mut commands: Commands, query: Query<Entity, With<Player>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
