use bevy::math::Vec2;
use bevy::prelude::{Component, Timer, TimerMode};

#[derive(Component, Default)]
pub struct Scenery;

/// AABB collider for manual scenery collision (no physics engine).
/// The collision box is centred at `entity_position + center_offset`.
#[derive(Component, Clone, Copy)]
pub struct SceneryCollider {
    pub half_extents: Vec2,
    /// Offset from the entity's Transform to the collider centre.
    pub center_offset: Vec2,
}

/// Marks scenery that rustles (animates) when the player walks through it.
#[derive(Component)]
pub struct Rustleable;

const RUSTLE_DURATION_SECS: f32 = 0.5;

/// Active rustle animation state. Inserted by the player crate on overlap.
#[derive(Component)]
pub struct Rustling {
    pub timer: Timer,
}

impl Rustling {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(RUSTLE_DURATION_SECS, TimerMode::Once),
        }
    }
}
