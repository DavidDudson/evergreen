//! Unified action-input helpers: read both keyboard and gamepad in one call.
//!
//! Use [`ActionInput`] as a [`SystemParam`] in any system that previously
//! pulled `ButtonInput<KeyCode>` + `Keybinds` to check action state. The
//! helper falls back gracefully when no gamepad is connected (zero-cost
//! iteration over an empty query).

use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::{Gamepad, KeyCode, Query, Res};

use crate::action::Action;
use crate::bindings::Keybinds;
use crate::controller::ControllerBinds;

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
}
