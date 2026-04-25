use bevy::prelude::*;

use crate::asset::{DialogueLine, DialogueScript};

/// The NPC entity the player is currently talking to.
/// Set on dialogue start, cleared on dialogue end.
#[derive(Resource, Default)]
pub struct DialogueTarget(pub Option<Entity>);

/// Tracks the current position in an active dialogue script.
#[derive(Resource, Default)]
pub struct DialogueRunner {
    pub(crate) state: RunnerState,
}

#[derive(Default)]
pub(crate) enum RunnerState {
    #[default]
    Idle,
    Running {
        script: DialogueScript,
        /// Remaining lines to process (front = next line).
        remaining: Vec<DialogueLine>,
        /// Lines already presented (locale keys), for lore recording.
        seen: Vec<String>,
        /// Whether the runner is waiting for the player to press "next".
        awaiting_advance: bool,
        /// Whether we've emitted choices and are waiting for a [`ChoiceMade`].
        awaiting_choice: bool,
    },
}
