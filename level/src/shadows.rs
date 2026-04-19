//! Baked soft drop shadows. Each spawn site gets one `Sprite` child using a
//! shared pre-blurred ellipse texture. The sun-swing system translates each
//! shadow's local X along an arc (left at sunrise, centered at noon, right at
//! sunset) and modulates alpha with time of day.

use bevy::math::Vec2;
use bevy::prelude::*;
use models::palette::DROP_SHADOW_TINT;
use models::time::GameClock;
use std::f32::consts::PI;

/// Z-offset placing shadow just under its parent sprite (same layer).
const SHADOW_Z_OFFSET: f32 = -0.1;

/// Hour when the sun rises (shadow fades in).
const SUN_RISE_HOUR: f32 = 5.0;
/// Hour when the sun sets (shadow fades out).
const SUN_SET_HOUR: f32 = 19.0;
/// Hour of solar noon (shadow directly under parent).
const SOLAR_NOON_HOUR: f32 = 12.0;
/// Half the sun's above-horizon arc, in hours.
const HALF_DAY_HOURS: f32 = (SUN_SET_HOUR - SUN_RISE_HOUR) * 0.5;

/// Peak horizontal offset (pixels) at sunrise/sunset. 0 at noon.
const SHADOW_PEAK_SHIFT_PX: f32 = 10.0;

/// Path to the baked soft-ellipse shadow sprite.
const SHADOW_SPRITE_PATH: &str = "sprites/particles/drop_shadow.webp";

/// Shared shadow assets. Spawned at `Startup` once.
#[derive(Resource)]
pub struct DropShadowAssets {
    pub texture: Handle<Image>,
}

/// Marker on each shadow sprite. Stores its base size / offset so the sun
/// system can animate X while preserving the per-site geometry.
#[derive(Component)]
pub struct DropShadow {
    pub ground_offset_y: f32,
}

/// Startup system: load the baked shadow texture.
pub fn init_shadow_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load(SHADOW_SPRITE_PATH);
    commands.insert_resource(DropShadowAssets { texture });
}

/// Spawn one drop shadow child of `parent`. The sprite is stretched to
/// `half_size * 2` and offset down by `ground_offset_y`.
pub fn spawn_drop_shadow(
    commands: &mut Commands,
    assets: &DropShadowAssets,
    parent: Entity,
    half_size: Vec2,
    ground_offset_y: f32,
) {
    commands.spawn((
        DropShadow { ground_offset_y },
        Sprite {
            image: assets.texture.clone(),
            custom_size: Some(half_size * 2.0),
            color: DROP_SHADOW_TINT,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, ground_offset_y, SHADOW_Z_OFFSET)),
        ChildOf(parent),
    ));
}

/// Sun-arc angle for the given hour. `None` when below horizon.
fn sun_angle(hour: f32) -> Option<f32> {
    if !(SUN_RISE_HOUR..=SUN_SET_HOUR).contains(&hour) {
        return None;
    }
    let t = (hour - SOLAR_NOON_HOUR) / HALF_DAY_HOURS;
    Some(t * (PI * 0.5))
}

/// Daylight intensity (0..=1) with a one-hour fade at sunrise and sunset.
fn daylight_intensity(hour: f32) -> f32 {
    const FADE_HOURS: f32 = 1.0;
    if sun_angle(hour).is_none() {
        return 0.0;
    }
    let rise_fade = ((hour - SUN_RISE_HOUR) / FADE_HOURS).clamp(0.0, 1.0);
    let set_fade = ((SUN_SET_HOUR - hour) / FADE_HOURS).clamp(0.0, 1.0);
    rise_fade.min(set_fade)
}

/// Per-frame system: swing every shadow's X along the sun arc and modulate
/// its alpha with daylight intensity.
pub fn animate_shadow_sun(
    clock: Res<GameClock>,
    mut query: Query<(&DropShadow, &mut Transform, &mut Sprite)>,
) {
    let intensity = daylight_intensity(clock.hour);
    let x_factor = sun_angle(clock.hour).map_or(0.0, f32::sin);
    let shift = SHADOW_PEAK_SHIFT_PX * x_factor;
    let tinted = DROP_SHADOW_TINT.with_alpha(DROP_SHADOW_TINT.alpha() * intensity);

    for (shadow, mut tf, mut sprite) in &mut query {
        tf.translation.x = shift;
        tf.translation.y = shadow.ground_offset_y;
        if sprite.color != tinted {
            sprite.color = tinted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    #[test]
    fn sun_angle_at_noon_is_zero() {
        approx(sun_angle(SOLAR_NOON_HOUR).unwrap(), 0.0);
    }

    #[test]
    fn sun_angle_negative_before_noon() {
        assert!(sun_angle(SUN_RISE_HOUR).unwrap() < 0.0);
    }

    #[test]
    fn sun_angle_positive_after_noon() {
        assert!(sun_angle(SUN_SET_HOUR).unwrap() > 0.0);
    }

    #[test]
    fn sun_angle_is_none_at_midnight() {
        assert!(sun_angle(0.0).is_none());
    }

    #[test]
    fn daylight_zero_at_night() {
        approx(daylight_intensity(2.0), 0.0);
        approx(daylight_intensity(22.0), 0.0);
    }

    #[test]
    fn daylight_peaks_at_noon() {
        approx(daylight_intensity(SOLAR_NOON_HOUR), 1.0);
    }
}
