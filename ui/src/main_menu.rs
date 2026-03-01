use bevy::prelude::*;
use models::game_states::GameState;

use crate::theme;

#[derive(Component)]
pub struct MainMenu;

pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            MainMenu,
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
            BackgroundColor(theme::DARK_BG),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Evergreen"),
                TextColor(theme::TITLE),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Node::default()
                },
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(14.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..Node::default()
                    },
                    BorderColor::all(theme::ACCENT),
                    BackgroundColor(theme::BUTTON_BG),
                ))
                .with_child((
                    Text::new("Begin Journey"),
                    TextColor(theme::BUTTON_TEXT),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                ));
        });
}

pub fn button_system(
    mut next_state: ResMut<NextState<GameState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    interaction_query
        .iter()
        .filter(|interaction| **interaction == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::Playing));
}
