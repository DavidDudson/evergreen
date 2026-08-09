//! Enemy entities placed by `AreaEvent::Enemy`. Animated via the existing
//! NPC sprite-sheet pipeline (`models::npc_anim`): each enemy carries an
//! `NpcSheet`, `NpcAnimKind`, `NpcAnimFrame`, `NpcAnimTimer`, and
//! `NpcFacing`, so the shared `npc_anim::advance_npc_frame` system animates
//! them automatically.
//!
//! Each enemy also carries an [`EnemyWander`] component that drives a
//! free-roaming idle/walk cycle. Unlike [`crate::npc_wander::NpcWander`],
//! enemies are not bound to their spawn point -- they can cross area
//! boundaries (zones) freely.
//!
//! Sheets are 4-row x 8-col (rows = facing south/east/north/west, cols 0-3
//! = idle frames, 4-7 = walk frames). 32x32 pixel frames.
//!
//! Spawned per area when the area's event is `AreaEvent::Enemy { kind,
//! count }`. Position is sampled deterministically from the area seed so
//! re-entering produces the same layout.

use bevy::math::IVec2;
use bevy::prelude::*;
use models::layer::Layer;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};
use models::scenery::SceneryCollider;
use rand::RngExt;

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
/// Enemy placeholder sheets pack a single 8-frame cycle per facing row
/// (idle breathing on front/back rows, walk cycle on side rows) rather
/// than the usual split. Both anim kinds use the full row -- see
/// `NpcSheet::walk_col_start` set to 0 below.
const ENEMY_ANIM_FRAMES: usize = 8;
const Y_SORT_SCALE: f32 = 0.001;

/// Collider half-extent for an enemy. Matches the NPC collider so the
/// player can't walk through them.
const ENEMY_COLLIDER_HALF: Vec2 = Vec2::new(7.0, 7.0);

/// Enemy walk speed in pixels per second. Slightly slower than NPC
/// wander (16px/s) so the player can outrun them.
const ENEMY_WALK_SPEED_PX: f32 = 12.0;
/// Idle dwell window when standing between walks.
const ENEMY_IDLE_MIN_SECS: f32 = 1.5;
const ENEMY_IDLE_MAX_SECS: f32 = 4.0;
/// Walk-leg duration window. Longer than NPC wander so enemies actually
/// cover ground (and can traverse a full 32-tile area in ~2 legs).
const ENEMY_WALK_MIN_SECS: f32 = 2.0;
const ENEMY_WALK_MAX_SECS: f32 = 6.0;

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
        let world_x =
            base_offset_x + f32::from(u16::try_from(tx).unwrap_or(0)) * tile_px + tile_px / 2.0;
        let world_y =
            base_offset_y + f32::from(u16::try_from(ty).unwrap_or(0)) * tile_px + tile_px / 2.0;
        commands.spawn((
            Enemy { kind },
            EnemyWander::default(),
            Sprite {
                image: asset_server.load(sprite_path(kind)),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::splat(ENEMY_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(
                world_x,
                world_y,
                Layer::World.z_f32() - world_y * Y_SORT_SCALE,
            ),
            SceneryCollider {
                half_extents: ENEMY_COLLIDER_HALF,
                center_offset: Vec2::ZERO,
            },
            NpcFacing::default(),
            NpcAnimKind::Idle,
            NpcAnimFrame::default(),
            NpcAnimTimer(Timer::from_seconds(
                1.0 / NpcAnimKind::Idle.fps(),
                TimerMode::Repeating,
            )),
            NpcSheet {
                idle_frames: ENEMY_ANIM_FRAMES,
                walk_frames: ENEMY_ANIM_FRAMES,
                cols: usize::try_from(ENEMY_SHEET_COLS).unwrap_or(8),
                walk_col_start: 0,
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

// ---------------------------------------------------------------------------
// Wander
// ---------------------------------------------------------------------------

/// Free-roaming idle/walk cycle for enemies. Unlike `NpcWander`, has no
/// origin / radius constraint -- enemies can cross zone boundaries.
#[derive(Component)]
pub struct EnemyWander {
    state: EnemyWanderState,
    timer: Timer,
}

enum EnemyWanderState {
    Idle,
    /// Unit direction vector.
    Walking(Vec2),
}

impl Default for EnemyWander {
    fn default() -> Self {
        Self {
            state: EnemyWanderState::Idle,
            timer: Timer::from_seconds(ENEMY_IDLE_MIN_SECS, TimerMode::Once),
        }
    }
}

/// Drive every enemy's wander cycle: pick a random direction, walk for a
/// while, idle, repeat. Updates facing, anim kind, transform, and y-sort z.
pub fn wander_enemies(
    time: Res<Time>,
    mut q: Query<(
        &mut Transform,
        &mut EnemyWander,
        &mut NpcFacing,
        &mut NpcAnimKind,
    )>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (mut tf, mut wander, mut facing, mut anim_kind) in &mut q {
        wander.timer.tick(time.delta());

        if let EnemyWanderState::Walking(dir) = wander.state {
            tf.translation.x += dir.x * ENEMY_WALK_SPEED_PX * dt;
            tf.translation.y += dir.y * ENEMY_WALK_SPEED_PX * dt;
            tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
        }

        if !wander.timer.is_finished() {
            continue;
        }

        match wander.state {
            EnemyWanderState::Idle => {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let dir = Vec2::new(angle.cos(), angle.sin());
                *facing = NpcFacing::from_vec2(dir);
                *anim_kind = NpcAnimKind::Walk;
                wander.state = EnemyWanderState::Walking(dir);
                let dur = rng.random_range(ENEMY_WALK_MIN_SECS..ENEMY_WALK_MAX_SECS);
                wander.timer = Timer::from_seconds(dur, TimerMode::Once);
            }
            EnemyWanderState::Walking(_) => {
                *anim_kind = NpcAnimKind::Idle;
                wander.state = EnemyWanderState::Idle;
                let dur = rng.random_range(ENEMY_IDLE_MIN_SECS..ENEMY_IDLE_MAX_SECS);
                wander.timer = Timer::from_seconds(dur, TimerMode::Once);
            }
        }
    }
}
