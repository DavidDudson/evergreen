use bevy::prelude::{Entity, Message};

/// Send this to begin a scripted dialogue with an NPC entity.
#[derive(Message, Debug, Clone)]
pub struct StartDialogue {
    pub npc: Entity,
}

/// Emitted by the runner when a speech line is ready to display.
#[derive(Message, Debug, Clone)]
pub struct DialogueLineReady {
    /// Locale key for the speaker name, if any.
    pub speaker_key: Option<String>,
    /// Locale key for the line text.
    pub text_key: String,
}

/// Emitted by the runner when the player must pick a choice.
#[derive(Message, Debug, Clone)]
pub struct ChoicesReady {
    /// `(option_index, text_key)` pairs for the visible options.
    pub options: Vec<(usize, String)>,
}

/// Send this to tell the runner which choice the player selected.
#[derive(Message, Debug, Clone)]
pub struct ChoiceMade {
    pub index: usize,
}

/// Emitted when the current dialogue script has finished.
#[derive(Message, Debug, Clone)]
pub struct DialogueEnded;

/// Emitted by the bark system when a bark fires.
#[derive(Message, Debug, Clone)]
pub struct BarkFired {
    pub npc: Entity,
    /// Locale key for the bark text.
    pub text_key: String,
}
