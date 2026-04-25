//! Histogram bar rendering helpers for the perf overlay.

use bevy::prelude::*;
use models::palette;

use super::components::{HistogramBar, OverlayState};

/// 60 fps target -- bar colours are always relative to this so green = hitting 60.
pub(crate) const TARGET_FRAME_MS: f32 = 1000.0 / 60.0;
/// >1.5x 60fps target -> yellow.
pub(crate) const BAR_WARN_MULT: f32 = 1.5;
/// >2.5x 60fps target -> red.
pub(crate) const BAR_BAD_MULT: f32 = 2.5;
/// Full bar height = 4x current frame time.
pub(crate) const GRAPH_SCALE_MULT: f32 = 4.0;

/// Updates each histogram bar's height and colour based on the rolling history.
/// Only mutates components whose value changed to avoid Bevy marking all
/// 60 bar entities dirty every frame.
pub(crate) fn update_bars(
    state: &OverlayState,
    bar_q: &mut Query<(&HistogramBar, &mut Node, &mut BackgroundColor)>,
    bar_max_height_px: f32,
) {
    let target = state.target_frame_ms;
    let max_ms = target * GRAPH_SCALE_MULT;
    for (bar, mut node, mut bg) in bar_q.iter_mut() {
        let ms = state.history.get(bar.0).copied().unwrap_or(0.0);
        let frac = (ms / max_ms).clamp(0.0, 1.0);
        let new_height = Val::Px(frac * bar_max_height_px);
        let new_color = bar_color(ms);
        if node.height != new_height {
            node.height = new_height;
        }
        if bg.0 != new_color {
            bg.0 = new_color;
        }
    }
}

pub(crate) fn bar_color(ms: f32) -> Color {
    if ms <= TARGET_FRAME_MS * BAR_WARN_MULT {
        palette::PERF_GOOD
    } else if ms <= TARGET_FRAME_MS * BAR_BAD_MULT {
        palette::PERF_WARN
    } else {
        palette::PERF_BAD
    }
}
