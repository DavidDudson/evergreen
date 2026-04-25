pub mod action;
pub mod bindings;
pub mod plugin;
pub mod remap;
pub mod serialize;
pub mod systems;

pub use action::Action;
pub use bindings::Keybinds;
pub use plugin::KeybindsPlugin;
pub use remap::{AwaitingRemap, CancelRemap, RemapCompleted, RequestRemap};

// Re-export so callers can iterate Action variants without depending on strum directly.
pub use strum::IntoEnumIterator;
