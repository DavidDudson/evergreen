use bevy::prelude::Resource;

/// Rate at which game time advances: 1 game hour per 25 real seconds.
const HOURS_PER_REAL_SECOND: f32 = 1.0 / 25.0;

/// Maximum hour value before wrapping back to 0.
const HOURS_PER_DAY: f32 = 24.0;

/// Starting hour when a new game begins (8:00 AM -- morning).
const DEFAULT_STARTING_HOUR: f32 = 8.0;

/// Tracks the in-game time of day.
///
/// `hour` ranges from 0.0 (midnight) to just under 24.0, wrapping.
/// Advances only while `GameState::Playing`.
#[derive(Resource)]
pub struct GameClock {
    /// Current hour (0.0..24.0).
    pub hour: f32,
    /// Game hours per real second.
    pub rate: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            hour: DEFAULT_STARTING_HOUR,
            rate: HOURS_PER_REAL_SECOND,
        }
    }
}

impl GameClock {
    /// Advance the clock by `delta_seconds` real time.
    pub fn tick(&mut self, delta_seconds: f32) {
        self.hour += self.rate * delta_seconds;
        if self.hour >= HOURS_PER_DAY {
            self.hour -= HOURS_PER_DAY;
        }
    }
}
