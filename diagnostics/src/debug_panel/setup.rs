//! Spawns the F5 debug panel UI tree.

use bevy::prelude::*;
use models::palette;

use super::components::{DebugPanel, DebugTimeText, DebugWeatherText};

const OVERLAY_FONT: &str = "fonts/NotoSans-Regular.ttf";
const FONT_SIZE_PX: f32 = 12.0;
const HEADER_FONT_SIZE_PX: f32 = 11.0;
const PANEL_PADDING_PX: f32 = 8.0;
const PANEL_MARGIN_PX: f32 = 8.0;
const PANEL_WIDTH_PX: f32 = 230.0;
const ROW_GAP_PX: f32 = 3.0;
const PANEL_Z_INDEX: i32 = 999;

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
            GlobalZIndex(PANEL_Z_INDEX),
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
