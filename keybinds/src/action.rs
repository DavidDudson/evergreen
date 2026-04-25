use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

/// Every player-controllable action in the game.
///
/// Add new variants here when adding new gameplay actions.
/// Each variant maps to a default key (in `Keybinds::default`) and a
/// human-readable label (via `Display` -- see `#[strum(to_string = ...)]`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    IntoStaticStr,
)]
pub enum Action {
    // Movement
    #[strum(to_string = "Move Up")]
    MoveUp,
    #[strum(to_string = "Move Down")]
    MoveDown,
    #[strum(to_string = "Move Left")]
    MoveLeft,
    #[strum(to_string = "Move Right")]
    MoveRight,
    #[strum(to_string = "Sprint")]
    Sprint,
    // Interaction
    #[strum(to_string = "Interact")]
    Interact,
    // UI
    #[strum(to_string = "Pause")]
    Pause,
    #[strum(to_string = "Advance Dialog")]
    DialogAdvance,
    // Diagnostics
    #[strum(to_string = "Toggle Diagnostics Overlay")]
    ToggleDiagnosticsOverlay,
    #[strum(to_string = "Toggle Debug Panel")]
    ToggleDebugPanel,
}
