use bevy::prelude::*;

pub fn despawn_all<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
