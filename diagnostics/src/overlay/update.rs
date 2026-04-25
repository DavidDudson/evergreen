//! Per-frame update + toggle systems for the perf overlay.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use keybinds::{Action, Keybinds};
use level::world::WorldMap;
use models::palette;
use models::time::GameClock;
use models::weather::WeatherState;
use models::wind::{WindDirection, WindStrength};

use super::components::{
    AreaStatsText, BreakdownText, DisplayCache, EntityCountText, FpsText, FrameTimeText,
    HistogramBar, OverlayState, PerfOverlay, TimeOfDayText, WeatherText,
};
use super::histogram::update_bars;
use super::setup::{BAR_MAX_HEIGHT_PX, HISTOGRAM_FRAMES};
use crate::frame_stages::FrameStageTimings;

/// FPS thresholds for the FPS text colour. These remain absolute because
/// the player cares whether the game is fast, not just consistent.
const FPS_GOOD_THRESHOLD: f32 = 55.0;
const FPS_WARN_THRESHOLD: f32 = 30.0;

/// Minimum FPS the graph will assume (caps target_ms at ~100ms).
const FPS_FLOOR: f32 = 10.0;
/// Maximum FPS the graph will assume (caps target_ms at ~8ms).
const FPS_CEIL: f32 = 120.0;

const SECONDS_PER_MINUTE: f32 = 60.0;
const MS_PER_SECOND: f32 = 1000.0;

