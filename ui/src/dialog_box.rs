use bevy::prelude::*;
use dialog::events::{ChoiceMade, ChoicesReady, DialogueLineReady};
use dialog::locale::LocaleMap;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;

use crate::fonts::UiFont;
use crate::theme;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const BOX_WIDTH_PERCENT: f32 = 70.0;
const BOX_HEIGHT_PX: f32 = 180.0;
const BOX_BOTTOM_PX: f32 = 32.0;
const BOX_PADDING_H_PX: f32 = 24.0;
const BOX_PADDING_V_PX: f32 = 16.0;
const BOX_BORDER_PX: f32 = 2.0;
const BOX_RADIUS_PX: f32 = 8.0;

const SPEAKER_FONT_SIZE_PX: f32 = 14.0;
const SPEAKER_MARGIN_BOTTOM_PX: f32 = 6.0;
const TEXT_FONT_SIZE_PX: f32 = 18.0;
const HINT_FONT_SIZE_PX: f32 = 12.0;
const HINT_MARGIN_TOP_PX: f32 = 8.0;

const CHOICE_FONT_SIZE_PX: f32 = 16.0;
const CHOICE_PADDING_H_PX: f32 = 16.0;
const CHOICE_PADDING_V_PX: f32 = 8.0;
const CHOICE_MARGIN_TOP_PX: f32 = 4.0;
const CHOICE_RADIUS_PX: f32 = 4.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct DialogBox;

#[derive(Component)]
pub(crate) struct DialogSpeaker;

#[derive(Component)]
pub(crate) struct DialogBodyText;

#[derive(Component)]
pub(crate) struct DialogHint;

#[derive(Component)]
pub(crate) struct DialogChoicesContainer;

/// Marker for spawned choice buttons so they can be bulk-despawned.
#[derive(Component)]
pub(crate) struct ChoiceButton(usize);

/// Tracks the keyboard-selected choice index. Reset when choices change.
#[derive(Resource, Default)]
pub(crate) struct SelectedChoice {
    pub index: usize,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands, fonts: Res<UiFont>) {
    commands.insert_resource(SelectedChoice::default());
    commands
        .spawn((
            DialogBox,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOX_BOTTOM_PX),
                left: Val::Percent((100.0 - BOX_WIDTH_PERCENT) / 2.0),
                width: Val::Percent(BOX_WIDTH_PERCENT),
                min_height: Val::Px(BOX_HEIGHT_PX),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(
                    Val::Px(BOX_PADDING_H_PX),
                    Val::Px(BOX_PADDING_V_PX),
                ),
                border: UiRect::all(Val::Px(BOX_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(BOX_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_BG),
            BorderColor::all(theme::DIALOG_BORDER),
        ))
        .with_children(|parent| {
            parent.spawn((
                DialogSpeaker,
                Text::default(),
                TextColor(theme::DIALOG_SPEAKER),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: SPEAKER_FONT_SIZE_PX,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(SPEAKER_MARGIN_BOTTOM_PX)),
                    ..Node::default()
                },
            ));

            parent.spawn((
                DialogBodyText,
                Text::default(),
                TextColor(theme::DIALOG_TEXT),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: TEXT_FONT_SIZE_PX,
                    ..default()
                },
            ));

            parent.spawn((
                DialogChoicesContainer,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Node::default()
                },
            ));

            parent.spawn((
                DialogHint,
                Text::new(""),
                TextColor(theme::DIALOG_SPEAKER),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: HINT_FONT_SIZE_PX,
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(HINT_MARGIN_TOP_PX)),
                    ..Node::default()
                },
            ));
        });
}

// ---------------------------------------------------------------------------
// Update systems
// ---------------------------------------------------------------------------

/// Display a new speech line.
pub fn on_line_ready(
    mut events: MessageReader<DialogueLineReady>,
    locale: Res<LocaleMap>,
    mut speaker_q: Query<
        &mut Text,
        (
            With<DialogSpeaker>,
            Without<DialogBodyText>,
            Without<DialogHint>,
        ),
    >,
    mut body_q: Query<
        &mut Text,
        (
            With<DialogBodyText>,
            Without<DialogSpeaker>,
            Without<DialogHint>,
        ),
    >,
    mut hint_q: Query<
        &mut Text,
        (
            With<DialogHint>,
            Without<DialogSpeaker>,
            Without<DialogBodyText>,
        ),
    >,
    choice_q: Query<Entity, With<ChoiceButton>>,
    mut commands: Commands,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    // Remove any choice buttons from the previous step.
    for entity in &choice_q {
        commands.entity(entity).despawn();
    }

    if let Ok(mut speaker) = speaker_q.single_mut() {
        let name = event
            .speaker_key
            .as_deref()
            .map(|k| locale.get(k))
            .unwrap_or("");
        **speaker = name.to_string();
    }

    if let Ok(mut body) = body_q.single_mut() {
        **body = locale.get(&event.text_key).to_string();
    }

    if let Ok(mut hint) = hint_q.single_mut() {
        **hint = locale.get("ui.dialog.continue").to_string();
    }
}

