//! Enemy entities placed by `AreaEvent::Enemy`. First pass uses static
//! south-facing sprites without animation or AI -- a placeholder for combat
//! work. Future passes will add NpcSheet animation and a wander/aggro
//! behaviour.
//!
//! Spawned per area when the area's event is `AreaEvent::Enemy { kind,
//! count }`. Position is sampled deterministically from the area seed so
//! re-entering produces the same layout.

use bevy::math::IVec2;
use bevy::prelude::*;
use models::layer::Layer;

use crate::area::{AreaEvent, EnemyKind, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::tile_hash;

#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Inset from the area's edges where enemies are allowed to stand. Keeps
/// them clear of road exits + ocean band.
const ENEMY_INSET_TILES: u32 = 4;

/// Render size for enemy sprites (square, in pixels).
const ENEMY_SPRITE_SIZE_PX: f32 = 24.0;

#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy {
    pub kind: EnemyKind,
}

/// Spawn the area's enemies according to the area's event.
pub fn spawn_area_enemies(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &crate::area::Area,
    area_pos: IVec2,
) {
    let AreaEvent::Enemy { kind, count } = area.event else {
        return;
    };
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223))
        .wrapping_add(0xE4E_017);

    let inner_w = u32::from(MAP_WIDTH).saturating_sub(ENEMY_INSET_TILES * 2);
    let inner_h = u32::from(MAP_HEIGHT).saturating_sub(ENEMY_INSET_TILES * 2);
    if inner_w == 0 || inner_h == 0 {
        return;
    }
    for i in 0..count {
        let salt = area_seed.wrapping_add(u32::from(i).wrapping_mul(2_166_136_261));
        let h = tile_hash(u32::from(i), 0, salt);
        let h_lo = u32::try_from(h & 0xFFFF_FFFF).unwrap_or(0);
        let h_hi = u32::try_from((h >> 8) & 0xFFFF_FFFF).unwrap_or(0);
        let tx = h_lo % inner_w + ENEMY_INSET_TILES;
        let ty = h_hi % inner_h + ENEMY_INSET_TILES;
        let world_x = base_offset_x + f32::from(u16::try_from(tx).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y + f32::from(u16::try_from(ty).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        commands.spawn((
            Enemy { kind },
            Sprite {
                image: asset_server.load(sprite_path(kind)),
                custom_size: Some(Vec2::splat(ENEMY_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::World.z_f32() - world_y * 0.001),
        ));
    }
}

fn sprite_path(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::PurpleSlime => "sprites/enemies/slime.webp",
        EnemyKind::DiseasedFox => "sprites/enemies/fox.webp",
        EnemyKind::DiseasedDeer => "sprites/enemies/deer.webp",
        EnemyKind::DiseasedBear => "sprites/enemies/bear.webp",
    }
}

/// Despawn every enemy on world teardown.
pub fn despawn_enemies(mut commands: Commands, q: Query<Entity, With<Enemy>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
