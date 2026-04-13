//! F3 performance overlay.
//!
//! Shows FPS, frame time (avg / min / max), entity count, and a 60-frame
//! timing histogram.  Toggle with F3.  For per-system profiling open
//! Chrome DevTools → Performance → Record.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use level::world::WorldMap;
use models::palette;
use std::collections::VecDeque;

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
const BAR_WIDTH_PX: f32 = 3.0;
const BAR_GAP_PX: f32 = 1.0;
const BAR_MAX_HEIGHT_PX: f32 = 36.0;
const HISTOGRAM_FRAMES: usize = 60;

/// 60 fps target — bar colours are always relative to this so green = hitting 60.
const TARGET_FRAME_MS: f32 = 1000.0 / 60.0;
const BAR_WARN_MULT: f32 = 1.5; // >1.5× 60fps target → yellow
const BAR_BAD_MULT: f32 = 2.5; // >2.5× 60fps target → red
const GRAPH_SCALE_MULT: f32 = 4.0; // full bar height = 4× current frame time

/// FPS thresholds for the FPS text colour. These remain absolute because
/// the player cares whether the game is fast, not just consistent.
const FPS_GOOD_THRESHOLD: f32 = 55.0;
const FPS_WARN_THRESHOLD: f32 = 30.0;

/// Minimum FPS the graph will assume (caps target_ms at ~100ms).
const FPS_FLOOR: f32 = 10.0;
/// Maximum FPS the graph will assume (caps target_ms at ~8ms).
const FPS_CEIL: f32 = 120.0;

// ---------------------------------------------------------------------------
// Components & resource
// ---------------------------------------------------------------------------

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
pub(crate) struct HistogramBar(usize);

/// Cached display values — only write Text/Node when the rounded value changes
/// so Bevy's text measurement pipeline isn't triggered every frame needlessly.
#[derive(Default)]
struct DisplayCache {
    fps: Option<String>,
    frame_time: Option<String>,
    entity_count: Option<String>,
    area_stats: Option<String>,
}

#[derive(Resource)]
pub struct OverlayState {
    pub visible: bool,
    history: VecDeque<f32>,
    cache: DisplayCache,
    /// Smoothed target frame time in ms, derived from the FPS diagnostic.
    /// Starts at 16.7ms (60 fps) and updates each frame the overlay is open.
    target_frame_ms: f32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            history: VecDeque::default(),
            cache: DisplayCache::default(),
            target_frame_ms: 1000.0 / 60.0,
        }
    }
}

