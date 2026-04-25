//! UI spawn for the perf overlay.

use bevy::prelude::*;
use models::palette;

use super::components::{
    AreaStatsText, BreakdownText, EntityCountText, FpsText, FrameTimeText, HistogramBar,
    PerfOverlay, TimeOfDayText, WeatherText,
};

const OVERLAY_FONT: &str = "fonts/NotoSans-Regular.ttf";
const FONT_SIZE_PX: f32 = 12.0;
const HEADER_FONT_SIZE_PX: f32 = 11.0;
const PANEL_PADDING_PX: f32 = 8.0;
const PANEL_MARGIN_PX: f32 = 8.0;
const PANEL_WIDTH_PX: f32 = 250.0;
const ROW_GAP_PX: f32 = 3.0;
const BAR_WIDTH_PX: f32 = 3.0;
const BAR_GAP_PX: f32 = 1.0;
pub(crate) const BAR_MAX_HEIGHT_PX: f32 = 36.0;
pub(crate) const HISTOGRAM_FRAMES: usize = 60;
const PANEL_Z_INDEX: i32 = 999;

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
            GlobalZIndex(PANEL_Z_INDEX),
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

    spawn_label(
        &mut commands,
        root,
        font.clone(),
        "STAGE BREAKDOWN  (avg ms)",
        palette::TITLE,
        HEADER_FONT_SIZE_PX,
    );
    commands.spawn((
        BreakdownText,
        Text::new("First       --\nPreUpdate   --\nUpdate      --\nPostUpdate  --\nLast        --\nCPU total   --"),
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
    commands.spawn((
        TimeOfDayText,
        Text::new("Time  --:--"),
        TextFont {
            font: font.clone(),
            font_size: FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        ChildOf(root),
    ));
    commands.spawn((
        WeatherText,
        Text::new("Weather  --"),
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
