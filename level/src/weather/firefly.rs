//! Firefly particles -- nighttime, darkwood-leaning biomes.

use bevy::prelude::*;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use models::layer::Layer;
use models::palette::FIREFLY;
use models::time::GameClock;
use models::weather::{ParticleVariant, WeatherParticle};

use crate::area::AreaAlignment;
use crate::world::WorldMap;

use super::helpers::{
    f32_to_seed, fractional_to_count, hash_f32, hash_frac, DEFAULT_ALIGNMENT, VIEWPORT_HALF_H_PX,
    VIEWPORT_HALF_W_PX,
};

/// Alignment threshold above which fireflies are eligible to spawn.
const FIREFLY_ALIGNMENT_THRESHOLD: AreaAlignment = 60;
/// Hour-of-day before which fireflies are active (early morning).
const FIREFLY_HOUR_START: f32 = 5.0;
/// Hour-of-day after which fireflies are active (post-dusk).
const FIREFLY_HOUR_END: f32 = 19.0;

/// Fireflies per second when active.
const FIREFLIES_PER_SEC: f32 = 0.8;
/// Firefly lifetime.
const FIREFLY_LIFETIME_SECS: f32 = 6.0;
/// Firefly horizontal drift speed (pixels/sec, randomized in +/-this range).
const FIREFLY_DRIFT_PX: f32 = 20.0;
/// Firefly visual size in world pixels (one side).
const FIREFLY_SIZE_PX: f32 = 2.0;

/// Salts mixed with the per-frame seed.
const FIREFLY_FRAME_SALT: u32 = 31415;
const FIREFLY_INSTANCE_SALT: u32 = 2718;

/// Firefly pulse frequency (Hz).
const FIREFLY_PULSE_FREQ_HZ: f32 = 2.5;
/// Firefly pulse alpha range: [BASE - AMP, BASE + AMP].
const FIREFLY_PULSE_BASE: f32 = 0.6;
const FIREFLY_PULSE_AMP: f32 = 0.4;
/// Firefly vertical bob frequency (Hz).
const FIREFLY_BOB_FREQ_HZ: f32 = 1.5;
/// Firefly vertical bob amplitude (pixels).
const FIREFLY_BOB_AMP_PX: f32 = 8.0;

/// Marker for firefly particles so animation/queries filter cheaply.
#[derive(Component)]
pub struct Firefly;

/// Pure predicate: should fireflies spawn for this hour + alignment?
pub fn firefly_active(hour: f32, alignment: AreaAlignment) -> bool {
    alignment > FIREFLY_ALIGNMENT_THRESHOLD
        && !(FIREFLY_HOUR_START..=FIREFLY_HOUR_END).contains(&hour)
}

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
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_ALIGNMENT, |a| a.alignment);
    if !firefly_active(clock.hour, alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(FIREFLY_FRAME_SALT);
    let count = fractional_to_count(FIREFLIES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed
            .wrapping_add(i)
            .wrapping_add(FIREFLY_INSTANCE_SALT);
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
        Firefly,
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

/// Per-frame system: pulse each firefly's material alpha and apply a sine
/// vertical bob so they feel alive.
#[allow(clippy::type_complexity)]
pub fn animate_fireflies(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<
        (Entity, &MeshMaterial2d<ColorMaterial>, &mut Transform),
        (With<WeatherParticle>, With<Firefly>),
    >,
) {
    let elapsed = time.elapsed_secs();
    let dt = time.delta_secs();
    for (entity, mat_handle, mut tf) in &mut query {
        let phase = entity_phase(entity);
        let pulse = FIREFLY_PULSE_BASE
            + FIREFLY_PULSE_AMP * (elapsed * FIREFLY_PULSE_FREQ_HZ + phase).sin();
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = mat.color.with_alpha(pulse);
        }
        // Sine bob: add per-frame y-delta. Derivative of sin gives cos for velocity.
        let bob_vel = (elapsed * FIREFLY_BOB_FREQ_HZ + phase).cos()
            * FIREFLY_BOB_AMP_PX
            * FIREFLY_BOB_FREQ_HZ
            * std::f32::consts::TAU;
        tf.translation.y += bob_vel * dt;
    }
}

fn entity_phase(entity: Entity) -> f32 {
    let bits = entity.to_bits();
    #[allow(clippy::as_conversions)]
    let frac = ((bits.wrapping_mul(2_654_435_761) % 10_000) as f32) / 10_000.0;
    frac * std::f32::consts::TAU
}

#[cfg(test)]
mod tests {
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
