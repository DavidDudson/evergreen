use bevy::prelude::*;
use bevy::window::WindowFocused;
use models::game_states::GameState;

pub fn handle_window_focus(
    mut focus_events: MessageReader<WindowFocused>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    focus_events
        .read()
        .for_each(|event| match (*current_state.get(), event.focused) {
            (GameState::Playing, false) => next_state.set(GameState::Paused),
            (GameState::Paused, true) => next_state.set(GameState::Playing),
            _ => {}
        });
}
