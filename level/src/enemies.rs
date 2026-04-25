//! Enemy entities placed by `AreaEvent::Enemy`. Animated via the existing
//! NPC sprite-sheet pipeline (`models::npc_anim`): each enemy carries an
//! `NpcSheet`, `NpcAnimKind::Walk`, `NpcAnimFrame`, `NpcAnimTimer`, and
//! `NpcFacing`, so the shared `npc_anim::advance_npc_frame` system animates
//! them automatically.
//!
//! Sheets are 4-row x 8-col (rows = facing south/east/north/west, cols 0-3
//! and 4-7 = walk frames repeated, so idle and walk look identical for the
//! placeholder enemies). 32x32 pixel frames.
//!
//! Spawned per area when the area's event is `AreaEvent::Enemy { kind,
//! count }`. Position is sampled deterministically from the area seed so
//! re-entering produces the same layout.

use bevy::math::IVec2;
use bevy::prelude::*;
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};

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
const ENEMY_SPRITE_SIZE_PX: f32 = 28.0;

const ENEMY_SHEET_FRAME_PX: u32 = 32;
const ENEMY_SHEET_COLS: u32 = 8;
const ENEMY_SHEET_ROWS: u32 = 4;
const ENEMY_IDLE_FRAMES: usize = 4;
const ENEMY_WALK_FRAMES: usize = 4;
const Y_SORT_SCALE: f32 = 0.001;

#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy {
    pub kind: EnemyKind,
}

/// Spawn the area's enemies according to the area's event.
pub fn spawn_area_enemies(
    commands: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
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

    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(ENEMY_SHEET_FRAME_PX),
        ENEMY_SHEET_COLS,
        ENEMY_SHEET_ROWS,
        None,
        None,
    ));

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
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::splat(ENEMY_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::World.z_f32() - world_y * Y_SORT_SCALE),
            NpcFacing::default(),
            NpcAnimKind::Walk,
            NpcAnimFrame::default(),
            NpcAnimTimer(Timer::from_seconds(1.0 / NpcAnimKind::Walk.fps(), TimerMode::Repeating)),
            NpcSheet {
                idle_frames: ENEMY_IDLE_FRAMES,
                walk_frames: ENEMY_WALK_FRAMES,
                cols: usize::try_from(ENEMY_SHEET_COLS).unwrap_or(8),
            },
        ));
    }
}

fn sprite_path(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::PurpleSlime => "sprites/enemies/slime_sheet.webp",
        EnemyKind::DiseasedFox => "sprites/enemies/fox_sheet.webp",
        EnemyKind::DiseasedDeer => "sprites/enemies/deer_sheet.webp",
        EnemyKind::DiseasedBear => "sprites/enemies/bear_sheet.webp",
    }
}

/// Despawn every enemy on world teardown.
pub fn despawn_enemies(mut commands: Commands, q: Query<Entity, With<Enemy>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}
