//! Dust mote particles -- shimmering specks during clear weather.

use bevy::prelude::*;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use models::layer::Layer;
use models::palette::DUST_MOTE;
use models::weather::{ParticleVariant, WeatherKind, WeatherParticle, WeatherState};
use models::wind::WindDirection;

use super::helpers::{
    f32_to_seed, fractional_to_count, hash_f32, VIEWPORT_HALF_H_PX, VIEWPORT_HALF_W_PX,
};

/// Dust motes per second during clear weather.
const DUST_MOTES_PER_SEC: f32 = 1.5;
/// Dust mote lifetime.
const DUST_LIFETIME_SECS: f32 = 8.0;
/// Dust mote drift speed (pixels/sec).
const DUST_DRIFT_SPEED_PX: f32 = 10.0;
/// Dust mote visual size (one side).
const DUST_SIZE_PX: f32 = 1.0;

/// Salts mixed with the per-frame seed.
const DUST_FRAME_SALT: u32 = 99991;
const DUST_INSTANCE_SALT: u32 = 31415;

/// Pure predicate: should dust motes spawn for this weather kind?
pub fn dust_mote_active(weather: WeatherKind) -> bool {
    matches!(weather, WeatherKind::Clear)
}

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
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(DUST_FRAME_SALT);
    let count = fractional_to_count(DUST_MOTES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(DUST_INSTANCE_SALT);
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

#[cfg(test)]
mod tests {
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
