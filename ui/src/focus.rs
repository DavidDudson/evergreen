use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::ActionInput;
use models::game_states::GameState;

pub fn handle_pause_input(
    input: ActionInput,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if !input.just_pressed(Action::Pause) {
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
/// `J`, gamepad `Select`/`Back`). Active in `Playing` and `QuestLog` only.
pub fn handle_quest_log_input(
    input: ActionInput,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if !input.just_pressed(Action::OpenQuestLog) {
        return;
    }
    match *current_state.get() {
        GameState::Playing => next_state.set(GameState::QuestLog),
        GameState::QuestLog => next_state.set(GameState::Playing),
        _ => {}
    }
}
