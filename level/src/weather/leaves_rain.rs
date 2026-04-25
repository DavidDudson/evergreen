//! Spawning leaves + raindrops -- the asset-driven particles that respond to
//! the active weather state.

use bevy::prelude::*;
use models::layer::Layer;
use models::weather::{ParticleVariant, WeatherKind, WeatherParticle, WeatherState};
use models::wind::WindDirection;

use crate::biome_registry::BiomeRegistry;
use crate::world::WorldMap;

use super::helpers::{
    f32_to_seed, fractional_to_count, hash_f32, hash_frac, DEFAULT_ALIGNMENT, VIEWPORT_HALF_H_PX,
    VIEWPORT_HALF_W_PX,
};

/// Leaf particles spawned per second during Breezy weather.
const BREEZY_LEAVES_PER_SEC: f32 = 2.0;
/// Leaf particles spawned per second during Windy weather.
const WINDY_LEAVES_PER_SEC: f32 = 6.0;
/// Leaf particles spawned per second during Storm weather.
const STORM_LEAVES_PER_SEC: f32 = 8.0;

/// Rain particles spawned per second during Rain weather.
const RAIN_DROPS_PER_SEC: f32 = 15.0;
/// Rain particles spawned per second during Storm weather.
const STORM_DROPS_PER_SEC: f32 = 30.0;

/// Leaf horizontal speed range (pixels/sec).
const LEAF_SPEED_MIN_PX: f32 = 30.0;
const LEAF_SPEED_MAX_PX: f32 = 80.0;
/// Leaf downward drift speed (pixels/sec).
const LEAF_FALL_SPEED_PX: f32 = 15.0;
/// Leaf lifetime in seconds.
const LEAF_LIFETIME_SECS: f32 = 4.0;

/// Rain fall speed (pixels/sec).
const RAIN_FALL_SPEED_PX: f32 = 200.0;
/// Rain horizontal drift (pixels/sec, ~70 degree angle).
const RAIN_DRIFT_SPEED_PX: f32 = 70.0;
/// Rain lifetime in seconds.
const RAIN_LIFETIME_SECS: f32 = 1.5;

/// Salt added to per-frame seed when picking rain count.
const RAIN_SEED_SALT: u32 = 7777;
/// Salt added per raindrop instance.
const RAIN_INSTANCE_SALT: u32 = 5555;

/// Spawn weather particles relative to the camera position.
#[allow(clippy::too_many_arguments)]
pub fn spawn_weather_particles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weather: Res<WeatherState>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
    registry: Res<BiomeRegistry>,
) {
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dt = time.delta_secs();
    let dir = wind_dir.as_vec2();

    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_ALIGNMENT, |a| a.alignment);

    // Accumulate fractional particles using a simple seed.
    let frame_seed = f32_to_seed(time.elapsed_secs());

    // Leaf spawning.
    if weather.current.has_leaves() {
        let rate = match weather.current {
            WeatherKind::Breezy => BREEZY_LEAVES_PER_SEC,
            WeatherKind::Windy => WINDY_LEAVES_PER_SEC,
            WeatherKind::Storm => STORM_LEAVES_PER_SEC,
            _ => 0.0,
        };
        let fractional = rate * dt;
        let count = fractional_to_count(fractional, frame_seed);
        for i in 0..count {
            let s = frame_seed.wrapping_add(i);
            spawn_leaf(
                &mut commands,
                &asset_server,
                cam_pos,
                s,
                alignment,
                dir,
                &registry,
            );
        }
    }

    // Rain spawning.
    if weather.current.has_rain() {
        let rate = match weather.current {
            WeatherKind::Rain => RAIN_DROPS_PER_SEC,
            WeatherKind::Storm => STORM_DROPS_PER_SEC,
            _ => 0.0,
        };
        let fractional = rate * dt;
        let count = fractional_to_count(fractional, frame_seed.wrapping_add(RAIN_SEED_SALT));
        for i in 0..count {
            let s = frame_seed.wrapping_add(i).wrapping_add(RAIN_INSTANCE_SALT);
            spawn_raindrop(&mut commands, &asset_server, cam_pos, s, dir);
        }
    }
}

fn spawn_leaf(
    commands: &mut Commands,
    asset_server: &AssetServer,
    cam_pos: Vec2,
    seed: u32,
    alignment: u8,
    wind_dir: Vec2,
    registry: &BiomeRegistry,
) {
    let leaf = registry.leaf_particle(alignment);
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let speed = LEAF_SPEED_MIN_PX
        + hash_frac(seed.wrapping_add(2)) * (LEAF_SPEED_MAX_PX - LEAF_SPEED_MIN_PX);

    let pos = Vec3::new(
        cam_pos.x + x_offset,
        cam_pos.y + y_offset,
        Layer::Weather.z_f32(),
    );

    // Leaf drifts along wind direction + slight downward fall.
    let velocity = wind_dir * speed + Vec2::new(0.0, -LEAF_FALL_SPEED_PX);

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(LEAF_LIFETIME_SECS, TimerMode::Once),
            variant: leaf.variant,
        },
        Sprite {
            image: asset_server.load(leaf.path),
            ..default()
        },
        Transform::from_translation(pos),
    ));
}

fn spawn_raindrop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    cam_pos: Vec2,
    seed: u32,
    wind_dir: Vec2,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_top = cam_pos.y + VIEWPORT_HALF_H_PX;

    let pos = Vec3::new(cam_pos.x + x_offset, y_top, Layer::Weather.z_f32());

    // Rain falls down with horizontal drift from wind direction.
    let velocity = wind_dir * RAIN_DRIFT_SPEED_PX + Vec2::new(0.0, -RAIN_FALL_SPEED_PX);

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(RAIN_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::Raindrop,
        },
        Sprite {
            image: asset_server.load("sprites/particles/raindrop.webp"),
            ..default()
        },
        Transform::from_translation(pos),
    ));
}

/// Move particles and despawn expired ones.
pub fn update_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut WeatherParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut particle) in &mut query {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;
    }
}

/// Despawn all weather particles on game exit.
pub fn despawn_weather_particles(
    mut commands: Commands,
    query: Query<Entity, With<WeatherParticle>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