/// Display choice buttons.
pub fn on_choices_ready(
    mut events: MessageReader<ChoicesReady>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
    mut selected: ResMut<SelectedChoice>,
    mut body_q: Query<
        &mut Text,
        (
            With<DialogBodyText>,
            Without<DialogSpeaker>,
            Without<DialogHint>,
        ),
    >,
    mut hint_q: Query<
        &mut Text,
        (
            With<DialogHint>,
            Without<DialogSpeaker>,
            Without<DialogBodyText>,
        ),
    >,
    container_q: Query<Entity, With<DialogChoicesContainer>>,
    choice_q: Query<Entity, With<ChoiceButton>>,
    mut commands: Commands,
) {
    let Some(event) = events.read().next() else {
        return;
    };

    // Clear body and hint while showing choices.
    if let Ok(mut body) = body_q.single_mut() {
        **body = String::new();
    }
    if let Ok(mut hint) = hint_q.single_mut() {
        **hint = String::new();
    }

    // Remove any existing choice buttons.
    for entity in &choice_q {
        commands.entity(entity).despawn();
    }

    let Ok(container) = container_q.single() else {
        return;
    };

    let choice_count = event.options.len();
    for (i, (index, text_key)) in event.options.iter().enumerate() {
        let label = locale.get(text_key).to_string();
        let bg = if i == 0 {
            theme::DIALOG_CHOICE_HOVER
        } else {
            theme::DIALOG_CHOICE_BG
        };
        commands
            .spawn((
                ChoiceButton(*index),
                Button,
                Node {
                    padding: UiRect::axes(
                        Val::Px(CHOICE_PADDING_H_PX),
                        Val::Px(CHOICE_PADDING_V_PX),
                    ),
                    margin: UiRect::top(Val::Px(CHOICE_MARGIN_TOP_PX)),
                    border_radius: BorderRadius::all(Val::Px(CHOICE_RADIUS_PX)),
                    justify_content: JustifyContent::FlexStart,
                    ..Node::default()
                },
                BackgroundColor(bg),
                ChildOf(container),
            ))
            .with_child((
                Text::new(label),
                TextColor(theme::DIALOG_TEXT),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: CHOICE_FONT_SIZE_PX,
                    ..default()
                },
            ));
    }

    selected.index = 0;
    selected.count = choice_count;
}

/// Handle choice button click and sync hover with keyboard selection.
pub fn handle_choice_interaction(
    mut interaction_q: Query<
        (&Interaction, &ChoiceButton),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedChoice>,
    mut writer: MessageWriter<ChoiceMade>,
    choice_q: Query<(Entity, &ChoiceButton)>,
    mut bg_q: Query<&mut BackgroundColor, With<ChoiceButton>>,
) {
    let mut needs_sync = false;

    for (interaction, choice) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                writer.write(ChoiceMade { index: choice.0 });
            }
            Interaction::Hovered => {
                let hovered_pos = choice_q
                    .iter()
                    .enumerate()
                    .find(|(_, (_, c))| c.0 == choice.0)
                    .map(|(i, _)| i);
                if let Some(pos) = hovered_pos {
                    selected.index = pos;
                    needs_sync = true;
                }
            }
            Interaction::None => {}
        }
    }

    if needs_sync {
        for (i, (entity, _)) in choice_q.iter().enumerate() {
            if let Ok(mut bg) = bg_q.get_mut(entity) {
                *bg = if i == selected.index {
                    BackgroundColor(theme::DIALOG_CHOICE_HOVER)
                } else {
                    BackgroundColor(theme::DIALOG_CHOICE_BG)
                };
            }
        }
    }
}

/// Navigate choices with keyboard (Up/Down/W/S to move, Enter/E to confirm).
pub fn handle_choice_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    mut selected: ResMut<SelectedChoice>,
    choice_q: Query<(Entity, &ChoiceButton)>,
    mut bg_q: Query<&mut BackgroundColor, With<ChoiceButton>>,
    mut writer: MessageWriter<ChoiceMade>,
) {
    if selected.count == 0 {
        return;
    }

    let mut moved = false;

    if keyboard.just_pressed(bindings.key(Action::MoveUp))
        || keyboard.just_pressed(KeyCode::ArrowUp)
    {
        if selected.index > 0 {
            selected.index -= 1;
            moved = true;
        }
    }

    if keyboard.just_pressed(bindings.key(Action::MoveDown))
        || keyboard.just_pressed(KeyCode::ArrowDown)
    {
        if selected.index + 1 < selected.count {
            selected.index += 1;
            moved = true;
        }
    }

    if moved {
        for (i, (entity, _)) in choice_q.iter().enumerate() {
            if let Ok(mut bg) = bg_q.get_mut(entity) {
                *bg = if i == selected.index {
                    BackgroundColor(theme::DIALOG_CHOICE_HOVER)
                } else {
                    BackgroundColor(theme::DIALOG_CHOICE_BG)
                };
            }
        }
    }

    if keyboard.just_pressed(bindings.key(Action::DialogAdvance))
        || keyboard.just_pressed(bindings.key(Action::Interact))
    {
        let target = choice_q
            .iter()
            .enumerate()
            .find(|(i, _)| *i == selected.index)
            .map(|(_, (_, c))| c.0);

        if let Some(index) = target {
            writer.write(ChoiceMade { index });
        }
    }
}
