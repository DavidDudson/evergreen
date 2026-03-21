use serde::{Deserialize, Serialize};

/// Every player-controllable action in the game.
///
/// Add new variants here when adding new gameplay actions.
/// Each variant maps to a default key and a display label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // Movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Sprint,
    // Interaction
    Interact,
    // UI
    Pause,
    DialogAdvance,
}

impl Action {
    /// All actions in display order for the config screen.
    pub const ALL: &'static [Action] = &[
        Action::MoveUp,
        Action::MoveDown,
        Action::MoveLeft,
        Action::MoveRight,
        Action::Sprint,
        Action::Interact,
        Action::Pause,
        Action::DialogAdvance,
    ];

    /// Human-readable label shown in the keybind config UI.
    pub fn label(self) -> &'static str {
        match self {
            Action::MoveUp => "Move Up",
            Action::MoveDown => "Move Down",
            Action::MoveLeft => "Move Left",
            Action::MoveRight => "Move Right",
            Action::Sprint => "Sprint",
            Action::Interact => "Interact",
            Action::Pause => "Pause",
            Action::DialogAdvance => "Advance Dialog",
        }
    }
}
