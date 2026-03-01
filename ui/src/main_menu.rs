use bevy::prelude::*;
use models::game_states::GameState;

use crate::theme;

const TITLE_FONT_SIZE_PX: u16 = 72;
const TITLE_MARGIN_BOTTOM_PX: u16 = 8;
const BUTTON_FONT_SIZE_PX: u16 = 26;
const BUTTON_PADDING_H_PX: u16 = 40;
const BUTTON_PADDING_V_PX: u16 = 14;
const BUTTON_MARGIN_TOP_PX: u16 = 8;
const BUTTON_BORDER_PX: u16 = 2;
const BUTTON_RADIUS_PX: u16 = 6;

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
                    font_size: f32::from(TITLE_FONT_SIZE_PX),
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(f32::from(TITLE_MARGIN_BOTTOM_PX))),
                    ..Node::default()
                },
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(
                            Val::Px(f32::from(BUTTON_PADDING_H_PX)),
                            Val::Px(f32::from(BUTTON_PADDING_V_PX)),
                        ),
                        margin: UiRect::top(Val::Px(f32::from(BUTTON_MARGIN_TOP_PX))),
                        border: UiRect::all(Val::Px(f32::from(BUTTON_BORDER_PX))),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(f32::from(BUTTON_RADIUS_PX))),
                        ..Node::default()
                    },
                    BorderColor::all(theme::ACCENT),
                    BackgroundColor(theme::BUTTON_BG),
                ))
                .with_child((
                    Text::new("Begin Journey"),
                    TextColor(theme::BUTTON_TEXT),
                    TextFont {
                        font_size: f32::from(BUTTON_FONT_SIZE_PX),
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
