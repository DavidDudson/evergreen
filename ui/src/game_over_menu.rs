use bevy::prelude::*;

#[derive(Component)]
pub struct GameOverMenu;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        GameOverMenu,
        Text::new("Game Over".to_string()),
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..Node::default()
        },
    ));
}
