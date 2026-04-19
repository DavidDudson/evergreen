// level/src/weather.rs

use bevy::prelude::*;
use models::decoration::Biome;
use models::layer::Layer;
use models::time::GameClock;
use models::weather::{ParticleVariant, WeatherKind, WeatherParticle, WeatherState};
use models::wind::{WindDirection, WindStrength};

use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum game-hours between weather transition checks.
const MIN_CHECK_INTERVAL_HOURS: f32 = 3.0;
/// Maximum game-hours between weather transition checks.
const MAX_CHECK_INTERVAL_HOURS: f32 = 5.0;

/// Hours in a full day (for wrapping).
const HOURS_PER_DAY: f32 = 24.0;

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

/// Splash lifetime in seconds.
const SPLASH_LIFETIME_SECS: f32 = 0.2;

/// Camera viewport half-width for particle spawning (pixels).
const VIEWPORT_HALF_W_PX: f32 = 280.0;
/// Camera viewport half-height for particle spawning (pixels).
const VIEWPORT_HALF_H_PX: f32 = 160.0;

/// Number of biome transition weights per row in the weight tables.
const WEIGHT_COUNT: usize = 5;

// Biome-weighted transition tables: [Clear, Breezy, Windy, Rain, Storm]
const CITY_WEIGHTS: [u32; WEIGHT_COUNT] = [50, 25, 10, 10, 5];
const GREENWOOD_WEIGHTS: [u32; WEIGHT_COUNT] = [30, 30, 15, 15, 10];
const DARKWOOD_WEIGHTS: [u32; WEIGHT_COUNT] = [15, 20, 25, 25, 15];

const ALL_WEATHER_KINDS: [WeatherKind; WEIGHT_COUNT] = [
    WeatherKind::Clear,
    WeatherKind::Breezy,
    WeatherKind::Windy,
    WeatherKind::Rain,
    WeatherKind::Storm,
];

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Check whether it is time to transition weather, and if so pick a new state.
pub fn weather_state_machine(
    mut weather: ResMut<WeatherState>,
    wind: Res<WindStrength>,
    mut wind_dir: ResMut<WindDirection>,
    clock: Res<GameClock>,
    world: Res<WorldMap>,
) {
    if clock.hour < weather.next_check_hour
        && !(weather.next_check_hour > HOURS_PER_DAY - MAX_CHECK_INTERVAL_HOURS
            && clock.hour < MIN_CHECK_INTERVAL_HOURS)
    {
        return;
    }

    // Determine biome of current area.
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    let biome = Biome::from_alignment(alignment);
    let weights = match biome {
        Biome::City => &CITY_WEIGHTS,
        Biome::Greenwood => &GREENWOOD_WEIGHTS,
        Biome::Darkwood => &DARKWOOD_WEIGHTS,
    };

    // Deterministic-ish random from clock + current state.
    let seed = f32_to_seed(clock.hour)
        .wrapping_add(weather_kind_discriminant(weather.current))
        .wrapping_mul(2_654_435_761);
    let total: u32 = weights.iter().sum();
    let roll = seed % total;
    let mut cumulative: u32 = 0;
    let mut next_kind = WeatherKind::Clear;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if roll < cumulative {
            next_kind = ALL_WEATHER_KINDS[i];
            break;
        }
    }

    // Set the next check time.
    let interval_seed = seed.wrapping_mul(1_013_904_223);
    let interval_range = MAX_CHECK_INTERVAL_HOURS - MIN_CHECK_INTERVAL_HOURS;
    #[allow(clippy::as_conversions)]
    let interval_frac = (interval_seed % 1000) as f32 / 1000.0;
    let interval = MIN_CHECK_INTERVAL_HOURS + interval_range * interval_frac;
    weather.next_check_hour = (clock.hour + interval) % HOURS_PER_DAY;

    // Transition to new state.
    weather.current = next_kind;
    let (wind_min, wind_max) = next_kind.wind_range();
    #[allow(clippy::as_conversions)]
    let wind_frac = (seed.wrapping_add(42) % 1000) as f32 / 1000.0;
    weather.target_wind = wind_min + (wind_max - wind_min) * wind_frac;
    weather.wind_lerp_start = wind.0;
    weather.wind_lerp_remaining = WeatherState::WIND_LERP_DURATION_SECS;

    // Pick a new random wind direction (radians, 0..2PI).
    #[allow(clippy::as_conversions)]
    let dir_frac = (seed.wrapping_add(137) % 1000) as f32 / 1000.0;
    wind_dir.0 = dir_frac * std::f32::consts::TAU;
}

