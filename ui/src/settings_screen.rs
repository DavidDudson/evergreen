use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use models::game_states::GameState;
use models::settings::GameSettings;

use crate::theme;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const PAGE_PADDING_PX: f32 = 48.0;
const TITLE_FONT_SIZE_PX: f32 = 32.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 28.0;

const SECTION_FONT_SIZE_PX: f32 = 14.0;
const SECTION_MARGIN_TOP_PX: f32 = 20.0;
const SECTION_MARGIN_BOTTOM_PX: f32 = 8.0;

const ROW_HEIGHT_PX: f32 = 40.0;
const ROW_MARGIN_BOTTOM_PX: f32 = 6.0;
const ROW_PADDING_H_PX: f32 = 12.0;
const ROW_RADIUS_PX: f32 = 4.0;

const LABEL_FONT_SIZE_PX: f32 = 16.0;
const VALUE_FONT_SIZE_PX: f32 = 16.0;
const VALUE_MIN_WIDTH_PX: f32 = 40.0;
const STEP_BTN_SIZE_PX: f32 = 32.0;
const STEP_BTN_FONT_SIZE_PX: f32 = 18.0;
const STEP_BTN_RADIUS_PX: f32 = 4.0;

const NAV_FONT_SIZE_PX: f32 = 18.0;
const NAV_PADDING_H_PX: f32 = 24.0;
const NAV_PADDING_V_PX: f32 = 10.0;
const NAV_MARGIN_TOP_PX: f32 = 24.0;
const NAV_MARGIN_RIGHT_PX: f32 = 12.0;
const NAV_BORDER_PX: f32 = 2.0;
const NAV_RADIUS_PX: f32 = 6.0;

const MAX_VOLUME: u8 = 10;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct SettingsScreen;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VolumeButton {
    MasterDown,
    MasterUp,
    BgmDown,
    BgmUp,
    SfxDown,
    SfxUp,
}

#[derive(Component, Clone, Copy)]
pub(crate) enum VolumeDisplay {
    Master,
    Bgm,
    Sfx,
}

#[derive(Component)]
pub(crate) struct FullscreenButton;

#[derive(Component)]
pub(crate) struct FullscreenDisplay;

#[derive(Component)]
pub(crate) struct KeybindsNavButton;

#[derive(Component)]
pub(crate) struct SettingsBackButton;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands, settings: Res<GameSettings>) {
    let root = commands
        .spawn((
            SettingsScreen,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(PAGE_PADDING_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DARK_BG),
        ))
        .id();

    commands.spawn((
        Text::new("Settings"),
        TextColor(theme::TITLE),
        TextFont { font_size: TITLE_FONT_SIZE_PX, ..default() },
        Node { margin: UiRect::bottom(Val::Px(TITLE_MARGIN_BOTTOM_PX)), ..Node::default() },
        ChildOf(root),
    ));

    spawn_section_header(&mut commands, root, "AUDIO");
    spawn_volume_row(&mut commands, root, "Master", VolumeDisplay::Master,
        VolumeButton::MasterDown, VolumeButton::MasterUp, settings.master_volume);
    spawn_volume_row(&mut commands, root, "BGM", VolumeDisplay::Bgm,
        VolumeButton::BgmDown, VolumeButton::BgmUp, settings.bgm_volume);
    spawn_volume_row(&mut commands, root, "SFX", VolumeDisplay::Sfx,
        VolumeButton::SfxDown, VolumeButton::SfxUp, settings.sfx_volume);

    spawn_section_header(&mut commands, root, "VIDEO");
    spawn_fullscreen_row(&mut commands, root, settings.fullscreen);

