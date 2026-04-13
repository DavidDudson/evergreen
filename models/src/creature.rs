use bevy::math::Vec2;
use bevy::prelude::{Component, Timer, TimerMode};

/// Marker for ambient creature entities.
#[derive(Component, Default)]
pub struct Creature;

/// Movement behavior category for a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementType {
    /// Moves along the ground, flips sprite horizontally to face direction.
    Ground,
    /// Moves freely with slight vertical bobbing.
    Flying,
}

/// AI state for a creature entity.
#[derive(Debug, Clone)]
pub enum CreatureState {
    /// Stationary, showing idle frame.
    Idle,
    /// Moving in a random direction.
    Wander(Vec2),
    /// Fleeing away from the player.
    Flee,
}

/// Minimum idle duration in seconds.
const MIN_IDLE_SECS: f32 = 2.0;
/// Maximum idle duration in seconds.
const MAX_IDLE_SECS: f32 = 5.0;
/// Minimum wander duration in seconds.
const MIN_WANDER_SECS: f32 = 1.0;
/// Maximum wander duration in seconds.
const MAX_WANDER_SECS: f32 = 3.0;

impl CreatureState {
    /// Create a new Idle state with a random timer duration.
    pub fn new_idle(seed: u32) -> (Self, Timer) {
        let duration = seeded_range(seed, MIN_IDLE_SECS, MAX_IDLE_SECS);
        (Self::Idle, Timer::from_seconds(duration, TimerMode::Once))
    }

    /// Create a new Wander state with a random direction and timer duration.
    pub fn new_wander(seed: u32) -> (Self, Timer) {
        let angle = seeded_frac(seed) * std::f32::consts::TAU;
        let direction = Vec2::new(angle.cos(), angle.sin());
        let duration = seeded_range(seed.wrapping_add(1), MIN_WANDER_SECS, MAX_WANDER_SECS);
        (
            Self::Wander(direction),
            Timer::from_seconds(duration, TimerMode::Once),
        )
    }
}

/// AI component driving creature behavior.
#[derive(Component)]
pub struct CreatureAi {
    /// Current behavioral state.
    pub state: CreatureState,
    /// Timer for state duration (Idle/Wander).
    pub timer: Timer,
    /// Movement speed in pixels per second.
    pub speed: f32,
    /// Whether this creature walks or flies.
    pub movement: MovementType,
    /// Monotonically increasing seed for random decisions.
    pub seed_counter: u32,
}

impl CreatureAi {
    /// Create a new AI in Idle state.
    pub fn new(speed: f32, movement: MovementType, seed: u32) -> Self {
        let (state, timer) = CreatureState::new_idle(seed);
        Self {
            state,
            timer,
            speed,
            movement,
            seed_counter: seed,
        }
    }

    /// Advance the seed counter and return a fresh seed.
    pub fn next_seed(&mut self) -> u32 {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.seed_counter
    }
}

/// Hash a seed to a fraction in [0.0, 1.0).
fn seeded_frac(seed: u32) -> f32 {
    let h = seed.wrapping_mul(374_761_393).wrapping_add(668_265_263);
    let h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    let h = h ^ (h >> 16);
    #[allow(clippy::as_conversions)]
    let frac = (h % 10000) as f32 / 10000.0;
    frac
}

/// Generate a random f32 in [min, max) from a seed.
fn seeded_range(seed: u32, min: f32, max: f32) -> f32 {
    min + seeded_frac(seed) * (max - min)
}
