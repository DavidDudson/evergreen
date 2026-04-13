use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;
use keybinds::remap::{AwaitingRemap, CancelRemap, RemapCompleted, RequestRemap};
use models::game_states::GameState;

use crate::fonts::UiFont;
use crate::theme;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const PAGE_PADDING_PX: f32 = 40.0;
const TITLE_FONT_SIZE_PX: f32 = 32.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 24.0;

const ROW_HEIGHT_PX: f32 = 44.0;
const ROW_MARGIN_BOTTOM_PX: f32 = 4.0;
const ROW_PADDING_H_PX: f32 = 16.0;
const ROW_RADIUS_PX: f32 = 4.0;

const LABEL_FONT_SIZE_PX: f32 = 16.0;
const KEY_FONT_SIZE_PX: f32 = 16.0;
const KEY_MIN_WIDTH_PX: f32 = 120.0;
const KEY_PADDING_H_PX: f32 = 12.0;
const KEY_RADIUS_PX: f32 = 4.0;

const RESET_FONT_SIZE_PX: f32 = 14.0;
const RESET_PADDING_H_PX: f32 = 12.0;
const RESET_PADDING_V_PX: f32 = 6.0;
const RESET_MARGIN_LEFT_PX: f32 = 8.0;
const RESET_RADIUS_PX: f32 = 4.0;

const HINT_FONT_SIZE_PX: f32 = 13.0;
const HINT_MARGIN_TOP_PX: f32 = 16.0;

const BACK_FONT_SIZE_PX: f32 = 18.0;
const BACK_PADDING_H_PX: f32 = 24.0;
const BACK_PADDING_V_PX: f32 = 10.0;
const BACK_MARGIN_TOP_PX: f32 = 20.0;
const BACK_BORDER_PX: f32 = 2.0;
const BACK_RADIUS_PX: f32 = 6.0;

const OVERLAY_FONT_SIZE_PX: f32 = 24.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct KeybindScreen;

/// The key-button for a specific action row.
#[derive(Component)]
pub(crate) struct KeyButton(Action);

/// The key label text inside a KeyButton.
#[derive(Component)]
pub(crate) struct KeyButtonLabel(Action);

/// "Reset" button on an action row.
#[derive(Component)]
pub(crate) struct ResetButton(Action);

/// "Reset All" button at the bottom.
#[derive(Component)]
pub(crate) struct ResetAllButton;

/// Back button.
#[derive(Component)]
pub(crate) struct BackButton;

