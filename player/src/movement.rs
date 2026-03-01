use bevy::prelude::*;
use level::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use level::plugin::TILE_SIZE_PX;
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

// How far inside the opposite edge the player reappears after a transition.
#[allow(clippy::as_conversions)]
const MARGIN_PX: f32 = TILE_SIZE_PX as f32 * 2.0; // 32 px

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
    let movement = direction
        * f32::from(speed.0)
        * f32::from(TILE_SIZE_PX)
        * terrain_mult
        * time.delta_secs();
    transform.translation += movement.extend(0.0);
}

/// Detect when the player walks off an edge and trigger an area transition.
pub fn check_area_transition(
    mut query: Query<&mut Transform, With<Player>>,
    mut world: ResMut<WorldMap>,
    mut messages: MessageWriter<AreaChanged>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let pos = transform.translation.truncate();

    let crossed = if pos.x > HALF_W {
        Some((Direction::East, Vec2::new(-HALF_W + MARGIN_PX, pos.y)))
    } else if pos.x < -HALF_W {
        Some((Direction::West, Vec2::new(HALF_W - MARGIN_PX, pos.y)))
    } else if pos.y > HALF_H {
        Some((Direction::North, Vec2::new(pos.x, -HALF_H + MARGIN_PX)))
    } else if pos.y < -HALF_H {
        Some((Direction::South, Vec2::new(pos.x, HALF_H - MARGIN_PX)))
    } else {
        None
    };

    let Some((dir, new_pos)) = crossed else {
        return;
    };

    // Only transition if the current area actually has an exit in that direction.
    if !world.current_area().exits.contains(&dir) {
        // Clamp the player back inside the area boundary.
        transform.translation.x = transform.translation.x.clamp(-HALF_W, HALF_W);
        transform.translation.y = transform.translation.y.clamp(-HALF_H, HALF_H);
        return;
    }

    world.transition(dir);
    transform.translation = new_pos.extend(transform.translation.z);
    messages.write(AreaChanged);
}

// ---------------------------------------------------------------------------

/// Returns 0.5 when the player is standing on grass, 1.0 on dirt/road.
fn terrain_speed_mult(world_pos: Vec2, world: &WorldMap) -> f32 {
    // Convert world position to tile coordinates within the current area.
    // Tile (tx, ty): tile_x = floor((world_x + HALF_W) / TILE_SIZE_PX)
    let tile_size = f32::from(TILE_SIZE_PX);
    let fx = (world_pos.x + HALF_W) / tile_size;
    let fy = (world_pos.y + HALF_H) / tile_size;

    // f32 → i32: values are bounded by MAP_WIDTH/HEIGHT (≤ 32/18), always fits.
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
