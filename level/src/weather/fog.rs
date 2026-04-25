//! Fog patches -- low-alpha ellipses that drift through darkwood areas.

use bevy::prelude::*;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use models::layer::Layer;
use models::palette::FOG;
use models::weather::{ParticleVariant, WeatherParticle};
use models::wind::WindDirection;

use crate::area::AreaAlignment;
use crate::world::WorldMap;

use super::helpers::{
    f32_to_seed, fractional_to_count, hash_f32, DEFAULT_ALIGNMENT, VIEWPORT_HALF_H_PX,
    VIEWPORT_HALF_W_PX,
};

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

/// Salts mixed with the per-frame seed.
const FOG_FRAME_SALT: u32 = 77777;
const FOG_INSTANCE_SALT: u32 = 11111;

/// Pure predicate: should fog patches spawn for this alignment?
pub fn fog_active(alignment: AreaAlignment) -> bool {
    alignment > FOG_ALIGNMENT_THRESHOLD
}

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
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_ALIGNMENT, |a| a.alignment);
    if !fog_active(alignment) {
        return;
    }
    let Ok(cam_tf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let dir = wind_dir.as_vec2();

    let dt = time.delta_secs();
    let frame_seed = f32_to_seed(time.elapsed_secs()).wrapping_add(FOG_FRAME_SALT);
    let count = fractional_to_count(FOG_PATCHES_PER_SEC * dt, frame_seed);

    for i in 0..count {
        let s = frame_seed.wrapping_add(i).wrapping_add(FOG_INSTANCE_SALT);
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
mod tests {
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
