//! Level exit -- a marker entity spawned in the exit dead-end area.

use bevy::math::IVec2;
use bevy::prelude::*;
use models::layer::Layer;

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{TILE_SIZE_PX, area_world_offset};
use crate::world::WorldMap;

/// Size of the exit marker sprite (pixels).
const EXIT_SPRITE_SIZE_PX: f32 = 24.0;

// Path intersection centre tile.
const PATH_CENTER_X: u16 = 15;
const PATH_CENTER_Y: u16 = 8;

/// Marker for the level exit entity.
#[derive(Component)]
pub struct LevelExit;

/// Spawn the exit marker in the exit area.
pub fn spawn_exit(
    mut commands: Commands,
    world: Res<WorldMap>,
    existing: Query<(), With<LevelExit>>,
) {
    if !existing.is_empty() {
        return;
    }
    let pos = exit_world_pos(world.exit_area);
    commands.spawn((
        LevelExit,
        Sprite {
            color: Color::srgba(0.9, 0.8, 0.2, 0.9),
            custom_size: Some(Vec2::splat(EXIT_SPRITE_SIZE_PX)),
            ..default()
        },
        Transform::from_translation(pos),
    ));
}

/// Despawn exit entity on game exit.
pub fn despawn_exit(mut commands: Commands, q: Query<Entity, With<LevelExit>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

fn exit_world_pos(area_pos: IVec2) -> Vec3 {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let offset_x = base.x - (f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = base.y - (f32::from(MAP_HEIGHT) * tile_px) / 2.0;
    Vec3::new(
        offset_x + f32::from(PATH_CENTER_X) * tile_px + tile_px / 2.0,
        offset_y + f32::from(PATH_CENTER_Y) * tile_px + tile_px / 2.0,
        Layer::Npc.z_f32(),
    )
}
