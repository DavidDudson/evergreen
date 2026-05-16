//! Shared input helpers for the player crate.
//!
//! Both the animation-state system and the movement system need to read the
//! same logical movement vector and sprint flag, so we centralise the
//! lookups here. [`ActionInput`] checks keyboard + gamepad together.

use bevy::prelude::Vec2;
use keybinds::{Action, ActionInput};

/// Returns the player's intended movement direction as a non-normalised
/// `Vec2`. Diagonal input yields a vector of length sqrt(2); callers that
/// need a unit vector should normalise it themselves.
pub fn read_movement_input(input: &ActionInput) -> Vec2 {
    [
        (Action::MoveUp, Vec2::Y),
        (Action::MoveDown, Vec2::NEG_Y),
        (Action::MoveLeft, Vec2::NEG_X),
        (Action::MoveRight, Vec2::X),
    ]
    .into_iter()
    .filter(|(action, _)| input.pressed(*action))
    .map(|(_, dir)| dir)
    .sum()
}

/// True while the bound sprint key or button is held.
pub fn is_sprinting(input: &ActionInput) -> bool {
    input.pressed(Action::Sprint)
}