/// Overlay shown when waiting for a keypress.
#[derive(Component)]
pub(crate) struct RemapOverlay;

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands, keybinds: Res<Keybinds>, fonts: Res<UiFont>) {
    let root = commands
        .spawn((
            KeybindScreen,
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

    let font = fonts.0.clone();

    // Title
    commands.spawn((
        Text::new("Key Bindings"),
        TextColor(theme::TITLE),
        TextFont {
            font: font.clone(),
            font_size: TITLE_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(TITLE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(root),
    ));

    // Action rows
    for &action in Action::ALL {
        let key = keybinds.key(action);
        spawn_action_row(&mut commands, root, action, key, font.clone());
    }

    // Hint text
    commands.spawn((
        Text::new("Click a key button, then press the new key. Escape cancels."),
        TextColor(theme::DIALOG_SPEAKER),
        TextFont {
            font: font.clone(),
            font_size: HINT_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::top(Val::Px(HINT_MARGIN_TOP_PX)),
            ..Node::default()
        },
        ChildOf(root),
    ));

    // Bottom button row
    let bottom_row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(BACK_MARGIN_TOP_PX)),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    // Back button
    commands
        .spawn((
            BackButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(BACK_PADDING_H_PX), Val::Px(BACK_PADDING_V_PX)),
                border: UiRect::all(Val::Px(BACK_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(BACK_RADIUS_PX)),
                ..Node::default()
            },
            BorderColor::all(theme::ACCENT),
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(bottom_row),
        ))
        .with_child((
            Text::new("Back"),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font: font.clone(),
                font_size: BACK_FONT_SIZE_PX,
                ..default()
            },
        ));

    // Reset All button
    commands
        .spawn((
            ResetAllButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(RESET_PADDING_H_PX), Val::Px(RESET_PADDING_V_PX)),
                margin: UiRect::left(Val::Px(BACK_PADDING_H_PX)),
                border_radius: BorderRadius::all(Val::Px(RESET_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(bottom_row),
        ))
        .with_child((
            Text::new("Reset All"),
            TextColor(theme::DIALOG_TEXT),
            TextFont {
                font,
                font_size: RESET_FONT_SIZE_PX,
                ..default()
            },
        ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Handles key-button clicks: sends `RequestRemap` and shows the overlay.
pub fn handle_key_buttons(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor, &KeyButton),
        Changed<Interaction>,
    >,
    mut writer: MessageWriter<RequestRemap>,
) {
    for (interaction, mut bg, btn) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                writer.write(RequestRemap { action: btn.0 });
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

/// Handles per-action reset button clicks.
pub fn handle_reset_buttons(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor, &ResetButton),
        Changed<Interaction>,
    >,
    mut keybinds: ResMut<Keybinds>,
) {
    for (interaction, mut bg, btn) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => keybinds.reset_action(btn.0),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

/// Handles "Reset All" button.
#[allow(clippy::type_complexity)]
pub fn handle_reset_all(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResetAllButton>),
    >,
    mut keybinds: ResMut<Keybinds>,
) {
    for (interaction, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => keybinds.reset_all(),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::DIALOG_CHOICE_BG),
        }
    }
}

/// Handles Back button.
#[allow(clippy::type_complexity)]
pub fn handle_back(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut cancel_writer: MessageWriter<CancelRemap>,
) {
    for (interaction, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                cancel_writer.write(CancelRemap);
                next_state.set(GameState::Settings);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
}

/// After a remap completes, refresh the key label for the affected action.
pub fn refresh_key_labels(
    mut events: MessageReader<RemapCompleted>,
    keybinds: Res<Keybinds>,
    mut label_q: Query<(&mut Text, &KeyButtonLabel)>,
) {
    for event in events.read() {
        for (mut text, label) in &mut label_q {
            if label.0 == event.action {
                **text = keycode_label(keybinds.key(label.0)).to_string();
            }
        }
    }
}

/// Refresh ALL key labels when keybinds change (covers reset_all).
pub fn sync_all_labels(keybinds: Res<Keybinds>, mut label_q: Query<(&mut Text, &KeyButtonLabel)>) {
    if !keybinds.is_changed() {
        return;
    }
    for (mut text, label) in &mut label_q {
        **text = keycode_label(keybinds.key(label.0)).to_string();
    }
}

/// Show/hide the "awaiting remap" overlay.
pub fn sync_remap_overlay(
    awaiting: Option<Res<AwaitingRemap>>,
    overlay_q: Query<Entity, With<RemapOverlay>>,
    screen_q: Query<Entity, With<KeybindScreen>>,
    fonts: Res<UiFont>,
    mut commands: Commands,
) {
    let overlay_exists = !overlay_q.is_empty();

    match (awaiting.as_deref(), overlay_exists) {
        (Some(a), false) => {
            // Spawn overlay
            let Ok(root) = screen_q.single() else { return };
            commands
                .spawn((
                    RemapOverlay,
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Node::default()
                    },
                    BackgroundColor(theme::OVERLAY),
                    ChildOf(root),
                ))
                .with_child((
                    Text::new(format!(
                        "Press a key for \"{}\" — Escape to cancel",
                        a.action.label()
                    )),
                    TextColor(theme::DIALOG_TEXT),
                    TextFont {
                        font: fonts.0.clone(),
                        font_size: OVERLAY_FONT_SIZE_PX,
                        ..default()
                    },
                ));
        }
        (None, true) => {
            // Despawn overlay
            for entity in &overlay_q {
                commands.entity(entity).despawn();
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_action_row(
    commands: &mut Commands,
    parent: Entity,
    action: Action,
    key: KeyCode,
    font: Handle<Font>,
) {
    let row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                min_height: Val::Px(ROW_HEIGHT_PX),
                padding: UiRect::axes(Val::Px(ROW_PADDING_H_PX), Val::ZERO),
                margin: UiRect::bottom(Val::Px(ROW_MARGIN_BOTTOM_PX)),
                border_radius: BorderRadius::all(Val::Px(ROW_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(parent),
        ))
        .id();

    // Action label
    commands.spawn((
        Text::new(action.label()),
        TextColor(theme::DIALOG_TEXT),
        TextFont {
            font: font.clone(),
            font_size: LABEL_FONT_SIZE_PX,
            ..default()
        },
        Node {
            flex_grow: 1.0,
            ..Node::default()
        },
        ChildOf(row),
    ));

    // Key button
    commands
        .spawn((
            KeyButton(action),
            Button,
            Node {
                min_width: Val::Px(KEY_MIN_WIDTH_PX),
                padding: UiRect::axes(Val::Px(KEY_PADDING_H_PX), Val::ZERO),
                border_radius: BorderRadius::all(Val::Px(KEY_RADIUS_PX)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(row),
        ))
        .with_child((
            KeyButtonLabel(action),
            Text::new(keycode_label(key).to_string()),
            TextColor(theme::DIALOG_SPEAKER),
            TextFont {
                font: font.clone(),
                font_size: KEY_FONT_SIZE_PX,
                ..default()
            },
        ));

    // Reset button
    commands
        .spawn((
            ResetButton(action),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(RESET_PADDING_H_PX), Val::Px(RESET_PADDING_V_PX)),
                margin: UiRect::left(Val::Px(RESET_MARGIN_LEFT_PX)),
                border_radius: BorderRadius::all(Val::Px(RESET_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_CHOICE_BG),
            ChildOf(row),
        ))
        .with_child((
            Text::new("Reset"),
            TextColor(theme::DIALOG_TEXT),
            TextFont {
                font,
                font_size: RESET_FONT_SIZE_PX,
                ..default()
            },
        ));
}

/// Short human-readable label for a [`KeyCode`].
pub fn keycode_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Backspace => "Backspace",
        KeyCode::Tab => "Tab",
        KeyCode::ShiftLeft => "L-Shift",
        KeyCode::ShiftRight => "R-Shift",
        KeyCode::ControlLeft => "L-Ctrl",
        KeyCode::ControlRight => "R-Ctrl",
        KeyCode::AltLeft => "L-Alt",
        KeyCode::AltRight => "R-Alt",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        _ => "?",
    }
}
