//! Marker components and shared state for the F5 debug panel.

use bevy::prelude::*;
use models::weather::WeatherKind;

pub(crate) const WEATHER_CYCLE: [WeatherKind; 5] = [
    WeatherKind::Clear,
    WeatherKind::Breezy,
    WeatherKind::Windy,
    WeatherKind::Rain,
    WeatherKind::Storm,
];

/// Root node of the debug panel UI.
#[derive(Component)]
pub struct DebugPanel;

/// Marker for the time-of-day line.
#[derive(Component)]
pub(crate) struct DebugTimeText;

/// Marker for the weather line.
#[derive(Component)]
pub(crate) struct DebugWeatherText;

/// Visibility + render-cache state for the panel.
#[derive(Resource, Default)]
pub struct DebugPanelState {
    pub visible: bool,
    pub(crate) cache_time: Option<String>,
    pub(crate) cache_weather: Option<String>,
}

/// `run_if` predicate -- only run update systems while the panel is open.
pub fn panel_visible(state: Res<DebugPanelState>) -> bool {
    state.visible
}
