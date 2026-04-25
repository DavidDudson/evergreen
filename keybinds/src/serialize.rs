use std::collections::HashMap;
use std::str::FromStr;

use bevy::prelude::KeyCode;
use strum::IntoEnumIterator;

use crate::action::Action;
use crate::bindings::Keybinds;

// ---------------------------------------------------------------------------
// KeyCode lookup table
// ---------------------------------------------------------------------------

/// Single source of truth for `KeyCode` <-> string round-trip. Linear scan is
/// cheap (a few dozen entries) and keeps add/remove to a single line each.
const KEYCODE_TABLE: &[(&str, KeyCode)] = &[
    ("KeyA", KeyCode::KeyA),
    ("KeyB", KeyCode::KeyB),
    ("KeyC", KeyCode::KeyC),
    ("KeyD", KeyCode::KeyD),
    ("KeyE", KeyCode::KeyE),
    ("KeyF", KeyCode::KeyF),
    ("KeyG", KeyCode::KeyG),
    ("KeyH", KeyCode::KeyH),
    ("KeyI", KeyCode::KeyI),
    ("KeyJ", KeyCode::KeyJ),
    ("KeyK", KeyCode::KeyK),
    ("KeyL", KeyCode::KeyL),
    ("KeyM", KeyCode::KeyM),
    ("KeyN", KeyCode::KeyN),
    ("KeyO", KeyCode::KeyO),
    ("KeyP", KeyCode::KeyP),
    ("KeyQ", KeyCode::KeyQ),
    ("KeyR", KeyCode::KeyR),
    ("KeyS", KeyCode::KeyS),
    ("KeyT", KeyCode::KeyT),
    ("KeyU", KeyCode::KeyU),
    ("KeyV", KeyCode::KeyV),
    ("KeyW", KeyCode::KeyW),
    ("KeyX", KeyCode::KeyX),
    ("KeyY", KeyCode::KeyY),
    ("KeyZ", KeyCode::KeyZ),
    ("Digit0", KeyCode::Digit0),
    ("Digit1", KeyCode::Digit1),
    ("Digit2", KeyCode::Digit2),
    ("Digit3", KeyCode::Digit3),
    ("Digit4", KeyCode::Digit4),
    ("Digit5", KeyCode::Digit5),
    ("Digit6", KeyCode::Digit6),
    ("Digit7", KeyCode::Digit7),
    ("Digit8", KeyCode::Digit8),
    ("Digit9", KeyCode::Digit9),
    ("Space", KeyCode::Space),
    ("Enter", KeyCode::Enter),
    ("Escape", KeyCode::Escape),
    ("Backspace", KeyCode::Backspace),
    ("Tab", KeyCode::Tab),
    ("ShiftLeft", KeyCode::ShiftLeft),
    ("ShiftRight", KeyCode::ShiftRight),
    ("ControlLeft", KeyCode::ControlLeft),
    ("ControlRight", KeyCode::ControlRight),
    ("AltLeft", KeyCode::AltLeft),
    ("AltRight", KeyCode::AltRight),
    ("ArrowUp", KeyCode::ArrowUp),
    ("ArrowDown", KeyCode::ArrowDown),
    ("ArrowLeft", KeyCode::ArrowLeft),
    ("ArrowRight", KeyCode::ArrowRight),
    ("F1", KeyCode::F1),
    ("F2", KeyCode::F2),
    ("F3", KeyCode::F3),
    ("F4", KeyCode::F4),
    ("F5", KeyCode::F5),
    ("F6", KeyCode::F6),
    ("F7", KeyCode::F7),
    ("F8", KeyCode::F8),
    ("F9", KeyCode::F9),
    ("F10", KeyCode::F10),
    ("F11", KeyCode::F11),
    ("F12", KeyCode::F12),
];

/// Returns the canonical name for a [`KeyCode`], or `None` if it is not in
/// the supported set. Callers that pass a key the user has already bound can
/// `.expect("known KeyCode")` because `capture_remap_key` only accepts keys
/// from this table indirectly via Bevy's input set.
pub fn keycode_name(kc: KeyCode) -> Option<&'static str> {
    KEYCODE_TABLE
        .iter()
        .find_map(|(name, key)| (*key == kc).then_some(*name))
}

/// Parses a [`KeyCode`] from its canonical name. Returns `None` for unknown
/// strings so unfamiliar entries in old save files are skipped silently.
pub fn keycode_from_name(s: &str) -> Option<KeyCode> {
    KEYCODE_TABLE
        .iter()
        .find_map(|(name, key)| (*name == s).then_some(*key))
}

// ---------------------------------------------------------------------------
// Public API: Keybinds <-> string map
// ---------------------------------------------------------------------------

/// Converts a [`Keybinds`] resource into a plain string map suitable for
/// serialization. The `save` crate owns JSON encoding; this returns raw data.
///
/// Entries with a `KeyCode` outside the [`KEYCODE_TABLE`] are silently
/// skipped -- this can only happen if user code rebinds an action to an
/// exotic key the UI does not expose.
pub fn to_map(keybinds: &Keybinds) -> HashMap<String, String> {
    Action::iter()
        .filter_map(|action| {
            keycode_name(keybinds.key(action)).map(|name| (action.to_string(), name.to_owned()))
        })
        .collect()
}

/// Builds a [`Keybinds`] from a plain string map, skipping unknown entries.
pub fn from_map(raw: &HashMap<String, String>) -> Keybinds {
    let mut keybinds = Keybinds::default();
    for (action_s, key_s) in raw {
        if let (Ok(action), Some(key)) = (Action::from_str(action_s), keycode_from_name(key_s)) {
            keybinds.set(action, key);
        }
    }
    keybinds
}
