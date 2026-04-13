use bevy::prelude::*;
use models::game_states::GameState;

use crate::fonts::UiFont;
use crate::settings_screen::SettingsOrigin;
use crate::theme;

const TITLE_FONT_SIZE_PX: f32 = 48.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 32.0;
const BUTTON_FONT_SIZE_PX: f32 = 22.0;
const BUTTON_PADDING_H_PX: f32 = 32.0;
const BUTTON_PADDING_V_PX: f32 = 12.0;
const BUTTON_MARGIN_BOTTOM_PX: f32 = 12.0;
const BUTTON_BORDER_PX: f32 = 2.0;
const BUTTON_RADIUS_PX: f32 = 6.0;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub(crate) struct ResumeButton;

#[derive(Component)]
pub(crate) struct SettingsButton;

pub fn setup(mut commands: Commands, fonts: Res<UiFont>) {
    let root = commands
        .spawn((
            PauseMenu,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            BackgroundColor(theme::OVERLAY),
        ))
        .id();

    // Title
    commands.spawn((
        Text::new("Paused"),
        TextColor(theme::TITLE),
        TextFont {
            font: fonts.0.clone(),
            font_size: TITLE_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(TITLE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(root),
    ));

    spawn_button(&mut commands, root, ResumeButton, "Resume", fonts.0.clone());
    spawn_button(
        &mut commands,
        root,
        SettingsButton,
        "Settings",
        fonts.0.clone(),
    );
}

#[allow(clippy::type_complexity)]
pub fn handle_resume(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ResumeButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => next_state.set(GameState::Playing),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_settings_button(
    mut q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut origin: ResMut<SettingsOrigin>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => {
                *origin = SettingsOrigin::Paused;
                next_state.set(GameState::Settings);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
}

fn spawn_button(
    commands: &mut Commands,
    parent: Entity,
    marker: impl Component,
    label: &str,
    font: Handle<Font>,
) {
    commands
        .spawn((
            marker,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(BUTTON_PADDING_H_PX), Val::Px(BUTTON_PADDING_V_PX)),
                margin: UiRect::bottom(Val::Px(BUTTON_MARGIN_BOTTOM_PX)),
                border: UiRect::all(Val::Px(BUTTON_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(BUTTON_RADIUS_PX)),
                justify_content: JustifyContent::Center,
                ..Node::default()
            },
            BorderColor::all(theme::ACCENT),
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(parent),
        ))
        .with_child((
            Text::new(label),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font,
                font_size: BUTTON_FONT_SIZE_PX,
                ..default()
            },
        ));
}
