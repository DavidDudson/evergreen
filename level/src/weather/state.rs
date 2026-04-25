//! Weather state machine + wind synchronisation.

use bevy::prelude::*;
use models::time::GameClock;
use models::weather::WeatherState;
use models::wind::{WindDirection, WindStrength};

use crate::biome_registry::{BiomeRegistry, WEATHER_KINDS};
use crate::world::WorldMap;

/// Minimum game-hours between weather transition checks.
const MIN_CHECK_INTERVAL_HOURS: f32 = 3.0;
/// Maximum game-hours between weather transition checks.
const MAX_CHECK_INTERVAL_HOURS: f32 = 5.0;

/// Hours in a full day (for wrapping).
const HOURS_PER_DAY: f32 = 24.0;

/// Default alignment used when the current area is unknown (mid-greenwood).
const DEFAULT_ALIGNMENT: u8 = 50;

/// Check whether it is time to transition weather, and if so pick a new state.
pub fn weather_state_machine(
    mut weather: ResMut<WeatherState>,
    wind: Res<WindStrength>,
    mut wind_dir: ResMut<WindDirection>,
    clock: Res<GameClock>,
    world: Res<WorldMap>,
    registry: Res<BiomeRegistry>,
) {
    if clock.hour < weather.next_check_hour
        && !(weather.next_check_hour > HOURS_PER_DAY - MAX_CHECK_INTERVAL_HOURS
            && clock.hour < MIN_CHECK_INTERVAL_HOURS)
    {
        return;
    }

    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_ALIGNMENT, |a| a.alignment);
    let weights = registry.weather_weights(alignment);

    // Deterministic-ish random from clock + current state.
    let seed = super::helpers::f32_to_seed(clock.hour)
        .wrapping_add(super::helpers::weather_kind_discriminant(weather.current))
        .wrapping_mul(2_654_435_761);
    let total: u32 = weights.iter().sum();
    let roll = seed % total;
    let mut cumulative: u32 = 0;
    let mut next_kind = WEATHER_KINDS[0];
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if roll < cumulative {
            next_kind = WEATHER_KINDS[i];
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
