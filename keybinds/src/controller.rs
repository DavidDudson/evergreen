//! Per-action [`GamepadButton`] bindings. Parallel to [`crate::Keybinds`] but
//! tracks gamepad buttons instead of keyboard keys.
//!
//! Actions without a default gamepad mapping (e.g. diagnostics overlays)
//! return `None` from [`ControllerBinds::button`]; callers should treat that
//! as "no gamepad binding for this action" rather than a panic.

use std::collections::HashMap;

use bevy::prelude::{GamepadButton, Resource};
use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::controller_serialize::{controller_from_map, controller_to_map};

/// Canonical default gamepad button for each action, matching Xbox layout
/// conventions. Actions with `None` have no default mapping; the player can
/// still bind them via the keybind screen if desired.
const DEFAULT_CONTROLLER_BINDINGS: &[(Action, Option<GamepadButton>)] = &[
    (Action::MoveUp, Some(GamepadButton::DPadUp)),
    (Action::MoveDown, Some(GamepadButton::DPadDown)),
    (Action::MoveLeft, Some(GamepadButton::DPadLeft)),
    (Action::MoveRight, Some(GamepadButton::DPadRight)),
    (Action::Sprint, Some(GamepadButton::LeftTrigger2)),
    (Action::Interact, Some(GamepadButton::South)),
    (Action::Pause, Some(GamepadButton::Start)),
    (Action::DialogAdvance, Some(GamepadButton::South)),
    (Action::OpenQuestLog, Some(GamepadButton::Select)),
    (Action::ToggleDiagnosticsOverlay, None),
    (Action::ToggleDebugPanel, None),
];

/// Active gamepad button map. Persisted alongside [`crate::Keybinds`].
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(into = "HashMap<String, String>", from = "HashMap<String, String>")]
pub struct ControllerBinds {
    map: HashMap<Action, GamepadButton>,
}

impl From<ControllerBinds> for HashMap<String, String> {
    fn from(value: ControllerBinds) -> Self {
        controller_to_map(&value)
    }
}

impl From<HashMap<String, String>> for ControllerBinds {
    fn from(value: HashMap<String, String>) -> Self {
        controller_from_map(&value)
    }
}

impl Default for ControllerBinds {
    fn default() -> Self {
        let map = DEFAULT_CONTROLLER_BINDINGS
            .iter()
            .filter_map(|(a, b)| b.map(|button| (*a, button)))
            .collect();
        Self { map }
    }
}

impl ControllerBinds {
    /// Returns the gamepad button bound for `action`, falling back to its
    /// canonical default. `None` means this action has no gamepad binding.
    pub fn button(&self, action: Action) -> Option<GamepadButton> {
        self.map
            .get(&action)
            .copied()
            .or_else(|| Self::default_button(action))
    }

    /// Returns the canonical default button for an action, ignoring the
    /// user's overrides. `None` if the action has no gamepad default.
    pub fn default_button(action: Action) -> Option<GamepadButton> {
        DEFAULT_CONTROLLER_BINDINGS
            .iter()
            .find_map(|(a, b)| (*a == action).then_some(*b))
            .flatten()
    }

    /// Rebinds `action` to `button`. Setting an action that previously had no
    /// gamepad mapping is fine -- it just creates the entry.
    pub fn set(&mut self, action: Action, button: GamepadButton) {
        self.map.insert(action, button);
    }

    /// Removes any user mapping for `action`. Subsequent [`Self::button`]
    /// calls return the canonical default (which may itself be `None`).
    pub fn clear(&mut self, action: Action) {
        self.map.remove(&action);
    }

    /// Resets all bindings to defaults.
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// Returns true if any OTHER action is already bound to `button`.
    pub fn conflicts(&self, action: Action, button: GamepadButton) -> bool {
        self.map.iter().any(|(a, b)| *a != action && *b == button)
    }

    /// Iterate every action with a non-`None` resolved button mapping.
    pub fn iter(&self) -> impl Iterator<Item = (&Action, &GamepadButton)> {
        self.map.iter()
    }
}