    // Bottom nav row
    let nav = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(NAV_MARGIN_TOP_PX)),
            ..Node::default()
        },
        ChildOf(root),
    )).id();

    spawn_nav_btn(&mut commands, nav, KeybindsNavButton, "Key Bindings", theme::BUTTON_BG);
    spawn_nav_btn(&mut commands, nav, SettingsBackButton, "Back", theme::DIALOG_CHOICE_BG);
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn handle_volume_buttons(
    mut q: Query<(&Interaction, &mut BackgroundColor, &VolumeButton), Changed<Interaction>>,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, mut bg, btn) in &mut q {
        match interaction {
            Interaction::Pressed => adjust_volume(&mut settings, *btn),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

pub fn handle_fullscreen_button(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<FullscreenButton>)>,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => settings.fullscreen = !settings.fullscreen,
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

pub fn handle_keybinds_nav(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<KeybindsNavButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => next_state.set(GameState::KeybindConfig),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
}

pub fn handle_back(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SettingsBackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => next_state.set(GameState::Paused),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

/// Keeps volume text labels in sync with [`GameSettings`].
pub fn sync_displays(
    settings: Res<GameSettings>,
    mut vol_q: Query<(&mut Text, &VolumeDisplay)>,
    mut fs_q: Query<&mut Text, (With<FullscreenDisplay>, Without<VolumeDisplay>)>,
) {
    if !settings.is_changed() {
        return;
    }
    for (mut text, display) in &mut vol_q {
        let val = match display {
            VolumeDisplay::Master => settings.master_volume,
            VolumeDisplay::Bgm => settings.bgm_volume,
            VolumeDisplay::Sfx => settings.sfx_volume,
        };
        **text = format!("{val}");
    }
    for mut text in &mut fs_q {
        **text = if settings.fullscreen { "On" } else { "Off" }.to_owned();
    }
}

/// Applies fullscreen setting to the Bevy window whenever settings change.
pub fn apply_fullscreen(
    settings: Res<GameSettings>,
    mut window_q: Query<&mut Window>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut window in &mut window_q {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn adjust_volume(settings: &mut GameSettings, btn: VolumeButton) {
    match btn {
        VolumeButton::MasterDown => settings.master_volume = settings.master_volume.saturating_sub(1),
        VolumeButton::MasterUp => settings.master_volume = settings.master_volume.saturating_add(1).min(MAX_VOLUME),
        VolumeButton::BgmDown => settings.bgm_volume = settings.bgm_volume.saturating_sub(1),
        VolumeButton::BgmUp => settings.bgm_volume = settings.bgm_volume.saturating_add(1).min(MAX_VOLUME),
        VolumeButton::SfxDown => settings.sfx_volume = settings.sfx_volume.saturating_sub(1),
        VolumeButton::SfxUp => settings.sfx_volume = settings.sfx_volume.saturating_add(1).min(MAX_VOLUME),
    }
}

fn spawn_section_header(commands: &mut Commands, parent: Entity, label: &str) {
    commands.spawn((
        Text::new(label),
        TextColor(theme::ACCENT),
        TextFont { font_size: SECTION_FONT_SIZE_PX, ..default() },
        Node {
            margin: UiRect::new(
                Val::ZERO, Val::ZERO,
                Val::Px(SECTION_MARGIN_TOP_PX), Val::Px(SECTION_MARGIN_BOTTOM_PX),
            ),
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_volume_row(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    display_marker: VolumeDisplay,
    btn_down: VolumeButton,
    btn_up: VolumeButton,
    value: u8,
) {
    let row = spawn_row(commands, parent);
    commands.spawn((
        Text::new(label),
        TextColor(theme::DIALOG_TEXT),
        TextFont { font_size: LABEL_FONT_SIZE_PX, ..default() },
        Node { flex_grow: 1.0, ..Node::default() },
        ChildOf(row),
    ));
    spawn_step_btn(commands, row, btn_down, "-");
    commands.spawn((
        display_marker,
        Text::new(format!("{value}")),
        TextColor(theme::DIALOG_SPEAKER),
        TextFont { font_size: VALUE_FONT_SIZE_PX, ..default() },
        Node {
            min_width: Val::Px(VALUE_MIN_WIDTH_PX),
            justify_content: JustifyContent::Center,
            ..Node::default()
        },
        ChildOf(row),
    ));
    spawn_step_btn(commands, row, btn_up, "+");
}

fn spawn_fullscreen_row(commands: &mut Commands, parent: Entity, fullscreen: bool) {
    let row = spawn_row(commands, parent);
    commands.spawn((
        Text::new("Fullscreen"),
        TextColor(theme::DIALOG_TEXT),
        TextFont { font_size: LABEL_FONT_SIZE_PX, ..default() },
        Node { flex_grow: 1.0, ..Node::default() },
        ChildOf(row),
    ));
    commands
        .spawn((
            FullscreenButton,
            Button,
            Node {
                min_width: Val::Px(STEP_BTN_SIZE_PX * 2.0),
                padding: UiRect::axes(Val::Px(STEP_BTN_SIZE_PX * 0.5), Val::ZERO),
                border_radius: BorderRadius::all(Val::Px(STEP_BTN_RADIUS_PX)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(row),
        ))
        .with_child((
            FullscreenDisplay,
            Text::new(if fullscreen { "On" } else { "Off" }),
            TextColor(theme::DIALOG_SPEAKER),
            TextFont { font_size: VALUE_FONT_SIZE_PX, ..default() },
        ));
}

fn spawn_row(commands: &mut Commands, parent: Entity) -> Entity {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            min_height: Val::Px(ROW_HEIGHT_PX),
            padding: UiRect::axes(Val::Px(ROW_PADDING_H_PX), Val::ZERO),
            margin: UiRect::bottom(Val::Px(ROW_MARGIN_BOTTOM_PX)),
            border_radius: BorderRadius::all(Val::Px(ROW_RADIUS_PX)),
            ..Node::default()
        },
        BackgroundColor(theme::DIALOG_CHOICE_BG),
        ChildOf(parent),
    )).id()
}

fn spawn_step_btn(commands: &mut Commands, parent: Entity, marker: VolumeButton, label: &str) {
    commands
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(STEP_BTN_SIZE_PX),
                height: Val::Px(STEP_BTN_SIZE_PX),
                border_radius: BorderRadius::all(Val::Px(STEP_BTN_RADIUS_PX)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(parent),
        ))
        .with_child((
            Text::new(label),
            TextColor(theme::DIALOG_TEXT),
            TextFont { font_size: STEP_BTN_FONT_SIZE_PX, ..default() },
        ));
}

fn spawn_nav_btn(
    commands: &mut Commands,
    parent: Entity,
    marker: impl Component,
    label: &str,
    bg: bevy::prelude::Color,
) {
    commands
        .spawn((
            marker,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(NAV_PADDING_H_PX), Val::Px(NAV_PADDING_V_PX)),
                margin: UiRect::right(Val::Px(NAV_MARGIN_RIGHT_PX)),
                border: UiRect::all(Val::Px(NAV_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(NAV_RADIUS_PX)),
                ..Node::default()
            },
            BorderColor::all(theme::ACCENT),
            BackgroundColor(bg),
            ChildOf(parent),
        ))
        .with_child((
            Text::new(label),
            TextColor(theme::BUTTON_TEXT),
            TextFont { font_size: NAV_FONT_SIZE_PX, ..default() },
        ));
}
