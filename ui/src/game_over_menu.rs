use bevy::prelude::*;
use dialog::locale::LocaleMap;

#[derive(Component)]
pub struct GameOverMenu;

pub fn setup(mut commands: Commands, locale: Res<LocaleMap>) {
    commands.spawn((
        GameOverMenu,
        Text::new(locale.get("ui.game_over.title").to_string()),
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..Node::default()
        },
    ));
}

pub struct GameOverScreen;

impl crate::screen::ScreenSetup for GameOverScreen {
    fn register(app: &mut bevy::prelude::App) {
        use bevy::prelude::*;
        use models::game_states::GameState;
        app.add_systems(OnEnter(GameState::GameOver), setup)
            .add_systems(
                OnExit(GameState::GameOver),
                crate::despawn::despawn_all::<GameOverMenu>,
            );
    }
}