pub(crate) fn toggle_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    mut state: ResMut<OverlayState>,
    mut overlay_q: Query<&mut Node, With<PerfOverlay>>,
) {
    if !keyboard.just_pressed(bindings.key(Action::ToggleDiagnosticsOverlay)) {
        return;
    }
    state.visible = !state.visible;
    // Clear cache so text re-renders immediately on open.
    if state.visible {
        state.cache = DisplayCache::default();
    }
    let display = if state.visible {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut overlay_q {
        node.display = display;
    }
}

/// Only runs when `overlay_visible` is true (gated via `run_if` in the plugin).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn update_overlay(
    diagnostics: Res<DiagnosticsStore>,
    stage_timings: Res<FrameStageTimings>,
    mut state: ResMut<OverlayState>,
    world: Option<Res<WorldMap>>,
    clock: Option<Res<GameClock>>,
    weather: Option<Res<WeatherState>>,
    wind: Option<Res<WindStrength>>,
    wind_dir: Option<Res<WindDirection>>,
    mut fps_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
            Without<BreakdownText>,
        ),
    >,
    mut frame_q: Query<
        &mut Text,
        (
            With<FrameTimeText>,
            Without<FpsText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
            Without<BreakdownText>,
        ),
    >,
    mut entity_q: Query<
        &mut Text,
        (
            With<EntityCountText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<AreaStatsText>,
            Without<BreakdownText>,
        ),
    >,
    mut area_q: Query<
        &mut Text,
        (
            With<AreaStatsText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<BreakdownText>,
        ),
    >,
    mut tod_q: Query<
        &mut Text,
        (
            With<TimeOfDayText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
            Without<WeatherText>,
            Without<BreakdownText>,
        ),
    >,
    mut weather_q: Query<
        &mut Text,
        (
            With<WeatherText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
            Without<TimeOfDayText>,
            Without<BreakdownText>,
        ),
    >,
    mut breakdown_q: Query<
        &mut Text,
        (
            With<BreakdownText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
            Without<TimeOfDayText>,
            Without<WeatherText>,
        ),
    >,
    mut bar_q: Query<(&HistogramBar, &mut Node, &mut BackgroundColor)>,
) {
    // FPS -- also derive the dynamic target frame time from the smoothed reading.
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(bevy::diagnostic::Diagnostic::smoothed)
    {
        #[allow(clippy::as_conversions)] // f64 -> f32: display only, precision loss acceptable
        let fps_f = fps as f32;
        let clamped = fps_f.clamp(FPS_FLOOR, FPS_CEIL);
        state.target_frame_ms = MS_PER_SECOND / clamped;
        let new_str = format!("FPS  {fps_f:.1}");
        if state.cache.fps.as_deref() != Some(&new_str) {
            if let Ok((mut text, mut color)) = fps_q.single_mut() {
                *text = Text::new(new_str.clone());
                color.0 = fps_color(fps_f);
            }
            state.cache.fps = Some(new_str);
        }
    }

    // Frame time -- collect history, compute stats
    if let Some(ft) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(bevy::diagnostic::Diagnostic::value)
    {
        #[allow(clippy::as_conversions)] // f64 -> f32: value is already ms in Bevy 0.15+
        let ms = ft as f32;
        state.history.push_back(ms);
        while state.history.len() > HISTOGRAM_FRAMES {
            state.history.pop_front();
        }
    }
    if !state.history.is_empty() {
        let sum: f32 = state.history.iter().sum();
        #[allow(clippy::as_conversions)] // usize -> f32: len <= HISTOGRAM_FRAMES = 60
        let avg = sum / state.history.len() as f32;
        let min = state.history.iter().copied().fold(f32::MAX, f32::min);
        let max = state.history.iter().copied().fold(0.0_f32, f32::max);
        let new_str = format!("Frame  {avg:.1}ms  ({min:.1} / {max:.1})");
        if state.cache.frame_time.as_deref() != Some(&new_str) {
            if let Ok(mut text) = frame_q.single_mut() {
                *text = Text::new(new_str.clone());
            }
            state.cache.frame_time = Some(new_str);
        }
    }

    // Entity count
    if let Some(count) = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(bevy::diagnostic::Diagnostic::value)
    {
        let new_str = format!("Entities  {count:.0}");
        if state.cache.entity_count.as_deref() != Some(&new_str) {
            if let Ok(mut text) = entity_q.single_mut() {
                *text = Text::new(new_str.clone());
            }
            state.cache.entity_count = Some(new_str);
        }
    }

    // Area stats -- position, alignment, biome label
    if let Some(world) = &world {
        let pos = world.current;
        let area_alignment = world.get_area(pos).map_or(0, |a| a.alignment);
        let biome = biome_label(area_alignment);
        let new_str = format!("Area  ({}, {})  {biome} [{area_alignment}]", pos.x, pos.y);
        if state.cache.area_stats.as_deref() != Some(&new_str) {
            if let Ok(mut text) = area_q.single_mut() {
                *text = Text::new(new_str.clone());
            }
            state.cache.area_stats = Some(new_str);
        }
    }

    // Time of day
    if let Some(clock) = &clock {
        let hour = clock.hour;
        #[allow(clippy::as_conversions)] // f32 -> u32: hour in 0..24
        let h = hour as u32;
        #[allow(clippy::as_conversions)] // f32 -> u32: minute fraction in 0..60
        let m = ((hour - f32::from(u16::try_from(h).unwrap_or(0))) * SECONDS_PER_MINUTE) as u32;
        let period = time_period_label(hour);
        let new_str = format!("Time  {h:02}:{m:02}  {period}");
        if state.cache.time_of_day.as_deref() != Some(&new_str) {
            if let Ok(mut text) = tod_q.single_mut() {
                *text = Text::new(new_str.clone());
            }
            state.cache.time_of_day = Some(new_str);
        }
    }

    // Weather and wind
    if let Some(weather) = &weather {
        let wind_val = wind.as_ref().map_or(0.0, |w| w.0);
        let dir_label = wind_dir.as_ref().map_or("--", |d| d.label());
        let kind = weather_label(weather.current);
        let new_str = format!("Weather  {kind}  Wind {wind_val:.2} {dir_label}");
        if state.cache.weather.as_deref() != Some(&new_str) {
            if let Ok(mut text) = weather_q.single_mut() {
                *text = Text::new(new_str.clone());
            }
            state.cache.weather = Some(new_str);
        }
    }

    // Stage breakdown -- per-schedule CPU timings.
    let stages = stage_timings.smoothed();
    let new_breakdown = format!(
        "First       {:>5.2}\nPreUpdate   {:>5.2}\nUpdate      {:>5.2}\nPostUpdate  {:>5.2}\nLast        {:>5.2}\nCPU total   {:>5.2}",
        stages.first,
        stages.pre_update,
        stages.update,
        stages.post_update,
        stages.last,
        stages.cpu_total(),
    );
    if state.cache.breakdown.as_deref() != Some(&new_breakdown) {
        if let Ok(mut text) = breakdown_q.single_mut() {
            *text = Text::new(new_breakdown.clone());
        }
        state.cache.breakdown = Some(new_breakdown);
    }

    update_bars(&state, &mut bar_q, BAR_MAX_HEIGHT_PX);
}

pub(crate) fn fps_color(fps: f32) -> Color {
    if fps >= FPS_GOOD_THRESHOLD {
        palette::PERF_GOOD
    } else if fps >= FPS_WARN_THRESHOLD {
        palette::PERF_WARN
    } else {
        palette::PERF_BAD
    }
}

pub(crate) fn biome_label(alignment: u8) -> &'static str {
    match alignment {
        1..=25 => "City",
        26..=50 => "Greenwood",
        51..=75 => "Deep Green",
        76..=100 => "Darkwood",
        _ => "Unknown",
    }
}

pub(crate) fn time_period_label(hour: f32) -> &'static str {
    match hour {
        h if h < 5.0 => "Night",
        h if h < 7.0 => "Dawn",
        h if h < 11.0 => "Morning",
        h if h < 14.0 => "Midday",
        h if h < 17.0 => "Afternoon",
        h if h < 19.0 => "Dusk",
        h if h < 22.0 => "Evening",
        _ => "Night",
    }
}

pub(crate) fn weather_label(kind: models::weather::WeatherKind) -> &'static str {
    use models::weather::WeatherKind;
    match kind {
        WeatherKind::Clear => "Clear",
        WeatherKind::Breezy => "Breezy",
        WeatherKind::Windy => "Windy",
        WeatherKind::Rain => "Rain",
        WeatherKind::Storm => "Storm",
    }
}
