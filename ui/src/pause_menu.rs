use bevy::prelude::*;
use dialog::locale::LocaleMap;
use models::game_states::GameState;

use crate::fonts::UiFont;
use crate::settings_screen::SettingsOrigin;
use crate::theme;
use crate::widgets::ButtonBuilder;

const TITLE_FONT_SIZE_PX: f32 = 48.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 32.0;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub(crate) struct ResumeButton;

#[derive(Component)]
pub(crate) struct SettingsButton;

#[derive(Component)]
pub(crate) struct QuitToMenuButton;

/// Flag set by the pause-menu quit button. Picked up one frame later by
/// `handle_quit_pending` (running in `Playing`) to transition to `MainMenu`,
/// which triggers the normal `OnExit(Playing)` world cleanup.
#[derive(Resource, Default)]
pub struct QuitToMenuRequested(pub bool);

pub fn setup(mut commands: Commands, fonts: Res<UiFont>, locale: Res<LocaleMap>) {
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
        Text::new(locale.get("ui.pause.title").to_string()),
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

    ButtonBuilder::new(
        locale.get("ui.pause.resume").to_string(),
        ResumeButton,
        fonts.0.clone(),
    )
    .spawn(&mut commands, root);
    ButtonBuilder::new(
        locale.get("ui.pause.settings").to_string(),
        SettingsButton,
        fonts.0.clone(),
    )
    .spawn(&mut commands, root);
    ButtonBuilder::new(
        locale.get("ui.pause.quit_to_menu").to_string(),
        QuitToMenuButton,
        fonts.0.clone(),
    )
    .spawn(&mut commands, root);
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

/// Flip `QuitToMenuRequested` + bounce out of Paused into Playing. The next
/// frame's `handle_quit_pending` sees the flag while in Playing and transitions
/// to MainMenu, which fires `OnExit(Playing)` cleanup via `should_despawn_world`.
#[allow(clippy::type_complexity)]
pub fn handle_quit_to_menu_button(
    mut q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitToMenuButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<QuitToMenuRequested>,
) {
    for (interaction, mut bg) in &mut q {
        match interaction {
            Interaction::Pressed => {
                pending.0 = true;
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
}

/// Consumes the flag while in Playing and routes to MainMenu so the standard
/// world-despawn chain fires.
pub fn handle_quit_pending(
    mut pending: ResMut<QuitToMenuRequested>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if pending.0 {
        pending.0 = false;
        next_state.set(GameState::MainMenu);
    }
}


pub struct PauseScreen;

impl crate::screen::ScreenSetup for PauseScreen {
    fn register(app: &mut bevy::prelude::App) {
        use bevy::prelude::*;
        use models::game_states::GameState;
        app.add_systems(OnEnter(GameState::Paused), setup)
            .add_systems(
                OnExit(GameState::Paused),
                crate::despawn::despawn_all::<PauseMenu>,
            )
            .add_systems(
                Update,
                (handle_resume, handle_settings_button, handle_quit_to_menu_button)
                    .run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                handle_quit_pending.run_if(in_state(GameState::Playing)),
            );
    }
}
