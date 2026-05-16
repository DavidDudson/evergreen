//! Shared input helpers for the player crate.
//!
//! Both the animation-state system and the movement system need to read the
//! same logical movement vector and sprint flag, so we centralise the
//! lookups here. [`ActionInput`] checks keyboard + gamepad together.

use bevy::prelude::Vec2;
use keybinds::{Action, ActionInput};

/// Returns the player's intended movement direction. Includes analog left
/// stick (with deadzone) when a controller is connected. Non-normalised --
/// digital input may yield length sqrt(2) on diagonals, analog input
/// preserves stick magnitude.
pub fn read_movement_input(input: &ActionInput) -> Vec2 {
    input.movement_vector()
}

/// True while the bound sprint key or button is held.
pub fn is_sprinting(input: &ActionInput) -> bool {
    input.pressed(Action::Sprint)
}
