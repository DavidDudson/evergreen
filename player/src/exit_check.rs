use bevy::prelude::*;
use level::exit::LevelExit;
use models::game_states::GameState;

use crate::spawning::Player;

/// Half-width of the exit trigger zone (pixels).
const EXIT_TRIGGER_HALF_PX: f32 = 16.0;

/// Detect player overlapping the exit and trigger level complete.
pub fn check_exit_overlap(
    player_q: Query<&Transform, With<Player>>,
    exit_q: Query<&Transform, (With<LevelExit>, Without<Player>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(exit_tf) = exit_q.single() else {
        return;
    };

    let pp = player_tf.translation.truncate();
    let ep = exit_tf.translation.truncate();

    if (pp.x - ep.x).abs() < EXIT_TRIGGER_HALF_PX
        && (pp.y - ep.y).abs() < EXIT_TRIGGER_HALF_PX
    {
        next_state.set(GameState::LevelComplete);
    }
}
