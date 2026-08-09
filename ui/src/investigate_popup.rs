//! Investigation popup: a description plus optional follow-up choice buttons
//! shown when the player investigates a world object via
//! [`quest::interact::Investigatable`].
//!
//! Two flows:
//! - **No choices** -- description shown, popup auto-hides after
//!   [`POPUP_DURATION_SECS`].
//! - **With choices** -- description plus N buttons; picking one fires
//!   [`InvestigateChoiceMade`], then a short response is shown before hide.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use dialog::locale::LocaleMap;
use models::game_states::{should_despawn_world, GameState};
use quest::interact::{InvestigateChoice, InvestigateChoiceMade, InvestigationFired};

use crate::fonts::UiFont;
use crate::theme;

const POPUP_DURATION_SECS: f32 = 5.0;
const RESPONSE_DURATION_SECS: f32 = 3.0;
const POPUP_WIDTH_PERCENT: f32 = 60.0;
const POPUP_TOP_PX: f32 = 80.0;
const POPUP_PADDING_H_PX: f32 = 20.0;
const POPUP_PADDING_V_PX: f32 = 14.0;
const POPUP_BORDER_PX: f32 = 2.0;
const POPUP_RADIUS_PX: f32 = 6.0;
const POPUP_BODY_FONT_PX: f32 = 16.0;
const POPUP_BUTTON_FONT_PX: f32 = 14.0;
const POPUP_BUTTON_PADDING_H_PX: f32 = 14.0;
const POPUP_BUTTON_PADDING_V_PX: f32 = 6.0;
const POPUP_BUTTON_RADIUS_PX: f32 = 4.0;
const POPUP_BUTTON_GAP_PX: f32 = 6.0;
const POPUP_BODY_MARGIN_BOTTOM_PX: f32 = 10.0;

#[derive(Component)]
pub struct InvestigatePopup;

#[derive(Component)]
pub struct InvestigatePopupText;

#[derive(Component)]
pub struct InvestigateChoiceButton(pub usize);

#[derive(Component)]
pub struct InvestigatePopupChoicesRow;

/// Popup runtime state: timer + the choices currently presented (so a click
/// on a button can map back to the original [`InvestigateChoice`]).
#[derive(Component, Default)]
pub struct InvestigatePopupState {
    pub timer: Timer,
    pub choices: Vec<InvestigateChoice>,
}

pub struct InvestigatePopupScreen;

impl crate::screen::ScreenSetup for InvestigatePopupScreen {
    fn register(app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(
                OnExit(GameState::Playing),
                crate::despawn::despawn_all::<InvestigatePopup>.run_if(should_despawn_world),
            )
            .add_systems(
                Update,
                (on_investigation, on_button_click, tick_popup)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup(mut commands: Commands, fonts: Res<UiFont>) {
    commands
        .spawn((
            InvestigatePopup,
            InvestigatePopupState {
                timer: Timer::from_seconds(POPUP_DURATION_SECS, TimerMode::Once),
                choices: Vec::new(),
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(POPUP_TOP_PX),
                left: Val::Percent((100.0 - POPUP_WIDTH_PERCENT) / 2.0),
                width: Val::Percent(POPUP_WIDTH_PERCENT),
                padding: UiRect::axes(Val::Px(POPUP_PADDING_H_PX), Val::Px(POPUP_PADDING_V_PX)),
                border: UiRect::all(Val::Px(POPUP_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(POPUP_RADIUS_PX)),
                flex_direction: FlexDirection::Column,
                ..Node::default()
            },
            BackgroundColor(theme::DIALOG_BG),
            BorderColor::all(theme::DIALOG_BORDER),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                InvestigatePopupText,
                Text::new(""),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: POPUP_BODY_FONT_PX,
                    ..default()
                },
                TextColor(theme::DIALOG_TEXT),
                Node {
                    margin: UiRect::bottom(Val::Px(POPUP_BODY_MARGIN_BOTTOM_PX)),
                    ..Node::default()
                },
            ));
            parent.spawn((
                InvestigatePopupChoicesRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(POPUP_BUTTON_GAP_PX),
                    flex_wrap: FlexWrap::Wrap,
                    ..Node::default()
                },
            ));
        });
}

