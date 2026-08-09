//! Stable string names for [`GamepadButton`] so [`ControllerBinds`] can
//! round-trip through the save file alongside [`crate::Keybinds`].

use std::collections::HashMap;
use std::str::FromStr;

use bevy::prelude::GamepadButton;

use crate::action::Action;
use crate::controller::ControllerBinds;

/// `GamepadButton` <-> string mapping. Mirrors `KEYCODE_TABLE` -- one row per
/// supported button; unknown buttons in old save files are skipped silently.
const BUTTON_TABLE: &[(&str, GamepadButton)] = &[
    ("South", GamepadButton::South),
    ("East", GamepadButton::East),
    ("North", GamepadButton::North),
    ("West", GamepadButton::West),
    ("C", GamepadButton::C),
    ("Z", GamepadButton::Z),
    ("LeftTrigger", GamepadButton::LeftTrigger),
    ("LeftTrigger2", GamepadButton::LeftTrigger2),
    ("RightTrigger", GamepadButton::RightTrigger),
    ("RightTrigger2", GamepadButton::RightTrigger2),
    ("Select", GamepadButton::Select),
    ("Start", GamepadButton::Start),
    ("Mode", GamepadButton::Mode),
    ("LeftThumb", GamepadButton::LeftThumb),
    ("RightThumb", GamepadButton::RightThumb),
    ("DPadUp", GamepadButton::DPadUp),
    ("DPadDown", GamepadButton::DPadDown),
    ("DPadLeft", GamepadButton::DPadLeft),
    ("DPadRight", GamepadButton::DPadRight),
];

/// Canonical name for a [`GamepadButton`], or `None` if it isn't in the
/// supported set.
pub fn button_name(button: GamepadButton) -> Option<&'static str> {
    BUTTON_TABLE
        .iter()
        .find_map(|(name, b)| (*b == button).then_some(*name))
}

/// Parses a [`GamepadButton`] from its canonical name.
pub fn button_from_name(s: &str) -> Option<GamepadButton> {
    BUTTON_TABLE
        .iter()
        .find_map(|(name, b)| (*name == s).then_some(*b))
}

/// Serializes [`ControllerBinds`] to a plain string map. Mirrors
/// [`crate::serialize::to_map`] for [`crate::Keybinds`].
pub fn controller_to_map(binds: &ControllerBinds) -> HashMap<String, String> {
    binds
        .iter()
        .filter_map(|(action, button)| {
            button_name(*button).map(|name| (action.to_string(), name.to_owned()))
        })
        .collect()
}

/// Builds [`ControllerBinds`] from a plain string map, skipping unknowns.
pub fn controller_from_map(raw: &HashMap<String, String>) -> ControllerBinds {
    let mut binds = ControllerBinds::default();
    for (action_s, button_s) in raw {
        if let (Ok(action), Some(button)) = (Action::from_str(action_s), button_from_name(button_s))
        {
            binds.set(action, button);
        }
    }
    binds
}