/// Run-condition: only execute the update system while the overlay is open.
pub fn overlay_visible(state: Res<OverlayState>) -> bool {
    state.visible
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub(crate) fn setup_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(OVERLAY_FONT);

    let root = commands
        .spawn((
            PerfOverlay,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(PANEL_MARGIN_PX),
                left: Val::Px(PANEL_MARGIN_PX),
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
        "F3  PERF",
        palette::TITLE,
        HEADER_FONT_SIZE_PX,
    );

    commands.spawn((
        FpsText,
        Text::new("FPS  --"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));
    commands.spawn((
        FrameTimeText,
        Text::new("Frame  --ms  (-- / --)"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));
    commands.spawn((
        EntityCountText,
        Text::new("Entities  --"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));
    commands.spawn((
        AreaStatsText,
        Text::new("Area  --"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));

    // Histogram
    let histogram = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                height: Val::Px(BAR_MAX_HEIGHT_PX),
                width: Val::Percent(100.0),
                column_gap: Val::Px(BAR_GAP_PX),
                margin: UiRect::top(Val::Px(ROW_GAP_PX)),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    for i in 0..HISTOGRAM_FRAMES {
        commands.spawn((
            HistogramBar(i),
            Node {
                width: Val::Px(BAR_WIDTH_PX),
                height: Val::Px(0.0),
                ..Node::default()
            },
            BackgroundColor(palette::PERF_GOOD),
            ChildOf(histogram),
        ));
    }

    spawn_label(
        &mut commands,
        root,
        font,
        "Chrome DevTools > Performance > Record",
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

pub(crate) fn toggle_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<OverlayState>,
    mut overlay_q: Query<&mut Node, With<PerfOverlay>>,
) {
    if !keyboard.just_pressed(KeyCode::F3) {
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
pub(crate) fn update_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut state: ResMut<OverlayState>,
    world: Option<Res<WorldMap>>,
    mut fps_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
        ),
    >,
    mut frame_q: Query<
        &mut Text,
        (
            With<FrameTimeText>,
            Without<FpsText>,
            Without<EntityCountText>,
            Without<AreaStatsText>,
        ),
    >,
    mut entity_q: Query<
        &mut Text,
        (
            With<EntityCountText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<AreaStatsText>,
        ),
    >,
    mut area_q: Query<
        &mut Text,
        (
            With<AreaStatsText>,
            Without<FpsText>,
            Without<FrameTimeText>,
            Without<EntityCountText>,
        ),
    >,
    mut bar_q: Query<(&HistogramBar, &mut Node, &mut BackgroundColor)>,
) {
    // FPS — also derive the dynamic target frame time from the smoothed reading.
    if let Some(d) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = d.smoothed() {
            #[allow(clippy::as_conversions)] // f64 → f32: display only, precision loss acceptable
            let fps_f = fps as f32;
            // Clamp to avoid degenerate scaling on startup or stalls.
            let clamped = fps_f.clamp(FPS_FLOOR, FPS_CEIL);
            state.target_frame_ms = 1000.0 / clamped;
            let new_str = format!("FPS  {fps_f:.1}");
            if state.cache.fps.as_deref() != Some(&new_str) {
                if let Ok((mut text, mut color)) = fps_q.single_mut() {
                    *text = Text::new(new_str.clone());
                    color.0 = fps_color(fps_f);
                }
                state.cache.fps = Some(new_str);
            }
        }
    }

    // Frame time — collect history, compute stats
    if let Some(d) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(ft) = d.value() {
            #[allow(clippy::as_conversions)] // f64 → f32: value is already ms in Bevy 0.15+
            let ms = ft as f32;
            state.history.push_back(ms);
            while state.history.len() > HISTOGRAM_FRAMES {
                state.history.pop_front();
            }
        }
    }

    if !state.history.is_empty() {
        let sum: f32 = state.history.iter().sum();
        #[allow(clippy::as_conversions)] // usize → f32: len ≤ HISTOGRAM_FRAMES = 60
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
    if let Some(d) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
        if let Some(count) = d.value() {
            let new_str = format!("Entities  {count:.0}");
            if state.cache.entity_count.as_deref() != Some(&new_str) {
                if let Ok(mut text) = entity_q.single_mut() {
                    *text = Text::new(new_str.clone());
                }
                state.cache.entity_count = Some(new_str);
            }
        }
    }

    // Area stats — position, alignment, biome label
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

    // Histogram bars — scale and colour relative to the live target frame time.
    // Only mutate components whose value changed to avoid Bevy marking all
    // 60 bar entities dirty every frame.
    let target = state.target_frame_ms;
    let max_ms = target * GRAPH_SCALE_MULT;
    for (bar, mut node, mut bg) in &mut bar_q {
        let ms = state.history.get(bar.0).copied().unwrap_or(0.0);
        let frac = (ms / max_ms).clamp(0.0, 1.0);
        let new_height = Val::Px(frac * BAR_MAX_HEIGHT_PX);
        let new_color = bar_color(ms);
        if node.height != new_height {
            node.height = new_height;
        }
        if bg.0 != new_color {
            bg.0 = new_color;
        }
    }
}

fn fps_color(fps: f32) -> Color {
    if fps >= FPS_GOOD_THRESHOLD {
        palette::PERF_GOOD
    } else if fps >= FPS_WARN_THRESHOLD {
        palette::PERF_WARN
    } else {
        palette::PERF_BAD
    }
}

fn bar_color(ms: f32) -> Color {
    if ms <= TARGET_FRAME_MS * BAR_WARN_MULT {
        palette::PERF_GOOD
    } else if ms <= TARGET_FRAME_MS * BAR_BAD_MULT {
        palette::PERF_WARN
    } else {
        palette::PERF_BAD
    }
}

fn biome_label(alignment: u8) -> &'static str {
    match alignment {
        1..=25 => "City",
        26..=50 => "Greenwood",
        51..=75 => "Deep Green",
        76..=100 => "Darkwood",
        _ => "Unknown",
    }
}
