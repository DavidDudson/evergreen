//! Detect when the player is wading through shallow water. Drives:
//!   - 50% movement-speed multiplier (consumed via [`shallow_speed_mult`]).
//!   - Sprite tint to a dark "fish-shadow" silhouette while submerged.
//!   - Periodic splash ripple sprites while moving through shallow water.
//!
//! Deep water blocks the player (via colliders in `level::water::shore`); pier
//! tiles are detected separately and walkable at full speed.

use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;
use level::area::{MAP_HEIGHT, MAP_WIDTH};
use level::plugin::TILE_SIZE_PX;
use level::spawning::area_world_offset;
use level::water::{WaterDepth, WaterMap};
use level::world::WorldMap;
use models::layer::Layer;
use models::palette;

use crate::spawning::Player;

/// Speed multiplier applied while wading through shallow water.
pub const SHALLOW_SPEED_MULT: f32 = 0.5;

/// Splash ripple cadence (one new splash every N seconds while moving).
const SPLASH_INTERVAL_S: f32 = 0.18;
/// How long a splash sprite lives before despawning.
const SPLASH_LIFETIME_S: f32 = 0.4;
const SPLASH_SIZE_PX: f32 = 14.0;
const SPLASH_SPRITE: &str = "sprites/effects/splash.webp";

/// Whether the player's center tile is shallow water.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PlayerWaterState {
    pub on_shallow: bool,
}

#[derive(Component)]
pub struct Splash {
    /// Lifetime remaining (seconds).
    pub remaining: f32,
}

/// Computes the player's tile and updates [`PlayerWaterState`]. Run before
/// movement so the speed multiplier picks up the latest value.
pub fn update_player_water_state(
    world: Res<WorldMap>,
    player: Query<&Transform, With<Player>>,
    mut state: ResMut<PlayerWaterState>,
) {
    let Ok(transform) = player.single() else {
        state.on_shallow = false;
        return;
    };
    let pos = transform.translation.truncate();
    state.on_shallow = match locate_player_tile(pos, &world) {
        Some((area, local)) => {
            // Pier overrides any underlying water -- treat as solid ground.
            !world.water.has_pier(area, local)
                && matches!(world.water.depth_at(area, local), Some(WaterDepth::Shallow))
        }
        None => false,
    };
}

/// Map a world-space position to its `(area, local)` tile coordinates.
fn locate_player_tile(pos: Vec2, world: &WorldMap) -> Option<(IVec2, UVec2)> {
    let area = world.current;
    let base = area_world_offset(area);
    let map_w_px = f32::from(MAP_WIDTH) * f32::from(TILE_SIZE_PX);
    let map_h_px = f32::from(MAP_HEIGHT) * f32::from(TILE_SIZE_PX);
    let local_x = pos.x - base.x + map_w_px / 2.0;
    let local_y = pos.y - base.y + map_h_px / 2.0;
    let tile_size = f32::from(TILE_SIZE_PX);
    #[allow(clippy::as_conversions)] // f32 -> i32: floor + range check follows
    let tx = (local_x / tile_size).floor() as i32;
    #[allow(clippy::as_conversions)] // f32 -> i32: floor + range check follows
    let ty = (local_y / tile_size).floor() as i32;
    if tx < 0 || ty < 0 {
        return None;
    }
    let ux = u32::try_from(tx).ok()?;
    let uy = u32::try_from(ty).ok()?;
    if ux >= u32::from(MAP_WIDTH) || uy >= u32::from(MAP_HEIGHT) {
        return None;
    }
    let _ = WaterMap::default; // keep `WaterMap` import alive (type used via world.water).
    Some((area, UVec2::new(ux, uy)))
}

// Player remains visible above the water surface while wading -- only
// creatures (fish shadows, etc.) render submerged. The previous
// `apply_submerged_tint` system was removed; speed and splash effects
// still convey "in water" without obscuring the player.

/// Splash spawn cadence -- fires while the player moves through shallow water.
#[derive(Resource, Default)]
pub struct SplashTimer {
    elapsed: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_splashes(
    time: Res<Time>,
    state: Res<PlayerWaterState>,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<SplashTimer>,
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
) {
    if !state.on_shallow {
        timer.elapsed = 0.0;
        return;
    }
    let Ok(transform) = player.single() else {
        return;
    };
    timer.elapsed += time.delta_secs();
    if timer.elapsed < SPLASH_INTERVAL_S {
        return;
    }
    timer.elapsed = 0.0;
    let p = transform.translation;
    commands.spawn((
        Splash {
            remaining: SPLASH_LIFETIME_S,
        },
        Sprite {
            image: asset_server.load(SPLASH_SPRITE),
            custom_size: Some(Vec2::splat(SPLASH_SIZE_PX)),
            color: palette::SPLASH_TINT,
            ..default()
        },
        Transform::from_xyz(p.x, p.y - f32::from(TILE_SIZE_PX) * 0.4, Layer::Tilemap.z_f32() + 0.6),
    ));
}

/// Tick splash lifetimes and despawn expired entries; fade them out via alpha.
pub fn tick_splashes(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Splash, &mut Sprite)>,
) {
    for (entity, mut splash, mut sprite) in &mut q {
        splash.remaining -= time.delta_secs();
        if splash.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let fade = (splash.remaining / SPLASH_LIFETIME_S).clamp(0.0, 1.0);
        let alpha = palette::SPLASH_TINT.alpha() * fade;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}
