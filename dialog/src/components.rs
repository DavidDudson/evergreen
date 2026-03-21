use bevy::prelude::*;

use crate::asset::DialogueScript;

/// Place on an NPC entity to give it scripted dialogue.
#[derive(Component, Debug)]
pub struct Talker {
    /// The scripted greeting to run when the player interacts.
    pub greeting: Handle<DialogueScript>,
    /// Whether the greeting fires every interaction or only once.
    pub repeat_greeting: bool,
    /// Runtime flag: has this NPC already greeted the player?
    pub has_greeted: bool,
}

impl Talker {
    pub fn new(greeting: Handle<DialogueScript>) -> Self {
        Self {
            greeting,
            repeat_greeting: false,
            has_greeted: false,
        }
    }

    pub fn repeating(greeting: Handle<DialogueScript>) -> Self {
        Self {
            greeting,
            repeat_greeting: true,
            has_greeted: false,
        }
    }
}

/// Place on an NPC entity to give it random ambient bark lines.
#[derive(Component, Debug)]
pub struct BarkPool {
    /// Set of possible bark scripts. One is chosen at random on trigger.
    pub barks: Vec<Handle<DialogueScript>>,
    /// Player must be within this distance (pixels) to trigger a bark.
    pub trigger_radius_px: f32,
    /// Minimum time between consecutive barks from this NPC.
    pub cooldown: Timer,
}

/// Inserted on the player entity when they are in interact range of a Talker.
/// Removed when the player moves out of range or starts a dialogue.
#[derive(Component, Debug)]
pub struct DialogueTrigger {
    pub npc: Entity,
}
