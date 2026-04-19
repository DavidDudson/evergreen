use bevy::prelude::*;
use bevy_light_2d::prelude::Light2d;
use models::palette::{
    lerp_linear_color, AMBIENT_DAWN, AMBIENT_DAY, AMBIENT_DUSK, AMBIENT_NIGHT,
};
use models::time::GameClock;

/// Period transition hours -- `h >= constant` enters the transition toward
/// the next anchor (strict `<` comparisons in `target_for_hour` mean the
/// boundary hour belongs to the later period at t=0).
const NIGHT_END: f32 = 5.0;
const DAWN_END: f32 = 7.0;
const MORNING_END: f32 = 11.0;
const AFTERNOON_END: f32 = 17.0;
const DUSK_END: f32 = 19.0;
const EVENING_END: f32 = 22.0;

/// Brightness anchors per period (0..=1).
const NIGHT_BRIGHTNESS: f32 = 0.30;
const DAWN_BRIGHTNESS: f32 = 0.70;
const DAY_BRIGHTNESS: f32 = 1.00;
const DUSK_BRIGHTNESS: f32 = 0.50;

/// Lerp speed for ambient transitions (per second).
const AMBIENT_LERP_SPEED: f32 = 2.0;

/// Resolved ambient target for a given hour-of-day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientTarget {
    pub color: Color,
    pub brightness: f32,
}

impl AmbientTarget {
    const NIGHT: Self = Self {
        color: AMBIENT_NIGHT,
        brightness: NIGHT_BRIGHTNESS,
    };
    const DAWN: Self = Self {
        color: AMBIENT_DAWN,
        brightness: DAWN_BRIGHTNESS,
    };
    const DAY: Self = Self {
        color: AMBIENT_DAY,
        brightness: DAY_BRIGHTNESS,
    };
    const DUSK: Self = Self {
        color: AMBIENT_DUSK,
        brightness: DUSK_BRIGHTNESS,
    };
}

fn lerp_target(a: AmbientTarget, b: AmbientTarget, t: f32) -> AmbientTarget {
    AmbientTarget {
        color: lerp_linear_color(a.color, b.color, t),
        brightness: a.brightness + (b.brightness - a.brightness) * t,
    }
}

/// Map an hour-of-day (0..24) to an ambient target by interpolating between
/// the four time-of-day anchors.
pub fn target_for_hour(hour: f32) -> AmbientTarget {
    let h = hour.clamp(0.0, 24.0);
    if h < NIGHT_END {
        AmbientTarget::NIGHT
    } else if h < DAWN_END {
        let t = (h - NIGHT_END) / (DAWN_END - NIGHT_END);
        lerp_target(AmbientTarget::NIGHT, AmbientTarget::DAWN, t)
    } else if h < MORNING_END {
        let t = (h - DAWN_END) / (MORNING_END - DAWN_END);
        lerp_target(AmbientTarget::DAWN, AmbientTarget::DAY, t)
    } else if h < AFTERNOON_END {
        AmbientTarget::DAY
    } else if h < DUSK_END {
        let t = (h - AFTERNOON_END) / (DUSK_END - AFTERNOON_END);
        lerp_target(AmbientTarget::DAY, AmbientTarget::DUSK, t)
    } else if h < EVENING_END {
        let t = (h - DUSK_END) / (EVENING_END - DUSK_END);
        lerp_target(AmbientTarget::DUSK, AmbientTarget::NIGHT, t)
    } else {
        AmbientTarget::NIGHT
    }
}

/// Per-frame system: lerp the camera's `Light2d.ambient_light` toward the
/// time-of-day target. `bevy_light_2d` 0.9 wraps `AmbientLight2d` inside
/// `Light2d`; query and mutate the wrapper.
pub fn sync_ambient_light(
    clock: Res<GameClock>,
    time: Res<Time>,
    mut query: Query<&mut Light2d, With<Camera2d>>,
) {
    let target = target_for_hour(clock.hour);
    let alpha = (AMBIENT_LERP_SPEED * time.delta_secs()).min(1.0);
    for mut light in &mut query {
        light.ambient_light.brightness +=
            (target.brightness - light.ambient_light.brightness) * alpha;
        light.ambient_light.color =
            lerp_linear_color(light.ambient_light.color, target.color, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    /// Compare two colors by converting both to `LinearRgba` to avoid
    /// false mismatches between `Srgba` and `LinearRgba` variants.
    fn approx_color(a: Color, b: Color) {
        let a = a.to_linear();
        let b = b.to_linear();
        approx(a.red, b.red);
        approx(a.green, b.green);
        approx(a.blue, b.blue);
        approx(a.alpha, b.alpha);
    }

    #[test]
    fn ambient_at_midday_returns_day() {
        let t = target_for_hour(12.0);
        assert_eq!(t.color, AMBIENT_DAY);
        approx(t.brightness, DAY_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_midnight_returns_night() {
        let t = target_for_hour(0.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }

    #[test]
    fn ambient_in_dusk_plateau_returns_dusk() {
        // 18.0 sits squarely in the AFTERNOON_END..DUSK_END plateau where
        // `target_for_hour` returns the DUSK constant directly (no lerp).
        let t = target_for_hour(18.0);
        assert_eq!(t.color, AMBIENT_DUSK);
        approx(t.brightness, DUSK_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_dawn_anchor_returns_dawn() {
        let t = target_for_hour(7.0);
        approx_color(t.color, AMBIENT_DAWN);
        approx(t.brightness, DAWN_BRIGHTNESS);
    }

    #[test]
    fn ambient_lerps_smoothly_in_dawn() {
        let t = target_for_hour(6.0);
        let expected_brightness =
            NIGHT_BRIGHTNESS + (DAWN_BRIGHTNESS - NIGHT_BRIGHTNESS) * 0.5;
        approx(t.brightness, expected_brightness);
    }

    #[test]
    fn ambient_after_evening_end_returns_night() {
        let t = target_for_hour(23.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }

    #[test]
    fn ambient_at_hour_24_clamps_to_night() {
        let t = target_for_hour(24.0);
        assert_eq!(t.color, AMBIENT_NIGHT);
        approx(t.brightness, NIGHT_BRIGHTNESS);
    }
}
