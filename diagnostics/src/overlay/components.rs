//! Components and run-condition resource for the perf overlay.

use bevy::prelude::*;
use std::collections::VecDeque;

/// Default smoothed target frame time in ms (60 fps).
const DEFAULT_TARGET_FRAME_MS: f32 = 1000.0 / 60.0;

#[derive(Component)]
pub struct PerfOverlay;

#[derive(Component)]
pub(crate) struct FpsText;

#[derive(Component)]
pub(crate) struct FrameTimeText;

#[derive(Component)]
pub(crate) struct EntityCountText;

#[derive(Component)]
pub(crate) struct AreaStatsText;

#[derive(Component)]
pub(crate) struct TimeOfDayText;

#[derive(Component)]
pub(crate) struct WeatherText;

#[derive(Component)]
pub(crate) struct BreakdownText;

#[derive(Component)]
pub(crate) struct HistogramBar(pub usize);

/// Cached display values -- only write Text/Node when the rounded value changes
/// so Bevy's text measurement pipeline isn't triggered every frame needlessly.
#[derive(Default)]
pub(crate) struct DisplayCache {
    pub fps: Option<String>,
    pub frame_time: Option<String>,
    pub entity_count: Option<String>,
    pub area_stats: Option<String>,
    pub time_of_day: Option<String>,
    pub weather: Option<String>,
    pub breakdown: Option<String>,
}

#[derive(Resource)]
pub struct OverlayState {
    pub visible: bool,
    pub(crate) history: VecDeque<f32>,
    pub(crate) cache: DisplayCache,
    /// Smoothed target frame time in ms, derived from the FPS diagnostic.
    /// Starts at 16.7ms (60 fps) and updates each frame the overlay is open.
    pub(crate) target_frame_ms: f32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            history: VecDeque::default(),
            cache: DisplayCache::default(),
            target_frame_ms: DEFAULT_TARGET_FRAME_MS,
        }
    }
}

/// Run-condition: only execute the update system while the overlay is open.
pub fn overlay_visible(state: Res<OverlayState>) -> bool {
    state.visible
}
