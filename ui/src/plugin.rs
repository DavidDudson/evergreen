use bevy::prelude::*;
use bevy::window::WindowFocused;
use models::game_states::GameState;
use models::health::Health;

pub struct UiPlugin;

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct HUD;

#[derive(Component)]
struct GameOverMenu;

#[derive(Component)]
struct PauseMenu;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            .add_systems(Update, button_system)
            .add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnExit(GameState::Playing), cleanup_hud)
            .add_systems(
                Update,
                update_health_text.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_menu)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over_menu)
            .add_systems(Update, handle_window_focus)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu);
    }
}

fn setup_main_menu(mut commands: Commands) {
    let green_dark = Color::srgb(0.06, 0.14, 0.08);
    let green_title = Color::srgb(0.58, 0.82, 0.52);
    let green_accent = Color::srgb(0.24, 0.48, 0.29);
    let green_btn_bg = Color::srgb(0.12, 0.30, 0.16);
    let green_btn_text = Color::srgb(0.72, 0.92, 0.68);

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
            BackgroundColor(green_dark),
        ))
        .with_children(|parent| {
            // Main title: "Evergreen"
            parent.spawn((
                Text::new("Evergreen"),
                TextColor(green_title),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Node::default()
                },
            ));

            // Start button
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(14.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Node::default()
                    },
                    BorderColor::all(green_accent),
                    BackgroundColor(green_btn_bg),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_child((
                    Text::new("Begin Journey"),
                    TextColor(green_btn_text),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                ));
        });
}

fn setup_game_over_menu(mut commands: Commands) {
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

fn setup_hud(mut commands: Commands) {
    commands.spawn((
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

fn update_health_text(
    castle_query: Query<&Health>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    // Get the castle's health
    if let Ok(health) = castle_query.single() {
        // Get the health text
        if let Ok(mut text) = text_query.single_mut() {
            // Update the text with the current health value
            text.0 = format!("Health: {}", health.0);
        }
    }
}

fn button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

fn cleanup_hud(mut commands: Commands, query: Query<Entity, With<HUD>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game_over_menu(mut commands: Commands, query: Query<Entity, With<GameOverMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_window_focus(
    mut focus_events: MessageReader<WindowFocused>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    for event in focus_events.read() {
        if !event.focused && *current_state.get() == GameState::Playing {
            next_state.set(GameState::Paused);
        } else if event.focused && *current_state.get() == GameState::Paused {
            next_state.set(GameState::Playing);
        }
    }
}

fn setup_pause_menu(mut commands: Commands) {
    let green_overlay = Color::srgba(0.04, 0.12, 0.06, 0.85);
    let green_title = Color::srgb(0.58, 0.82, 0.52);

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
            BackgroundColor(green_overlay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextColor(green_title),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
            ));
        });
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
