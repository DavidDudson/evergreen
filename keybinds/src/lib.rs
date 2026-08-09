pub mod action;
pub mod bindings;
pub mod controller;
pub mod controller_serialize;
pub mod haptics;
pub mod input;
pub mod plugin;
pub mod remap;
pub mod serialize;
pub mod systems;

pub use action::Action;
pub use bindings::Keybinds;
pub use controller::ControllerBinds;
pub use haptics::{HapticPulse, Haptics};
pub use input::ActionInput;
pub use plugin::KeybindsPlugin;
pub use remap::{
    AwaitingControllerRemap, AwaitingRemap, CancelRemap, ControllerRemapCompleted, RemapCompleted,
    RequestControllerRemap, RequestRemap,
};

// Re-export so callers can iterate Action variants without depending on strum directly.
pub use strum::IntoEnumIterator;
