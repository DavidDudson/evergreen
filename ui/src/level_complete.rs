use bevy::prelude::*;
use models::game_states::GameState;
use models::palette;

use crate::fonts::UiFont;

const TITLE_FONT_SIZE_PX: f32 = 28.0;
const BUTTON_FONT_SIZE_PX: f32 = 16.0;
const BUTTON_PADDING_H_PX: f32 = 24.0;
const BUTTON_PADDING_V_PX: f32 = 10.0;

#[derive(Component)]
pub struct LevelCompleteScreen;

#[derive(Component)]
pub(crate) struct PlayAgainButton;

pub fn setup(mut commands: Commands, font: Res<UiFont>) {
    let root = commands
        .spawn((
            LevelCompleteScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..Node::default()
            },
            BackgroundColor(palette::DARK_BG),
        ))
        .id();

    commands.spawn((
        Text::new("Level Complete"),
        TextFont {
            font: font.0.clone(),
            font_size: TITLE_FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::TITLE),
        ChildOf(root),
    ));

    commands.spawn((
        PlayAgainButton,
        Button,
        Text::new("Play Again"),
        TextFont {
            font: font.0.clone(),
            font_size: BUTTON_FONT_SIZE_PX,
            ..default()
        },
        TextColor(palette::BUTTON_TEXT),
        Node {
            padding: UiRect::axes(Val::Px(BUTTON_PADDING_H_PX), Val::Px(BUTTON_PADDING_V_PX)),
            ..Node::default()
        },
        BackgroundColor(palette::BUTTON_BG),
        ChildOf(root),
    ));
}

pub fn handle_play_again(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<PlayAgainButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
}
