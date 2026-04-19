//! F5 debug panel. Cycle time-of-day and weather state live.
//!
//! `[` / `]` -- step current hour between 8 fixed periods.
//! `-` / `=` -- cycle `WeatherKind` (Clear/Breezy/Windy/Rain/Storm).

use bevy::prelude::*;
use models::palette;
use models::time::GameClock;
use models::weather::{WeatherKind, WeatherState};
use models::wind::WindStrength;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const OVERLAY_FONT: &str = "fonts/NotoSans-Regular.ttf";
const FONT_SIZE_PX: f32 = 12.0;
const HEADER_FONT_SIZE_PX: f32 = 11.0;
const PANEL_PADDING_PX: f32 = 8.0;
const PANEL_MARGIN_PX: f32 = 8.0;
const PANEL_WIDTH_PX: f32 = 230.0;
const ROW_GAP_PX: f32 = 3.0;

/// Hour anchors for each of the 8 period keys, in order.
const PERIOD_HOURS: [(f32, &str); 8] = [
    (0.0, "Night"),
    (5.5, "Dawn"),
    (9.0, "Morning"),
    (12.0, "Midday"),
    (15.0, "Afternoon"),
    (18.0, "Dusk"),
    (20.5, "Evening"),
    (23.0, "LateNight"),
];

const WEATHER_CYCLE: [WeatherKind; 5] = [
    WeatherKind::Clear,
    WeatherKind::Breezy,
    WeatherKind::Windy,
    WeatherKind::Rain,
    WeatherKind::Storm,
];

/// How far ahead to push the weather state-machine's next check, in game-hours,
/// after the user forces a weather kind. Keeps manual override visible for a while.
const FORCED_WEATHER_HOLD_HOURS: f32 = 12.0;
const HOURS_PER_DAY: f32 = 24.0;

// ---------------------------------------------------------------------------
// Components & resource
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct DebugPanel;

#[derive(Component)]
pub(crate) struct DebugTimeText;

#[derive(Component)]
pub(crate) struct DebugWeatherText;

#[derive(Resource, Default)]
pub struct DebugPanelState {
    pub visible: bool,
    cache_time: Option<String>,
    cache_weather: Option<String>,
}

pub fn panel_visible(state: Res<DebugPanelState>) -> bool {
    state.visible
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub(crate) fn setup_debug_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(OVERLAY_FONT);

    let root = commands
        .spawn((
            DebugPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(PANEL_MARGIN_PX),
                right: Val::Px(PANEL_MARGIN_PX),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(PANEL_PADDING_PX)),
                width: Val::Px(PANEL_WIDTH_PX),
                row_gap: Val::Px(ROW_GAP_PX),
                ..Node::default()
            },
            BackgroundColor(palette::PERF_BG),
            GlobalZIndex(999),
        ))
        .id();

    spawn_label(
        &mut commands,
        root,
        font.clone(),
        "F5  DEBUG",
        palette::TITLE,
        HEADER_FONT_SIZE_PX,
    );

    commands.spawn((
        DebugTimeText,
        Text::new("Time  --:--  --"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));

    commands.spawn((
        DebugWeatherText,
        Text::new("Weather  --"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));

    spawn_label(
        &mut commands,
        root,
        font.clone(),
        "[ ] or , .   prev/next period",
        palette::DIALOG_TEXT,
        HEADER_FONT_SIZE_PX,
    );
    spawn_label(
        &mut commands,
        root,
        font,
        "- = ; / '   prev/next weather",
        palette::DIALOG_TEXT,
        HEADER_FONT_SIZE_PX,
    );
}

fn spawn_label(
    commands: &mut Commands,
    parent: Entity,
    font: Handle<Font>,
    text: &str,
    color: Color,
    size: f32,
) {
    commands.spawn((
        Text::new(text),
        TextFont {
            font,
            font_size: size,
            ..default()
        },
        TextColor(color),
        ChildOf(parent),
    ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub(crate) fn toggle_debug_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugPanelState>,
    mut panel_q: Query<&mut Node, With<DebugPanel>>,
) {
    if !keyboard.just_pressed(KeyCode::F5) {
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

/// Keys that step the time-of-day backwards / forwards by one period.
const PREV_PERIOD_KEYS: &[KeyCode] = &[KeyCode::BracketLeft, KeyCode::Comma];
const NEXT_PERIOD_KEYS: &[KeyCode] = &[KeyCode::BracketRight, KeyCode::Period];

/// Keys that cycle weather backwards / forwards.
const PREV_WEATHER_KEYS: &[KeyCode] = &[KeyCode::Minus, KeyCode::Semicolon];
const NEXT_WEATHER_KEYS: &[KeyCode] = &[KeyCode::Equal, KeyCode::Slash, KeyCode::Quote];

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
        let len = i32::try_from(WEATHER_CYCLE.len()).unwrap_or(5);
        let next_idx = ((i32::try_from(idx).unwrap_or(0) + direction).rem_euclid(len))
            .try_into()
            .unwrap_or(0);
        let next_kind = WEATHER_CYCLE[next_idx];
        force_weather(&mut weather, next_kind, wind.0, clock.hour);
    }
}

pub(crate) fn update_debug_panel(
    mut state: ResMut<DebugPanelState>,
    clock: Res<GameClock>,
    weather: Res<WeatherState>,
    mut time_q: Query<&mut Text, (With<DebugTimeText>, Without<DebugWeatherText>)>,
    mut weather_q: Query<&mut Text, (With<DebugWeatherText>, Without<DebugTimeText>)>,
) {
    let hour = clock.hour;
    #[allow(clippy::as_conversions)]
    let h = hour as u32;
    #[allow(clippy::as_conversions)]
    let m = ((hour - f32::from(u16::try_from(h).unwrap_or(0))) * 60.0) as u32;
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn weather_index(kind: WeatherKind) -> usize {
    WEATHER_CYCLE
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(0)
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

fn nearest_period_label(hour: f32) -> &'static str {
    PERIOD_HOURS
        .iter()
        .rev()
        .find(|(h, _)| hour + 0.0001 >= *h)
        .map_or(PERIOD_HOURS[0].1, |(_, label)| *label)
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

    #[test]
    fn nearest_label_picks_current_period() {
        assert_eq!(nearest_period_label(12.5), "Midday");
        assert_eq!(nearest_period_label(6.0), "Dawn");
    }
}
