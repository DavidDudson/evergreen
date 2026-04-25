use bevy::prelude::{KeyCode, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::action::Action;
use crate::serialize::{from_map, to_map};

/// The active keybind map. Query this resource in any system that needs
/// to check what key is bound to a given action.
///
/// # Example
/// ```rust
/// fn my_system(keyboard: Res<ButtonInput<KeyCode>>, bindings: Res<Keybinds>) {
///     if keyboard.just_pressed(bindings.key(Action::Interact)) { ... }
/// }
/// ```
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(into = "HashMap<String, String>", from = "HashMap<String, String>")]
pub struct Keybinds {
    map: HashMap<Action, KeyCode>,
}

impl From<Keybinds> for HashMap<String, String> {
    fn from(value: Keybinds) -> Self {
        to_map(&value)
    }
}

impl From<HashMap<String, String>> for Keybinds {
    fn from(value: HashMap<String, String>) -> Self {
        from_map(&value)
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Action::MoveUp, KeyCode::KeyW);
        map.insert(Action::MoveDown, KeyCode::KeyS);
        map.insert(Action::MoveLeft, KeyCode::KeyA);
        map.insert(Action::MoveRight, KeyCode::KeyD);
        map.insert(Action::Sprint, KeyCode::ShiftLeft);
        map.insert(Action::Interact, KeyCode::KeyE);
        map.insert(Action::Pause, KeyCode::Escape);
        map.insert(Action::DialogAdvance, KeyCode::Space);
        map.insert(Action::ToggleDiagnosticsOverlay, KeyCode::F3);
        map.insert(Action::ToggleDebugPanel, KeyCode::F5);
        Self { map }
    }
}

impl Keybinds {
    /// Returns the bound key for an action, falling back to the default if unset.
    pub fn key(&self, action: Action) -> KeyCode {
        self.map
            .get(&action)
            .copied()
            .unwrap_or_else(|| Self::default_key(action))
    }

    /// Returns the canonical default key for an action (not affected by user config).
    pub fn default_key(action: Action) -> KeyCode {
        Self::default().map[&action]
    }

    /// Rebinds an action to a new key.
    pub fn set(&mut self, action: Action, key: KeyCode) {
        self.map.insert(action, key);
    }

    /// Resets a single action to its default.
    pub fn reset_action(&mut self, action: Action) {
        self.map.insert(action, Self::default_key(action));
    }

    /// Resets all bindings to defaults.
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// Returns true if any OTHER action is already bound to `key`.
    pub fn conflicts(&self, action: Action, key: KeyCode) -> bool {
        self.map.iter().any(|(a, k)| *a != action && *k == key)
    }
}
