//! Per-frame text updates for the F5 debug panel.

use bevy::prelude::*;
use models::time::GameClock;
use models::weather::{WeatherKind, WeatherState};

use super::components::{DebugPanelState, DebugTimeText, DebugWeatherText};
use super::input::PERIOD_HOURS;

const MINUTES_PER_HOUR: f32 = 60.0;
const PERIOD_NEAR_EPS: f32 = 0.0001;

pub(crate) fn update_debug_panel(
    mut state: ResMut<DebugPanelState>,
    clock: Res<GameClock>,
    weather: Res<WeatherState>,
    mut time_q: Query<&mut Text, (With<DebugTimeText>, Without<DebugWeatherText>)>,
    mut weather_q: Query<&mut Text, (With<DebugWeatherText>, Without<DebugTimeText>)>,
) {
    let hour = clock.hour;
    #[allow(clippy::as_conversions)]
    // hour is bounded 0..=24; rounded floor to u32 has no fallible From impl.
    let h = hour as u32;
    #[allow(clippy::as_conversions)]
    // minutes within an hour is bounded 0..=59; rounded floor to u32 fits.
    let m = ((hour - f32::from(u16::try_from(h).unwrap_or(0))) * MINUTES_PER_HOUR) as u32;
    let label = nearest_period_label(hour);
    let time_str = format!("Time  {h:02}:{m:02}  {label}");
    if state.cache_time.as_deref() != Some(&time_str) {
        if let Ok(mut text) = time_q.single_mut() {
            *text = Text::new(time_str.clone());
        }
        state.cache_time = Some(time_str);
    }

    let weather_str = format!("Weather  {}", weather_label(weather.current));
    if state.cache_weather.as_deref() != Some(&weather_str) {
        if let Ok(mut text) = weather_q.single_mut() {
            *text = Text::new(weather_str.clone());
        }
        state.cache_weather = Some(weather_str);
    }
}

fn weather_label(kind: WeatherKind) -> &'static str {
    match kind {
        WeatherKind::Clear => "Clear",
        WeatherKind::Breezy => "Breezy",
        WeatherKind::Windy => "Windy",
        WeatherKind::Rain => "Rain",
        WeatherKind::Storm => "Storm",
    }
}

fn nearest_period_label(hour: f32) -> &'static str {
    PERIOD_HOURS
        .iter()
        .rev()
        .find(|(h, _)| hour + PERIOD_NEAR_EPS >= *h)
        .map_or(PERIOD_HOURS[0].1, |(_, label)| *label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_label_picks_current_period() {
        assert_eq!(nearest_period_label(12.5), "Midday");
        assert_eq!(nearest_period_label(6.0), "Dawn");
    }
}
