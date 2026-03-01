use bevy::prelude::*;
use level::plugin::TILE_SIZE_PX;
use models::speed::Speed;

use crate::spawning::Player;

pub fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Speed, &mut Transform), With<Player>>,
) {
    let Ok((speed, mut transform)) = query.single_mut() else {
        return;
    };

    let direction = [
        (KeyCode::KeyW, Vec2::Y),
        (KeyCode::ArrowUp, Vec2::Y),
        (KeyCode::KeyS, Vec2::NEG_Y),
        (KeyCode::ArrowDown, Vec2::NEG_Y),
        (KeyCode::KeyA, Vec2::NEG_X),
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::KeyD, Vec2::X),
        (KeyCode::ArrowRight, Vec2::X),
    ]
    .iter()
    .filter(|(key, _)| keyboard.pressed(*key))
    .map(|(_, dir)| *dir)
    .sum::<Vec2>();

    let direction = if direction != Vec2::ZERO {
        direction.normalize()
    } else {
        return;
    };

    let movement = direction * f32::from(speed.0) * f32::from(TILE_SIZE_PX) * time.delta_secs();
    transform.translation += movement.extend(0.0);
}
