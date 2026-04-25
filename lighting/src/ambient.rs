use bevy::prelude::*;
use bevy_light_2d::prelude::{AmbientLight2d, Light2d};
use models::palette::{lerp_linear_color, AMBIENT_DAWN, AMBIENT_DAY, AMBIENT_DUSK, AMBIENT_NIGHT};
use models::time::GameClock;

/// Lerp speed for ambient transitions (per second).
const AMBIENT_LERP_SPEED_PER_SEC: f32 = 2.0;

const HOURS_PER_DAY: f32 = 24.0;
const NIGHT_BRIGHTNESS: f32 = 0.30;
const DAWN_BRIGHTNESS: f32 = 0.70;
const DAY_BRIGHTNESS: f32 = 1.00;
const DUSK_BRIGHTNESS: f32 = 0.50;

/// Resolved ambient target for a given hour-of-day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientTarget {
    pub color: Color,
    pub brightness: f32,
}

impl AmbientTarget {
    pub const NIGHT: Self = Self {
        color: AMBIENT_NIGHT,
        brightness: NIGHT_BRIGHTNESS,
    };
    pub const DAWN: Self = Self {
        color: AMBIENT_DAWN,
        brightness: DAWN_BRIGHTNESS,
    };
    pub const DAY: Self = Self {
        color: AMBIENT_DAY,
        brightness: DAY_BRIGHTNESS,
    };
    pub const DUSK: Self = Self {
        color: AMBIENT_DUSK,
        brightness: DUSK_BRIGHTNESS,
    };
}

/// Day-cycle anchor table. `target_for_hour` interpolates between consecutive
/// anchors. Keep entries sorted ascending by hour, with `0.0` and `24.0`
/// endpoints so any clamp-input maps cleanly.
#[derive(Resource, Debug, Clone)]
pub struct DayCycleProfile {
    anchors: Vec<(f32, AmbientTarget)>,
}

impl Default for DayCycleProfile {
    fn default() -> Self {
        Self {
            anchors: vec![
                (0.0, AmbientTarget::NIGHT),
                (5.0, AmbientTarget::NIGHT),
                (7.0, AmbientTarget::DAWN),
                (11.0, AmbientTarget::DAY),
                (17.0, AmbientTarget::DAY),
                (19.0, AmbientTarget::DUSK),
                (22.0, AmbientTarget::NIGHT),
                (HOURS_PER_DAY, AmbientTarget::NIGHT),
            ],
        }
    }
}

impl DayCycleProfile {
    /// Map an hour-of-day (0..24) to an ambient target by interpolating
    /// between the surrounding anchor entries.
    pub fn target_for_hour(&self, hour: f32) -> AmbientTarget {
        let h = hour.clamp(0.0, HOURS_PER_DAY);
        let next_idx = self
            .anchors
            .iter()
            .position(|(anchor_h, _)| *anchor_h > h)
            .unwrap_or(self.anchors.len() - 1);
        if next_idx == 0 {
            return self.anchors[0].1;
        }
        let prev_idx = next_idx - 1;
        let (prev_h, prev_target) = self.anchors[prev_idx];
        let (next_h, next_target) = self.anchors[next_idx];
        let span = next_h - prev_h;
        if span <= f32::EPSILON {
            return prev_target;
        }
        let t = (h - prev_h) / span;
        lerp_target(prev_target, next_target, t)
    }
}

fn lerp_target(a: AmbientTarget, b: AmbientTarget, t: f32) -> AmbientTarget {
    AmbientTarget {
        color: lerp_linear_color(a.color, b.color, t),
        brightness: a.brightness + (b.brightness - a.brightness) * t,
    }
}

/// Per-frame system: lerp the camera's `Light2d.ambient_light` toward the
/// time-of-day target. `bevy_light_2d` 0.9 wraps `AmbientLight2d` inside
/// `Light2d`; query and mutate the wrapper.
pub fn sync_ambient_light(
    profile: Res<DayCycleProfile>,
    clock: Res<GameClock>,
    time: Res<Time>,
    mut query: Query<&mut Light2d, With<Camera2d>>,
) {
    let target = profile.target_for_hour(clock.hour);
    let alpha = (AMBIENT_LERP_SPEED_PER_SEC * time.delta_secs()).min(1.0);
    for mut light in &mut query {
        light.ambient_light.brightness +=
            (target.brightness - light.ambient_light.brightness) * alpha;
        light.ambient_light.color =
            lerp_linear_color(light.ambient_light.color, target.color, alpha);
    }
}

/// Reset the camera's ambient to neutral white so menus/GameOver/etc. don't
/// inherit night-tinted blue from the last gameplay frame. Mirror of
/// `post_processing::grading::reset_color_grading`.
pub fn reset_ambient_light(mut query: Query<&mut Light2d, With<Camera2d>>) {
    for mut light in &mut query {
        light.ambient_light = AmbientLight2d {
            color: Color::WHITE,
            brightness: 1.0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    fn approx_color(a: Color, b: Color) {
        let a = a.to_linear();
        let b = b.to_linear();
        approx(a.red, b.red);
        approx(a.green, b.green);
        approx(a.blue, b.blue);
        approx(a.alpha, b.alpha);
    }

    fn profile() -> DayCycleProfile {
        DayCycleProfile::default()
    }

    #[test]
    fn ambient_at_midday_returns_day() {
        let t = profile().target_for_hour(12.0);
        assert_eq!(t.color, AMBIENT_DAY);
        approx(t.brightness, DAY_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_midnight_returns_night() {
        let t = profile().target_for_hour(0.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_dusk_anchor_returns_dusk() {
        let t = profile().target_for_hour(19.0);
        approx_color(t.color, AMBIENT_DUSK);
        approx(t.brightness, DUSK_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_dawn_anchor_returns_dawn() {
        let t = profile().target_for_hour(7.0);
        approx_color(t.color, AMBIENT_DAWN);
        approx(t.brightness, DAWN_BRIGHTNESS);
    }

    #[test]
    fn ambient_lerps_smoothly_in_dawn() {
        let t = profile().target_for_hour(6.0);
        let expected_brightness = NIGHT_BRIGHTNESS + (DAWN_BRIGHTNESS - NIGHT_BRIGHTNESS) * 0.5;
        approx(t.brightness, expected_brightness);
    }

    #[test]
    fn ambient_after_evening_end_returns_night() {
        let t = profile().target_for_hour(23.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_hour_24_clamps_to_night() {
        let t = profile().target_for_hour(24.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }
}
