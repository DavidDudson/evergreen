use bevy::prelude::*;
use level::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use level::plugin::TILE_SIZE_PX;
use level::spawning::area_world_offset;
use level::terrain::Terrain;
use level::world::{AreaChanged, WorldMap};
use models::speed::Speed;

use crate::animation::AnimationKind;
use crate::spawning::Player;

const RUN_SPEED: Speed = Speed(6); // 6 tiles/s
const WALK_SPEED: Speed = Speed(2); // 2 tiles/s (run / 3)

/// Grass slows the player by 30%.
const GRASS_SPEED_MULT: f32 = 0.7;

// World-space half-extents of one area (const: f32::from is not const-stable).
#[allow(clippy::as_conversions)] // const context: no From/Into available
const HALF_W: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32 / 2.0; // 256 px
#[allow(clippy::as_conversions)]
const HALF_H: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32 / 2.0; // 144 px

#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

pub fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    world: Res<WorldMap>,
    mut query: Query<(&AnimationKind, &mut Transform), With<Player>>,
) {
    let Ok((kind, mut transform)) = query.single_mut() else {
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

    let speed = match kind {
        AnimationKind::Run => RUN_SPEED,
        _ => WALK_SPEED,
    };

    let terrain_mult = terrain_speed_mult(transform.translation.truncate(), &world);
    let movement =
        direction * f32::from(speed.0) * f32::from(TILE_SIZE_PX) * terrain_mult * time.delta_secs();
    transform.translation += movement.extend(0.0);
}

/// Detect when the player has moved into a different area grid cell.
///
/// With absolute coordinates the player is never teleported -- they just keep
/// walking.  This system updates `WorldMap::current` and emits `AreaChanged`
/// when the grid cell changes.
pub fn check_area_transition(
    query: Query<&Transform, With<Player>>,
    mut world: ResMut<WorldMap>,
    mut messages: MessageWriter<AreaChanged>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    let pos = transform.translation.truncate();
    let new_area = area_grid_from_world(pos);

    if new_area == world.current {
        return;
    }

    // Determine direction for the AreaChanged message.
    let delta = new_area - world.current;
    let dir = if delta.x > 0 {
        Direction::East
    } else if delta.x < 0 {
        Direction::West
    } else if delta.y > 0 {
        Direction::North
    } else {
        Direction::South
    };

    world.transition(dir);
    messages.write(AreaChanged { direction: dir });
}

// ---------------------------------------------------------------------------

/// Convert absolute world position to area grid coordinates.
fn area_grid_from_world(pos: Vec2) -> bevy::math::IVec2 {
    #[allow(clippy::as_conversions)]
    let ax = ((pos.x + HALF_W) / MAP_W_PX).floor() as i32;
    #[allow(clippy::as_conversions)]
    let ay = ((pos.y + HALF_H) / MAP_H_PX).floor() as i32;
    bevy::math::IVec2::new(ax, ay)
}

/// Returns 0.7 when the player is standing on grass, 1.0 on dirt/road.
fn terrain_speed_mult(world_pos: Vec2, world: &WorldMap) -> f32 {
    // Convert absolute world position to area-local tile coordinates.
    let base = area_world_offset(world.current);
    let local = world_pos - base;
    let tile_size = f32::from(TILE_SIZE_PX);
    let fx = (local.x + HALF_W) / tile_size;
    let fy = (local.y + HALF_H) / tile_size;

    #[allow(clippy::as_conversions)]
    let tx = fx.floor() as i32;
    #[allow(clippy::as_conversions)]
    let ty = fy.floor() as i32;

    let terrain = u32::try_from(tx)
        .ok()
        .zip(u32::try_from(ty).ok())
        .and_then(|(x, y)| world.current_area().terrain_at(x, y));

    match terrain {
        Some(Terrain::Grass) => GRASS_SPEED_MULT,
        _ => 1.0,
    }
}