/// Grouped popup queries so [`on_investigation`] stays under the clippy
/// `too_many_arguments` threshold.
#[derive(SystemParam)]
struct PopupQueries<'w, 's> {
    popup: Query<
        'w,
        's,
        (&'static mut Visibility, &'static mut InvestigatePopupState),
        With<InvestigatePopup>,
    >,
    text: Query<'w, 's, &'static mut Text, With<InvestigatePopupText>>,
    row: Query<'w, 's, Entity, With<InvestigatePopupChoicesRow>>,
    existing_buttons: Query<'w, 's, Entity, With<InvestigateChoiceButton>>,
}

#[allow(clippy::type_complexity)]
fn on_investigation(
    mut events: MessageReader<InvestigationFired>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
    mut commands: Commands,
    mut q: PopupQueries,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    let Ok((mut visibility, mut state)) = q.popup.single_mut() else {
        return;
    };
    let Ok(mut text) = q.text.single_mut() else {
        return;
    };

    text.0 = locale.get(&event.description_key).to_string();
    *visibility = Visibility::Inherited;
    state.timer = Timer::from_seconds(POPUP_DURATION_SECS, TimerMode::Once);
    state.choices = event.choices.clone();

    // Clear existing buttons.
    for entity in &q.existing_buttons {
        commands.entity(entity).despawn();
    }
    if event.choices.is_empty() {
        return;
    }
    let Ok(row) = q.row.single() else {
        return;
    };
    for (idx, choice) in event.choices.iter().enumerate() {
        let label = locale.get(&choice.text_key).to_string();
        let button = commands
            .spawn((
                InvestigateChoiceButton(idx),
                Button,
                Node {
                    padding: UiRect::axes(
                        Val::Px(POPUP_BUTTON_PADDING_H_PX),
                        Val::Px(POPUP_BUTTON_PADDING_V_PX),
                    ),
                    border_radius: BorderRadius::all(Val::Px(POPUP_BUTTON_RADIUS_PX)),
                    ..Node::default()
                },
                BackgroundColor(theme::DIALOG_CHOICE_BG),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(label),
                    TextFont {
                        font: fonts.0.clone(),
                        font_size: POPUP_BUTTON_FONT_PX,
                        ..default()
                    },
                    TextColor(theme::DIALOG_TEXT),
                ));
            })
            .id();
        commands.entity(row).add_child(button);
    }
}

#[allow(clippy::type_complexity)]
fn on_button_click(
    mut interaction_q: Query<(&Interaction, &InvestigateChoiceButton), Changed<Interaction>>,
    locale: Res<LocaleMap>,
    mut commands: Commands,
    mut popup_q: Query<&mut InvestigatePopupState, With<InvestigatePopup>>,
    mut text_q: Query<&mut Text, With<InvestigatePopupText>>,
    button_q: Query<Entity, With<InvestigateChoiceButton>>,
    mut writer: MessageWriter<InvestigateChoiceMade>,
) {
    for (interaction, button) in &mut interaction_q {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let Ok(mut state) = popup_q.single_mut() else {
            return;
        };
        let Some(choice) = state.choices.get(button.0).cloned() else {
            continue;
        };
        // Swap body text to the response, drop buttons, restart shorter timer.
        if let Some(response_key) = &choice.response_key {
            if let Ok(mut text) = text_q.single_mut() {
                text.0 = locale.get(response_key).to_string();
            }
        }
        for entity in &button_q {
            commands.entity(entity).despawn();
        }
        state.choices.clear();
        state.timer = Timer::from_seconds(RESPONSE_DURATION_SECS, TimerMode::Once);
        writer.write(InvestigateChoiceMade { choice });
    }
}

fn tick_popup(
    time: Res<Time>,
    mut popup_q: Query<(&mut Visibility, &mut InvestigatePopupState), With<InvestigatePopup>>,
) {
    let Ok((mut visibility, mut state)) = popup_q.single_mut() else {
        return;
    };
    if matches!(*visibility, Visibility::Hidden) {
        return;
    }
    // Don't auto-hide while the player still has a choice to make.
    if !state.choices.is_empty() {
        return;
    }
    state.timer.tick(time.delta());
    if state.timer.is_finished() {
        *visibility = Visibility::Hidden;
    }
}