/// Smoothly lerp `WindStrength` toward the weather state's target.
pub fn sync_wind_strength(
    mut weather: ResMut<WeatherState>,
    mut wind: ResMut<WindStrength>,
    time: Res<Time>,
) {
    if weather.wind_lerp_remaining <= 0.0 {
        wind.0 = weather.target_wind;
        return;
    }

    weather.wind_lerp_remaining -= time.delta_secs();
    if weather.wind_lerp_remaining <= 0.0 {
        weather.wind_lerp_remaining = 0.0;
        wind.0 = weather.target_wind;
    } else {
        let t = 1.0 - weather.wind_lerp_remaining / WeatherState::WIND_LERP_DURATION_SECS;
        wind.0 = weather.wind_lerp_start + (weather.target_wind - weather.wind_lerp_start) * t;
    }
}

/// Spawn weather particles relative to the camera position.
pub fn spawn_weather_particles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weather: Res<WeatherState>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
) {
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dt = time.delta_secs();
    let dir = wind_dir.as_vec2();

    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);

    // Accumulate fractional particles using a simple seed.
    let frame_seed = f32_to_seed(time.elapsed_secs());

    // Leaf spawning
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
            spawn_leaf(&mut commands, &asset_server, cam_pos, s, alignment, dir);
        }
    }

    // Rain spawning
    if weather.current.has_rain() {
        let rate = match weather.current {
            WeatherKind::Rain => RAIN_DROPS_PER_SEC,
            WeatherKind::Storm => STORM_DROPS_PER_SEC,
            _ => 0.0,
        };
        let fractional = rate * dt;
        let count = fractional_to_count(fractional, frame_seed.wrapping_add(7777));
        for i in 0..count {
            let s = frame_seed.wrapping_add(i).wrapping_add(5555);
            spawn_raindrop(&mut commands, &asset_server, cam_pos, s, dir);
        }
    }
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_leaf(
    commands: &mut Commands,
    asset_server: &AssetServer,
    cam_pos: Vec2,
    seed: u32,
    alignment: u8,
    wind_dir: Vec2,
) {
    let biome = Biome::from_alignment(alignment);
    let (path, variant) = match biome {
        Biome::City => (
            "sprites/particles/paper_scrap.webp",
            ParticleVariant::PaperScrap,
        ),
        Biome::Greenwood => (
            "sprites/particles/green_leaf.webp",
            ParticleVariant::GreenLeaf,
        ),
        Biome::Darkwood => (
            "sprites/particles/brown_leaf.webp",
            ParticleVariant::BrownLeaf,
        ),
    };

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
            variant,
        },
        Sprite {
            image: asset_server.load(path),
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

/// Convert a fractional particle count to an integer, probabilistically rounding up.
fn fractional_to_count(fractional: f32, seed: u32) -> u32 {
    #[allow(clippy::as_conversions)]
    let whole = fractional as u32;
    let remainder = fractional - f32::from(u16::try_from(whole).unwrap_or(u16::MAX));
    let extra = if hash_frac(seed) < remainder { 1 } else { 0 };
    whole + extra
}

/// Hash a u32 seed into a float in [-range, +range].
fn hash_f32(seed: u32, range: f32) -> f32 {
    (hash_frac(seed) * 2.0 - 1.0) * range
}

/// Hash a u32 seed into a float in [0.0, 1.0).
fn hash_frac(seed: u32) -> f32 {
    let h = seed.wrapping_mul(374_761_393).wrapping_add(668_265_263);
    let h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    let h = h ^ (h >> 16);
    #[allow(clippy::as_conversions)]
    let frac = (h % 10000) as f32 / 10000.0;
    frac
}

/// Convert an f32 to a deterministic u32 seed.
fn f32_to_seed(value: f32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

/// Map a `WeatherKind` to a unique `u32` for use as a seed component.
fn weather_kind_discriminant(kind: WeatherKind) -> u32 {
    match kind {
        WeatherKind::Clear => 0,
        WeatherKind::Breezy => 1,
        WeatherKind::Windy => 2,
        WeatherKind::Rain => 3,
        WeatherKind::Storm => 4,
    }
}

// Suppress dead code warnings for constants used only for documentation/future use.
const _: f32 = SPLASH_LIFETIME_SECS;

// ---------------------------------------------------------------------------
// Firefly
// ---------------------------------------------------------------------------

use crate::area::AreaAlignment;

/// Alignment threshold above which fireflies are eligible to spawn.
const FIREFLY_ALIGNMENT_THRESHOLD: AreaAlignment = 60;
/// Hour-of-day before which fireflies are active (early morning).
const FIREFLY_HOUR_START: f32 = 5.0;
/// Hour-of-day after which fireflies are active (post-dusk).
const FIREFLY_HOUR_END: f32 = 19.0;

/// Pure predicate: should fireflies spawn for this hour + alignment?
pub fn firefly_active(hour: f32, alignment: AreaAlignment) -> bool {
    alignment > FIREFLY_ALIGNMENT_THRESHOLD
        && !(FIREFLY_HOUR_START..=FIREFLY_HOUR_END).contains(&hour)
}

use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use models::palette::FIREFLY;

/// Fireflies per second when active.
const FIREFLIES_PER_SEC: f32 = 0.8;
/// Firefly lifetime.
const FIREFLY_LIFETIME_SECS: f32 = 6.0;
/// Firefly horizontal drift speed (pixels/sec, randomized in +/-this range).
const FIREFLY_DRIFT_PX: f32 = 20.0;
/// Firefly visual size in world pixels (one side).
const FIREFLY_SIZE_PX: f32 = 2.0;

/// Per-frame system: spawn fireflies at night in darkwood-leaning biomes.
pub fn spawn_fireflies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    clock: Res<GameClock>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    if !firefly_active(clock.hour, alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(31415);
    let count = fractional_to_count(FIREFLIES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(2718);
        spawn_firefly(&mut commands, &mut meshes, &mut materials, cam_pos, s);
    }
}

fn spawn_firefly(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let drift = (hash_frac(seed.wrapping_add(2)) - 0.5) * 2.0 * FIREFLY_DRIFT_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(FIREFLY));

    commands.spawn((
        WeatherParticle {
            velocity: Vec2::new(drift, 0.0),
            lifetime: Timer::from_seconds(FIREFLY_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::Firefly,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(FIREFLY_SIZE_PX / 2.0, FIREFLY_SIZE_PX / 2.0, 1.0)),
    ));
}

/// Firefly pulse frequency (Hz).
const FIREFLY_PULSE_FREQ_HZ: f32 = 2.5;
/// Firefly pulse alpha range: [BASE - AMP, BASE + AMP].
const FIREFLY_PULSE_BASE: f32 = 0.6;
const FIREFLY_PULSE_AMP: f32 = 0.4;

/// Per-frame system: pulse each firefly's material alpha.
pub fn animate_fireflies(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &WeatherParticle)>,
) {
    let elapsed = time.elapsed_secs();
    for (entity, mat_handle, particle) in &query {
        if particle.variant != ParticleVariant::Firefly {
            continue;
        }
        let phase = entity_phase(entity);
        let pulse = FIREFLY_PULSE_BASE
            + FIREFLY_PULSE_AMP * (elapsed * FIREFLY_PULSE_FREQ_HZ + phase).sin();
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = mat.color.with_alpha(pulse);
        }
    }
}

