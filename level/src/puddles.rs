//! Rain-triggered puddles + hot-spring steam emitters. Puddles spawn while
//! `WeatherKind::Rain` is active and fade out when the weather shifts; steam
//! rises continuously above every hot-spring water tile.

use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use models::layer::Layer;
use models::palette;
use models::time::GameClock;
use models::weather::{WeatherKind, WeatherState};

use crate::water::{WaterKind, WaterTile};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PUDDLE_SPRITE: &str = "sprites/scenery/ponds/puddle.webp";
const STEAM_SPRITE: &str = "sprites/particles/steam.webp";

/// Puddle dimensions.
const PUDDLE_SIZE_PX: f32 = 18.0;
/// Fade-out duration once the rain stops.
const PUDDLE_FADE_SECS: f32 = 6.0;
/// Puddles accumulate while raining; capped so the screen doesn't fill up.
const PUDDLE_MAX: usize = 18;
/// Seconds between puddle-spawn attempts while raining.
const PUDDLE_SPAWN_INTERVAL_SECS: f32 = 1.8;
/// Half-extent of the camera-relative zone puddles may spawn in.
const PUDDLE_ZONE_HALF_W_PX: f32 = 260.0;
const PUDDLE_ZONE_HALF_H_PX: f32 = 150.0;

/// Steam emission rate per hot-spring tile, in puffs per second.
const STEAM_PER_TILE_PER_SEC: f32 = 0.4;
/// Steam lifetime.
const STEAM_LIFETIME_SECS: f32 = 3.0;
/// Steam rise speed (positive Y, pixels/sec).
const STEAM_RISE_SPEED_PX: f32 = 14.0;
/// Steam horizontal drift range.
const STEAM_DRIFT_SPEED_PX: f32 = 4.0;
/// Steam visual size.
const STEAM_SIZE_PX: f32 = 16.0;

// ---------------------------------------------------------------------------
// Components & resources
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Puddle {
    pub fade_remaining: Option<Timer>,
}

#[derive(Component)]
pub struct SteamParticle {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

#[derive(Resource)]
pub struct PuddleSpawnTimer(Timer);

impl Default for PuddleSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            PUDDLE_SPAWN_INTERVAL_SECS,
            TimerMode::Repeating,
        ))
    }
}

#[derive(Resource, Default)]
pub struct SteamAccumulator(pub f32);

// ---------------------------------------------------------------------------
// Systems: puddles
// ---------------------------------------------------------------------------

pub fn spawn_puddles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weather: Res<WeatherState>,
    time: Res<Time>,
    mut timer: ResMut<PuddleSpawnTimer>,
    camera_q: Query<&Transform, With<Camera2d>>,
    puddle_q: Query<&Puddle>,
) {
    if weather.current != WeatherKind::Rain && weather.current != WeatherKind::Storm {
        return;
    }
    if puddle_q.iter().count() >= PUDDLE_MAX {
        return;
    }
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam = cam_tf.translation.truncate();
    let seed = f32_to_seed(time.elapsed_secs());
    let offset_x = hash_range(seed, PUDDLE_ZONE_HALF_W_PX);
    let offset_y = hash_range(seed.wrapping_add(1), PUDDLE_ZONE_HALF_H_PX);
    let pos = Vec3::new(
        cam.x + offset_x,
        cam.y + offset_y,
        Layer::Tilemap.z_f32() + 0.4,
    );
    commands.spawn((
        Puddle {
            fade_remaining: None,
        },
        Sprite {
            image: asset_server.load(PUDDLE_SPRITE),
            custom_size: Some(Vec2::splat(PUDDLE_SIZE_PX)),
            color: Color::WHITE,
            ..default()
        },
        Transform::from_translation(pos),
    ));
}

