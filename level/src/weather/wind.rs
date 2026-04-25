//! Smoothly lerp the global `WindStrength` toward the active weather state's
//! target value, then snap once the lerp finishes.

use bevy::prelude::*;
use models::weather::WeatherState;
use models::wind::WindStrength;

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