fn entity_phase(entity: Entity) -> f32 {
    let bits = entity.to_bits();
    #[allow(clippy::as_conversions)]
    let frac = ((bits.wrapping_mul(2_654_435_761) % 10_000) as f32) / 10_000.0;
    frac * std::f32::consts::TAU
}

// ---------------------------------------------------------------------------
// Dust motes
// ---------------------------------------------------------------------------

/// Dust motes per second during clear weather.
const DUST_MOTES_PER_SEC: f32 = 1.5;
/// Dust mote lifetime.
const DUST_LIFETIME_SECS: f32 = 8.0;
/// Dust mote drift speed (pixels/sec).
const DUST_DRIFT_SPEED_PX: f32 = 10.0;
/// Dust mote visual size (one side).
const DUST_SIZE_PX: f32 = 1.0;

/// Pure predicate: should dust motes spawn for this weather kind?
pub fn dust_mote_active(weather: WeatherKind) -> bool {
    matches!(weather, WeatherKind::Clear)
}

use models::palette::DUST_MOTE;

/// Per-frame system: spawn dust motes during clear weather.
pub fn spawn_dust_motes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    weather: Res<WeatherState>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    if !dust_mote_active(weather.current) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dir = wind_dir.as_vec2();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(99991);
    let count = fractional_to_count(DUST_MOTES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(31415);
        spawn_dust_mote(&mut commands, &mut meshes, &mut materials, cam_pos, s, dir);
    }
}

