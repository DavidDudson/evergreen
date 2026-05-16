//! Investigation popup: a one-line description that surfaces when the player
//! investigates a world object via [`quest::interact::Investigatable`].
//!
//! Always mounted while in `GameState::Playing`; hidden by default and shown
//! for [`POPUP_DURATION_SECS`] after each [`InvestigationFired`] event.

use bevy::prelude::*;
use dialog::locale::LocaleMap;
use models::game_states::{should_despawn_world, GameState};
use quest::interact::InvestigationFired;

use crate::fonts::UiFont;
use crate::theme;

const POPUP_DURATION_SECS: f32 = 5.0;
const POPUP_WIDTH_PERCENT: f32 = 60.0;
const POPUP_TOP_PX: f32 = 80.0;
const POPUP_PADDING_H_PX: f32 = 20.0;
const POPUP_PADDING_V_PX: f32 = 14.0;
const POPUP_BORDER_PX: f32 = 2.0;
const POPUP_RADIUS_PX: f32 = 6.0;
const POPUP_FONT_SIZE_PX: f32 = 16.0;

/// Marker for the popup root node.
#[derive(Component)]
pub struct InvestigatePopup;

/// Marker for the popup body text node.
#[derive(Component)]
pub struct InvestigatePopupText;

/// Counts down while the popup is visible; hidden when it hits zero.
#[derive(Component)]
pub struct InvestigatePopupTimer(pub Timer);

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
                (on_investigation, tick_popup).run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup(mut commands: Commands, fonts: Res<UiFont>) {
    commands
        .spawn((
            InvestigatePopup,
            InvestigatePopupTimer(Timer::from_seconds(
                POPUP_DURATION_SECS,
                TimerMode::Once,
            )),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(POPUP_TOP_PX),
                left: Val::Percent((100.0 - POPUP_WIDTH_PERCENT) / 2.0),
                width: Val::Percent(POPUP_WIDTH_PERCENT),
                padding: UiRect::axes(
                    Val::Px(POPUP_PADDING_H_PX),
                    Val::Px(POPUP_PADDING_V_PX),
                ),
                border: UiRect::all(Val::Px(POPUP_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(POPUP_RADIUS_PX)),
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
                    font_size: POPUP_FONT_SIZE_PX,
                    ..default()
                },
                TextColor(theme::DIALOG_TEXT),
            ));
        });
}

fn on_investigation(
    mut events: MessageReader<InvestigationFired>,
    locale: Res<LocaleMap>,
    mut popup_q: Query<(&mut Visibility, &mut InvestigatePopupTimer), With<InvestigatePopup>>,
    mut text_q: Query<&mut Text, With<InvestigatePopupText>>,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    let Ok((mut visibility, mut timer)) = popup_q.single_mut() else {
        return;
    };
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let body = locale.get(&event.description_key);
    text.0 = body.to_string();
    *visibility = Visibility::Inherited;
    timer.0.reset();
}

fn tick_popup(
    time: Res<Time>,
    mut popup_q: Query<(&mut Visibility, &mut InvestigatePopupTimer), With<InvestigatePopup>>,
) {
    let Ok((mut visibility, mut timer)) = popup_q.single_mut() else {
        return;
    };
    if matches!(*visibility, Visibility::Hidden) {
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        *visibility = Visibility::Hidden;
    }
}
