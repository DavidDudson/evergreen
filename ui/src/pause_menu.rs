use bevy::prelude::*;

use crate::theme;

const PAUSE_FONT_SIZE_PX: u16 = 48;

#[derive(Component)]
pub struct PauseMenu;

pub fn setup(mut commands: Commands) {
    commands
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
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextColor(theme::TITLE),
                TextFont {
                    font_size: f32::from(PAUSE_FONT_SIZE_PX),
                    ..default()
                },
            ));
        });
}
