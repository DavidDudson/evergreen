//! Lifecycle messages emitted by the quest system.
//!
//! Producers: [`crate::plugin::watch_quest_flags`] reads [`DialogueFlags`] each
//! frame and emits a message when a quest crosses a state boundary. Consumers
//! are typically UI (toast, log) and audio (stinger).

use bevy::prelude::Message;

use crate::model::QuestId;

/// Emitted the first frame the quest's `offer_flag` becomes set. The quest is
/// now visible to the giver but not yet in the active log.
#[derive(Message, Debug, Clone)]
pub struct QuestOffered {
    pub quest: QuestId,
}

/// Emitted the first frame the quest's `accept_flag` becomes set. The quest
/// joins the active log; milestone watching begins.
#[derive(Message, Debug, Clone)]
pub struct QuestAccepted {
    pub quest: QuestId,
}

/// Emitted whenever the active milestone index advances (one or more
/// milestone flags became set since the last frame).
#[derive(Message, Debug, Clone)]
pub struct MilestoneAdvanced {
    pub quest: QuestId,
    /// The milestone index that just completed.
    pub milestone: usize,
}

/// Emitted the first frame all milestones are complete. Rewards have been
/// applied at this point.
#[derive(Message, Debug, Clone)]
pub struct QuestCompleted {
    pub quest: QuestId,
}
