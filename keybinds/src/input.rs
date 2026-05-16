//! Unified action-input helpers: read both keyboard and gamepad in one call.
//!
//! Use [`ActionInput`] as a [`SystemParam`] in any system that previously
//! pulled `ButtonInput<KeyCode>` + `Keybinds` to check action state. The
//! helper falls back gracefully when no gamepad is connected (zero-cost
//! iteration over an empty query).

use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Gamepad, KeyCode, Query, Res};

use crate::action::Action;
use crate::bindings::Keybinds;
use crate::controller::ControllerBinds;

/// Magnitude below which left-stick input is treated as zero. Tuned so a
/// resting stick on a worn Xbox controller doesn't drift the player. Per-axis
/// deadzones already live in `GamepadSettings`; this is a magnitude-based
/// second pass that also kills small diagonal jitter.
pub const STICK_DEADZONE: f32 = 0.2;

/// Bundled keyboard + gamepad input view. Pass this into systems and call
/// [`ActionInput::just_pressed`] / [`ActionInput::pressed`] instead of
/// matching against `KeyCode` directly.
#[derive(SystemParam)]
pub struct ActionInput<'w, 's> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub keybinds: Res<'w, Keybinds>,
    pub controller: Res<'w, ControllerBinds>,
    pub gamepads: Query<'w, 's, &'static Gamepad>,
}

impl ActionInput<'_, '_> {
    /// Returns `true` if `action`'s bound key or gamepad button was pressed
    /// this frame.
    pub fn just_pressed(&self, action: Action) -> bool {
        if self.keyboard.just_pressed(self.keybinds.key(action)) {
            return true;
        }
        let Some(button) = self.controller.button(action) else {
            return false;
        };
        self.gamepads.iter().any(|gp| gp.just_pressed(button))
    }

    /// Returns `true` while `action`'s bound key or gamepad button is held.
    pub fn pressed(&self, action: Action) -> bool {
        if self.keyboard.pressed(self.keybinds.key(action)) {
            return true;
        }
        let Some(button) = self.controller.button(action) else {
            return false;
        };
        self.gamepads.iter().any(|gp| gp.pressed(button))
    }

    /// Combined movement vector: keyboard/DPad direction (axis-aligned) plus
    /// analog left stick. Stick wins when its magnitude exceeds
    /// [`STICK_DEADZONE`]; otherwise the digital vector is returned as-is.
    ///
    /// Returned vector is **not** normalised so callers retain analog
    /// magnitude for variable-speed walking. Diagonal digital input yields
    /// length sqrt(2) -- normalise at the call site if needed.
    pub fn movement_vector(&self) -> Vec2 {
        let stick = self
            .gamepads
            .iter()
            .map(Gamepad::left_stick)
            .find(|v| v.length_squared() > STICK_DEADZONE * STICK_DEADZONE)
            .unwrap_or(Vec2::ZERO);
        if stick != Vec2::ZERO {
            return stick;
        }
        [
            (Action::MoveUp, Vec2::Y),
            (Action::MoveDown, Vec2::NEG_Y),
            (Action::MoveLeft, Vec2::NEG_X),
            (Action::MoveRight, Vec2::X),
        ]
        .into_iter()
        .filter(|(action, _)| self.pressed(*action))
        .map(|(_, dir)| dir)
        .sum()
    }
}
