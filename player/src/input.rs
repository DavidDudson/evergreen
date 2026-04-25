//! Shared keyboard input helpers for the player crate.
//!
//! Both the animation-state system and the movement system need to read the
//! same logical movement vector and sprint flag, so we centralise the
//! `Keybinds`-aware lookups here to keep the two systems in lock-step.

use bevy::prelude::*;
use keybinds::{Action, Keybinds};

/// Returns the player's intended movement direction as a non-normalised
/// `Vec2`. Diagonal input yields a vector of length sqrt(2); callers that
/// need a unit vector should normalise it themselves.
pub fn read_movement_input(keyboard: &ButtonInput<KeyCode>, bindings: &Keybinds) -> Vec2 {
    [
        (Action::MoveUp, Vec2::Y),
        (Action::MoveDown, Vec2::NEG_Y),
        (Action::MoveLeft, Vec2::NEG_X),
        (Action::MoveRight, Vec2::X),
    ]
    .into_iter()
    .filter(|(action, _)| keyboard.pressed(bindings.key(*action)))
    .map(|(_, dir)| dir)
    .sum()
}

/// True while the bound sprint key is held.
pub fn is_sprinting(keyboard: &ButtonInput<KeyCode>, bindings: &Keybinds) -> bool {
    keyboard.pressed(bindings.key(Action::Sprint))
}
