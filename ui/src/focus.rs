use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;
use models::game_states::GameState;

pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    match *current_state.get() {
        GameState::Playing => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Playing),
        GameState::QuestLog => next_state.set(GameState::Playing),
        _ => {}
    }
}

/// Toggle the quest log on/off. Bound to [`Action::OpenQuestLog`] (default
/// `J`). Active in `Playing` and `QuestLog` states only.
pub fn handle_quest_log_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if !keyboard.just_pressed(bindings.key(Action::OpenQuestLog)) {
        return;
    }
    match *current_state.get() {
        GameState::Playing => next_state.set(GameState::QuestLog),
        GameState::QuestLog => next_state.set(GameState::Playing),
        _ => {}
    }
}
