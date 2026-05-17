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

/// One visible choice in a [`ChoicesReady`] event.
#[derive(Debug, Clone)]
pub struct ChoiceOptionView {
    /// Index of the option in the script's `options` vector.
    pub index: usize,
    /// Locale key for the option label.
    pub text_key: String,
    /// `true` when this option pitches a quest the player hasn't been
    /// offered yet. The dialog UI renders these with a yellow `?` icon.
    pub is_quest_offer: bool,
}

/// Emitted by the runner when the player must pick a choice.
#[derive(Message, Debug, Clone)]
pub struct ChoicesReady {
    pub options: Vec<ChoiceOptionView>,
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
