use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;
use keybinds::remap::{AwaitingRemap, CancelRemap, RemapCompleted, RequestRemap};
use keybinds::serialize::keycode_name;
use keybinds::IntoEnumIterator;
use models::game_states::GameState;

use dialog::locale::LocaleMap;

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

pub fn setup(
    mut commands: Commands,
    keybinds: Res<Keybinds>,
    fonts: Res<UiFont>,
    locale: Res<LocaleMap>,
) {
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
        Text::new(locale.get("ui.keybind.title").to_string()),
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
    for action in Action::iter() {
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
    crate::widgets::ButtonBuilder::new(
        locale.get("ui.keybind.back").to_string(),
        BackButton,
        font.clone(),
    )
    .padding(BACK_PADDING_H_PX, BACK_PADDING_V_PX)
    .font_size(BACK_FONT_SIZE_PX)
    .margin(0.0, 0.0)
    .spawn(&mut commands, bottom_row);

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
            Text::new(locale.get("ui.keybind.reset_all").to_string()),
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
                **text = keycode_name(keybinds.key(label.0))
                    .unwrap_or("?")
                    .to_string();
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
        **text = keycode_name(keybinds.key(label.0))
            .unwrap_or("?")
            .to_string();
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
                        "Press a key for \"{}\" -- Escape to cancel",
                        a.action
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
        Text::new(action.to_string()),
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
            Text::new(keycode_name(key).unwrap_or("?").to_string()),
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

pub struct KeybindScreenSetup;

impl crate::screen::ScreenSetup for KeybindScreenSetup {
    fn register(app: &mut bevy::prelude::App) {
        use bevy::prelude::*;
        use models::game_states::GameState;
        app.add_systems(OnEnter(GameState::KeybindConfig), setup)
            .add_systems(
                OnExit(GameState::KeybindConfig),
                crate::despawn::despawn_all::<KeybindScreen>,
            )
            .add_systems(
                Update,
                (
                    handle_key_buttons,
                    handle_reset_buttons,
                    handle_reset_all,
                    handle_back,
                    refresh_key_labels,
                    sync_all_labels,
                    sync_remap_overlay,
                )
                    .run_if(in_state(GameState::KeybindConfig)),
            );
    }
}
