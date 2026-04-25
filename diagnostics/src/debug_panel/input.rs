//! Toggle and live-edit inputs for the F5 debug panel.

use bevy::prelude::*;
use keybinds::{Action, Keybinds};
use models::time::GameClock;
use models::weather::{WeatherKind, WeatherState};
use models::wind::WindStrength;

use super::components::{DebugPanel, DebugPanelState, WEATHER_CYCLE};

/// Keys that step the time-of-day backwards / forwards by one period.
const PREV_PERIOD_KEYS: &[KeyCode] = &[KeyCode::BracketLeft, KeyCode::Comma];
const NEXT_PERIOD_KEYS: &[KeyCode] = &[KeyCode::BracketRight, KeyCode::Period];

/// Keys that cycle weather backwards / forwards.
const PREV_WEATHER_KEYS: &[KeyCode] = &[KeyCode::Minus, KeyCode::Semicolon];
const NEXT_WEATHER_KEYS: &[KeyCode] = &[KeyCode::Equal, KeyCode::Slash, KeyCode::Quote];

/// How far ahead to push the weather state-machine's next check, in game-hours,
/// after the user forces a weather kind. Keeps manual override visible for a while.
const FORCED_WEATHER_HOLD_HOURS: f32 = 12.0;
const HOURS_PER_DAY: f32 = 24.0;

const FALLBACK_WEATHER_LEN: i32 = 5;

/// Hour anchors for each of the 8 period keys, in order.
pub(crate) const PERIOD_HOURS: [(f32, &str); 8] = [
    (0.0, "Night"),
    (5.5, "Dawn"),
    (9.0, "Morning"),
    (12.0, "Midday"),
    (15.0, "Afternoon"),
    (18.0, "Dusk"),
    (20.5, "Evening"),
    (23.0, "LateNight"),
];

pub(crate) fn toggle_debug_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    mut state: ResMut<DebugPanelState>,
    mut panel_q: Query<&mut Node, With<DebugPanel>>,
) {
    if !keyboard.just_pressed(bindings.key(Action::ToggleDebugPanel)) {
        return;
    }
    state.visible = !state.visible;
    if state.visible {
        state.cache_time = None;
        state.cache_weather = None;
    }
    let display = if state.visible {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panel_q {
        node.display = display;
    }
}

fn any_just_pressed(keyboard: &ButtonInput<KeyCode>, keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| keyboard.just_pressed(*k))
}

pub(crate) fn handle_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<GameClock>,
    mut weather: ResMut<WeatherState>,
    wind: Res<WindStrength>,
) {
    if any_just_pressed(&keyboard, PREV_PERIOD_KEYS) {
        clock.hour = prev_period_hour(clock.hour);
    }
    if any_just_pressed(&keyboard, NEXT_PERIOD_KEYS) {
        clock.hour = next_period_hour(clock.hour);
    }

    let direction: i32 = if any_just_pressed(&keyboard, PREV_WEATHER_KEYS) {
        -1
    } else if any_just_pressed(&keyboard, NEXT_WEATHER_KEYS) {
        1
    } else {
        0
    };
    if direction != 0 {
        let idx = weather_index(weather.current);
        let len = i32::try_from(WEATHER_CYCLE.len()).unwrap_or(FALLBACK_WEATHER_LEN);
        let next_idx = ((i32::try_from(idx).unwrap_or(0) + direction).rem_euclid(len))
            .try_into()
            .unwrap_or(0);
        let next_kind = WEATHER_CYCLE[next_idx];
        force_weather(&mut weather, next_kind, wind.0, clock.hour);
    }
}

fn weather_index(kind: WeatherKind) -> usize {
    WEATHER_CYCLE
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(0)
}

fn next_period_hour(current: f32) -> f32 {
    let idx = PERIOD_HOURS
        .iter()
        .position(|(h, _)| *h > current + 0.01)
        .unwrap_or(0);
    PERIOD_HOURS[idx].0
}

fn prev_period_hour(current: f32) -> f32 {
    let idx = PERIOD_HOURS
        .iter()
        .rposition(|(h, _)| *h < current - 0.01)
        .unwrap_or(PERIOD_HOURS.len() - 1);
    PERIOD_HOURS[idx].0
}

fn force_weather(weather: &mut WeatherState, kind: WeatherKind, current_wind: f32, hour: f32) {
    let (wind_min, wind_max) = kind.wind_range();
    weather.current = kind;
    weather.target_wind = (wind_min + wind_max) * 0.5;
    weather.wind_lerp_start = current_wind;
    weather.wind_lerp_remaining = WeatherState::WIND_LERP_DURATION_SECS;
    weather.next_check_hour = (hour + FORCED_WEATHER_HOLD_HOURS) % HOURS_PER_DAY;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_period_wraps_from_last() {
        let h = next_period_hour(23.5);
        assert!((h - PERIOD_HOURS[0].0).abs() < 0.001);
    }

    #[test]
    fn prev_period_wraps_from_first() {
        let h = prev_period_hour(0.0);
        assert!((h - PERIOD_HOURS[PERIOD_HOURS.len() - 1].0).abs() < 0.001);
    }

    #[test]
    fn next_period_steps_forward() {
        let h = next_period_hour(5.5);
        assert!((h - 9.0).abs() < 0.001);
    }

    #[test]
    fn prev_period_steps_back() {
        let h = prev_period_hour(9.0);
        assert!((h - 5.5).abs() < 0.001);
    }
}
