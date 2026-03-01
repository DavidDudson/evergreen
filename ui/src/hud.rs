use bevy::prelude::*;
use models::health::Health;

const HUD_OFFSET_PX: u16 = 5;
const HUD_FONT_SIZE_PX: u16 = 14;

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct HealthText;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Hud,
        Text::new("Health: --".to_string()),
        HealthText,
        TextFont {
            font_size: f32::from(HUD_FONT_SIZE_PX),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(f32::from(HUD_OFFSET_PX)),
            right: Val::Px(f32::from(HUD_OFFSET_PX)),
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
