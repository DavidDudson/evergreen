use bevy::prelude::*;
use models::time::GameClock;

use crate::atmosphere::BiomeAtmosphere;

/// Lerp speed for time-of-day transitions (per second).
const TIME_LERP_SPEED: f32 = 2.0;

const NIGHT_END: f32 = 5.0;
const DAWN_END: f32 = 7.0;
const MORNING_END: f32 = 11.0;
const MIDDAY_END: f32 = 14.0;
const AFTERNOON_END: f32 = 17.0;
const DUSK_END: f32 = 19.0;
const EVENING_END: f32 = 22.0;

const NIGHT_BRIGHTNESS: f32 = 0.3;
const NIGHT_TINT_R: f32 = 0.6;
const NIGHT_TINT_G: f32 = 0.7;
const NIGHT_TINT_B: f32 = 1.0;

const MORNING_BRIGHTNESS: f32 = 1.0;
const MORNING_TINT_R: f32 = 1.0;
const MORNING_TINT_G: f32 = 0.98;
const MORNING_TINT_B: f32 = 0.95;

const DAWN_START_BRIGHTNESS: f32 = NIGHT_BRIGHTNESS;
const DAWN_START_TINT_R: f32 = NIGHT_TINT_R;
const DAWN_START_TINT_G: f32 = NIGHT_TINT_G;
const DAWN_START_TINT_B: f32 = NIGHT_TINT_B;

const DAWN_END_BRIGHTNESS: f32 = 1.0;
const DAWN_END_TINT_R: f32 = 1.0;
const DAWN_END_TINT_G: f32 = 0.9;
const DAWN_END_TINT_B: f32 = 0.8;

const MIDDAY_BRIGHTNESS: f32 = 1.0;
const MIDDAY_TINT_R: f32 = 1.0;
const MIDDAY_TINT_G: f32 = 1.0;
const MIDDAY_TINT_B: f32 = 0.95;

const AFTERNOON_BRIGHTNESS: f32 = 0.95;
const AFTERNOON_TINT_R: f32 = 1.0;
const AFTERNOON_TINT_G: f32 = 0.95;
const AFTERNOON_TINT_B: f32 = 0.85;

const DUSK_START_BRIGHTNESS: f32 = AFTERNOON_BRIGHTNESS;
const DUSK_START_TINT_R: f32 = AFTERNOON_TINT_R;
const DUSK_START_TINT_G: f32 = AFTERNOON_TINT_G;
const DUSK_START_TINT_B: f32 = AFTERNOON_TINT_B;

const DUSK_END_BRIGHTNESS: f32 = 0.3;
const DUSK_END_TINT_R: f32 = 0.8;
const DUSK_END_TINT_G: f32 = 0.5;
const DUSK_END_TINT_B: f32 = 0.6;

const EVENING_BRIGHTNESS: f32 = 0.3;
const EVENING_TINT_R: f32 = 0.7;
const EVENING_TINT_G: f32 = 0.6;
const EVENING_TINT_B: f32 = 0.9;

/// Advance the game clock each frame.
pub fn tick_game_clock(mut clock: ResMut<GameClock>, time: Res<Time>) {
    clock.tick(time.delta_secs());
}

/// Interpolate brightness and tint from the current game hour and write to
/// the camera's `BiomeAtmosphere` time-of-day fields.
pub fn sync_time_of_day(
    clock: Res<GameClock>,
    time: Res<Time>,
    mut query: Query<&mut BiomeAtmosphere>,
) {
    let (target_brightness, target_r, target_g, target_b) = period_values(clock.hour);
    let alpha = (TIME_LERP_SPEED * time.delta_secs()).min(1.0);

    for mut atmo in &mut query {
        atmo.tod_brightness += (target_brightness - atmo.tod_brightness) * alpha;
        atmo.tod_tint_r += (target_r - atmo.tod_tint_r) * alpha;
        atmo.tod_tint_g += (target_g - atmo.tod_tint_g) * alpha;
        atmo.tod_tint_b += (target_b - atmo.tod_tint_b) * alpha;
    }
}

fn period_values(hour: f32) -> (f32, f32, f32, f32) {
    if hour < NIGHT_END {
        (NIGHT_BRIGHTNESS, NIGHT_TINT_R, NIGHT_TINT_G, NIGHT_TINT_B)
    } else if hour < DAWN_END {
        let t = (hour - NIGHT_END) / (DAWN_END - NIGHT_END);
        (
            lerp(DAWN_START_BRIGHTNESS, DAWN_END_BRIGHTNESS, t),
            lerp(DAWN_START_TINT_R, DAWN_END_TINT_R, t),
            lerp(DAWN_START_TINT_G, DAWN_END_TINT_G, t),
            lerp(DAWN_START_TINT_B, DAWN_END_TINT_B, t),
        )
    } else if hour < MORNING_END {
        (
            MORNING_BRIGHTNESS,
            MORNING_TINT_R,
            MORNING_TINT_G,
            MORNING_TINT_B,
        )
    } else if hour < MIDDAY_END {
        (
            MIDDAY_BRIGHTNESS,
            MIDDAY_TINT_R,
            MIDDAY_TINT_G,
            MIDDAY_TINT_B,
        )
    } else if hour < AFTERNOON_END {
        (
            AFTERNOON_BRIGHTNESS,
            AFTERNOON_TINT_R,
            AFTERNOON_TINT_G,
            AFTERNOON_TINT_B,
        )
    } else if hour < DUSK_END {
        let t = (hour - AFTERNOON_END) / (DUSK_END - AFTERNOON_END);
        (
            lerp(DUSK_START_BRIGHTNESS, DUSK_END_BRIGHTNESS, t),
            lerp(DUSK_START_TINT_R, DUSK_END_TINT_R, t),
            lerp(DUSK_START_TINT_G, DUSK_END_TINT_G, t),
            lerp(DUSK_START_TINT_B, DUSK_END_TINT_B, t),
        )
    } else if hour < EVENING_END {
        (
            EVENING_BRIGHTNESS,
            EVENING_TINT_R,
            EVENING_TINT_G,
            EVENING_TINT_B,
        )
    } else {
        (NIGHT_BRIGHTNESS, NIGHT_TINT_R, NIGHT_TINT_G, NIGHT_TINT_B)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
