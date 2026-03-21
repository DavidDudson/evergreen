use bevy::prelude::*;
use dialog::events::{ChoiceMade, ChoicesReady, DialogueLineReady};
use dialog::locale::LocaleMap;

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

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands) {
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

    for (index, text_key) in &event.options {
        let label = locale.get(text_key).to_string();
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
                BackgroundColor(theme::DIALOG_CHOICE_BG),
                ChildOf(container),
            ))
            .with_child((
                Text::new(label),
                TextColor(theme::DIALOG_TEXT),
                TextFont {
                    font_size: CHOICE_FONT_SIZE_PX,
                    ..default()
                },
            ));
    }
}

/// Handle choice button hover highlight and click.
pub fn handle_choice_interaction(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor, &ChoiceButton),
        Changed<Interaction>,
    >,
    mut writer: MessageWriter<ChoiceMade>,
) {
    for (interaction, mut bg, choice) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                writer.write(ChoiceMade { index: choice.0 });
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(theme::DIALOG_CHOICE_BG);
            }
        }
    }
}