fn spawn_dust_mote(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
    wind_dir: Vec2,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let velocity = wind_dir * DUST_DRIFT_SPEED_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(DUST_MOTE));

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(DUST_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::DustMote,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(DUST_SIZE_PX / 2.0, DUST_SIZE_PX / 2.0, 1.0)),
    ));
}

// ---------------------------------------------------------------------------
// Fog
// ---------------------------------------------------------------------------

/// Fog patches per second in darkwood.
const FOG_PATCHES_PER_SEC: f32 = 0.3;
/// Fog patch lifetime.
const FOG_LIFETIME_SECS: f32 = 12.0;
/// Fog patch drift speed (pixels/sec).
const FOG_DRIFT_SPEED_PX: f32 = 15.0;
/// Fog patch ellipse half-size (x, y) in world pixels.
const FOG_HALF_PX_X: f32 = 16.0;
const FOG_HALF_PX_Y: f32 = 8.0;
/// Alignment threshold above which fog spawns.
const FOG_ALIGNMENT_THRESHOLD: AreaAlignment = 75;

/// Pure predicate: should fog patches spawn for this alignment?
pub fn fog_active(alignment: AreaAlignment) -> bool {
    alignment > FOG_ALIGNMENT_THRESHOLD
}

use models::palette::FOG;

/// Per-frame system: spawn fog patches in darkwood areas.
pub fn spawn_fog_patches(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    wind_dir: Res<WindDirection>,
    camera_q: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
    world: Res<WorldMap>,
) {
    let alignment = world.get_area(world.current).map_or(50, |a| a.alignment);
    if !fog_active(alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dir = wind_dir.as_vec2();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(77777);
    let count = fractional_to_count(FOG_PATCHES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(11111);
        spawn_fog_patch(&mut commands, &mut meshes, &mut materials, cam_pos, s, dir);
    }
}

fn spawn_fog_patch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cam_pos: Vec2,
    seed: u32,
    wind_dir: Vec2,
) {
    let x_offset = hash_f32(seed, VIEWPORT_HALF_W_PX);
    let y_offset = hash_f32(seed.wrapping_add(1), VIEWPORT_HALF_H_PX);
    let velocity = wind_dir * FOG_DRIFT_SPEED_PX;

    let mesh = meshes.add(Circle::new(1.0));
    let material = materials.add(ColorMaterial::from(FOG));

    commands.spawn((
        WeatherParticle {
            velocity,
            lifetime: Timer::from_seconds(FOG_LIFETIME_SECS, TimerMode::Once),
            variant: ParticleVariant::FogPatch,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(
            cam_pos.x + x_offset,
            cam_pos.y + y_offset,
            Layer::Weather.z_f32(),
        ))
        .with_scale(Vec3::new(FOG_HALF_PX_X, FOG_HALF_PX_Y, 1.0)),
    ));
}

#[cfg(test)]
mod fog_tests {
    use super::*;

    #[test]
    fn fog_active_in_darkwood() {
        assert!(fog_active(80));
    }

    #[test]
    fn fog_inactive_in_greenwood() {
        assert!(!fog_active(50));
    }
}

#[cfg(test)]
mod dust_mote_tests {
    use super::*;

    #[test]
    fn dust_mote_active_during_clear() {
        assert!(dust_mote_active(WeatherKind::Clear));
    }

    #[test]
    fn dust_mote_inactive_during_rain() {
        assert!(!dust_mote_active(WeatherKind::Rain));
    }
}

#[cfg(test)]
mod firefly_tests {
    use super::*;

    #[test]
    fn firefly_active_at_night_in_darkwood() {
        assert!(firefly_active(22.0, 80));
    }

    #[test]
    fn firefly_inactive_at_midday() {
        assert!(!firefly_active(12.0, 80));
    }

    #[test]
    fn firefly_inactive_in_city_at_night() {
        assert!(!firefly_active(22.0, 10));
    }

    #[test]
    fn firefly_threshold_boundary_strict() {
        assert!(!firefly_active(22.0, 60));
    }
}
