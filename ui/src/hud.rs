use bevy::prelude::*;
use models::health::Health;

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct HealthText;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Hud,
        Text::new("Health: --".to_string()),
        HealthText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..Node::default()
        },
    ));
}

pub fn update_health_text(
    castle_query: Query<&Health>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    if let Some((health, mut text)) = castle_query.single().ok().zip(text_query.single_mut().ok()) {
        text.0 = format!("Health: {}", health.0);
    }
}