pub fn fade_puddles_when_clear(
    mut commands: Commands,
    weather: Res<WeatherState>,
    time: Res<Time>,
    mut puddles: Query<(Entity, &mut Puddle, &mut Sprite)>,
) {
    let raining = matches!(weather.current, WeatherKind::Rain | WeatherKind::Storm);
    for (entity, mut puddle, mut sprite) in &mut puddles {
        match (&mut puddle.fade_remaining, raining) {
            (None, false) => {
                puddle.fade_remaining =
                    Some(Timer::from_seconds(PUDDLE_FADE_SECS, TimerMode::Once));
            }
            (Some(timer), true) => {
                // Rain resumed before fade completed -- restore full alpha.
                timer.reset();
                puddle.fade_remaining = None;
                sprite.color = sprite.color.with_alpha(1.0);
            }
            (Some(timer), false) => {
                timer.tick(time.delta());
                let remaining = 1.0 - timer.fraction();
                sprite.color = sprite.color.with_alpha(remaining);
                if timer.is_finished() {
                    commands.entity(entity).despawn();
                }
            }
            (None, true) => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Systems: hot-spring steam
// ---------------------------------------------------------------------------

pub fn spawn_hotspring_steam(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    clock: Res<GameClock>,
    mut accumulator: ResMut<SteamAccumulator>,
    hot_springs: Query<(&WaterTile, &Transform)>,
) {
    let dt = time.delta_secs();
    accumulator.0 += dt;
    let spawn_every = 1.0 / STEAM_PER_TILE_PER_SEC;
    if accumulator.0 < spawn_every {
        return;
    }
    accumulator.0 -= spawn_every;

    // Rain dampens steam; skip spawning entirely during rain/storm.
    // (Clock parameter reserved for time-of-day modulation later.)
    let _ = clock;

    let seed = f32_to_seed(time.elapsed_secs()).wrapping_add(0x57_EA_70);
    let tiles: Vec<&Transform> = hot_springs
        .iter()
        .filter(|(t, _)| t.kind == WaterKind::HotSpring)
        .map(|(_, tf)| tf)
        .collect();
    if tiles.is_empty() {
        return;
    }

    let idx = usize::try_from(seed).unwrap_or(0) % tiles.len();
    let tf = tiles[idx];
    let jitter_x = hash_range(seed.wrapping_add(5), 4.0);
    let drift_x = hash_range(seed.wrapping_add(7), STEAM_DRIFT_SPEED_PX);
    let pos = tf.translation + Vec3::new(jitter_x, 2.0, Layer::Weather.z_f32() - 2.0);
    let tint = palette::FOG.with_alpha(0.55);

    commands.spawn((
        SteamParticle {
            velocity: Vec2::new(drift_x, STEAM_RISE_SPEED_PX),
            lifetime: Timer::from_seconds(STEAM_LIFETIME_SECS, TimerMode::Once),
        },
        Sprite {
            image: asset_server.load(STEAM_SPRITE),
            custom_size: Some(Vec2::splat(STEAM_SIZE_PX)),
            color: tint,
            ..default()
        },
        Transform::from_translation(Vec3::new(pos.x, pos.y, Layer::Weather.z_f32() - 2.0)),
    ));
}

pub fn update_steam(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut SteamParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut tf, mut sprite) in &mut particles {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;
        let remaining = 1.0 - particle.lifetime.fraction();
        sprite.color = sprite.color.with_alpha(remaining * 0.6);
    }
}

// ---------------------------------------------------------------------------
// Teardown
// ---------------------------------------------------------------------------

pub fn despawn_puddles(mut commands: Commands, q: Query<Entity, With<Puddle>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn despawn_steam(mut commands: Commands, q: Query<Entity, With<SteamParticle>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(clippy::as_conversions)]
fn f32_to_seed(value: f32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

fn hash_range(seed: u32, range: f32) -> f32 {
    let h = seed.wrapping_mul(374_761_393).wrapping_add(668_265_263);
    let h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    let h = h ^ (h >> 16);
    #[allow(clippy::as_conversions)]
    let frac = (h % 10000) as f32 / 10000.0;
    (frac - 0.5) * 2.0 * range
}
