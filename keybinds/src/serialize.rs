use std::collections::HashMap;

use bevy::prelude::KeyCode;

use crate::action::Action;
use crate::bindings::Keybinds;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Converts a [`Keybinds`] resource into a plain string map suitable for
/// serialization. The `save` crate owns JSON encoding; this returns raw data.
pub fn to_map(keybinds: &Keybinds) -> HashMap<String, String> {
    Action::ALL
        .iter()
        .map(|&action| {
            (
                action_name(action).to_owned(),
                keycode_name(keybinds.key(action)).to_owned(),
            )
        })
        .collect()
}

/// Builds a [`Keybinds`] from a plain string map, skipping unknown entries.
pub fn from_map(raw: &HashMap<String, String>) -> Keybinds {
    let mut keybinds = Keybinds::default();
    for (action_s, key_s) in raw {
        if let (Some(action), Some(key)) = (action_from_name(action_s), keycode_from_name(key_s)) {
            keybinds.set(action, key);
        }
    }
    keybinds
}

// ---------------------------------------------------------------------------
// Action name mapping
// ---------------------------------------------------------------------------

fn action_name(action: Action) -> &'static str {
    match action {
        Action::MoveUp => "MoveUp",
        Action::MoveDown => "MoveDown",
        Action::MoveLeft => "MoveLeft",
        Action::MoveRight => "MoveRight",
        Action::Sprint => "Sprint",
        Action::Interact => "Interact",
        Action::Pause => "Pause",
        Action::DialogAdvance => "DialogAdvance",
        Action::ToggleDiagnosticsOverlay => "ToggleDiagnosticsOverlay",
        Action::ToggleDebugPanel => "ToggleDebugPanel",
    }
}

fn action_from_name(s: &str) -> Option<Action> {
    match s {
        "MoveUp" => Some(Action::MoveUp),
        "MoveDown" => Some(Action::MoveDown),
        "MoveLeft" => Some(Action::MoveLeft),
        "MoveRight" => Some(Action::MoveRight),
        "Sprint" => Some(Action::Sprint),
        "Interact" => Some(Action::Interact),
        "Pause" => Some(Action::Pause),
        "DialogAdvance" => Some(Action::DialogAdvance),
        "ToggleDiagnosticsOverlay" => Some(Action::ToggleDiagnosticsOverlay),
        "ToggleDebugPanel" => Some(Action::ToggleDebugPanel),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// KeyCode name mapping  (covers all keys in keycode_label + common extras)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_lines)]
fn keycode_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "KeyA",
        KeyCode::KeyB => "KeyB",
        KeyCode::KeyC => "KeyC",
        KeyCode::KeyD => "KeyD",
        KeyCode::KeyE => "KeyE",
        KeyCode::KeyF => "KeyF",
        KeyCode::KeyG => "KeyG",
        KeyCode::KeyH => "KeyH",
        KeyCode::KeyI => "KeyI",
        KeyCode::KeyJ => "KeyJ",
        KeyCode::KeyK => "KeyK",
        KeyCode::KeyL => "KeyL",
        KeyCode::KeyM => "KeyM",
        KeyCode::KeyN => "KeyN",
        KeyCode::KeyO => "KeyO",
        KeyCode::KeyP => "KeyP",
        KeyCode::KeyQ => "KeyQ",
        KeyCode::KeyR => "KeyR",
        KeyCode::KeyS => "KeyS",
        KeyCode::KeyT => "KeyT",
        KeyCode::KeyU => "KeyU",
        KeyCode::KeyV => "KeyV",
        KeyCode::KeyW => "KeyW",
        KeyCode::KeyX => "KeyX",
        KeyCode::KeyY => "KeyY",
        KeyCode::KeyZ => "KeyZ",
        KeyCode::Digit0 => "Digit0",
        KeyCode::Digit1 => "Digit1",
        KeyCode::Digit2 => "Digit2",
        KeyCode::Digit3 => "Digit3",
        KeyCode::Digit4 => "Digit4",
        KeyCode::Digit5 => "Digit5",
        KeyCode::Digit6 => "Digit6",
        KeyCode::Digit7 => "Digit7",
        KeyCode::Digit8 => "Digit8",
        KeyCode::Digit9 => "Digit9",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Backspace => "Backspace",
        KeyCode::Tab => "Tab",
        KeyCode::ShiftLeft => "ShiftLeft",
        KeyCode::ShiftRight => "ShiftRight",
        KeyCode::ControlLeft => "ControlLeft",
        KeyCode::ControlRight => "ControlRight",
        KeyCode::AltLeft => "AltLeft",
        KeyCode::AltRight => "AltRight",
        KeyCode::ArrowUp => "ArrowUp",
        KeyCode::ArrowDown => "ArrowDown",
        KeyCode::ArrowLeft => "ArrowLeft",
        KeyCode::ArrowRight => "ArrowRight",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        _ => "Unknown",
    }
}

#[allow(clippy::too_many_lines)]
fn keycode_from_name(s: &str) -> Option<KeyCode> {
    match s {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space),
        "Enter" => Some(KeyCode::Enter),
        "Escape" => Some(KeyCode::Escape),
        "Backspace" => Some(KeyCode::Backspace),
        "Tab" => Some(KeyCode::Tab),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}
