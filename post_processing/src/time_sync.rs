use bevy::prelude::*;
use models::time::GameClock;

/// Advance the game clock each frame.
pub fn tick_game_clock(mut clock: ResMut<GameClock>, time: Res<Time>) {
    clock.tick(time.delta_secs());
}
