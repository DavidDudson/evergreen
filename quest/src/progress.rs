//! Per-quest player progress, persisted across sessions.

use std::collections::HashMap;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::model::QuestId;

/// Lifecycle stage of a single quest from the player's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    /// Giver has revealed the quest exists but the player has not accepted.
    Offered,
    /// In the active log; milestone watcher is running.
    Active,
    /// All milestones complete and rewards applied.
    Completed,
}

/// Per-quest record: status plus the highest milestone index reached so far.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRecord {
    pub status: QuestStatus,
    /// Highest milestone index whose flag has been observed set. `None` if no
    /// milestone has completed yet (status is `Offered` or `Active` at start).
    pub completed_milestone: Option<usize>,
}

/// Player-facing quest state. Persisted to the save file under [`QUEST_SLOT`].
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    pub records: HashMap<QuestId, QuestRecord>,
}

impl QuestProgress {
    pub fn status(&self, id: &QuestId) -> Option<QuestStatus> {
        self.records.get(id).map(|r| r.status)
    }

    pub fn is_active(&self, id: &QuestId) -> bool {
        matches!(self.status(id), Some(QuestStatus::Active))
    }

    pub fn is_completed(&self, id: &QuestId) -> bool {
        matches!(self.status(id), Some(QuestStatus::Completed))
    }

    /// Insert/update a record. Returns the previous status if any.
    pub fn upsert(
        &mut self,
        id: QuestId,
        status: QuestStatus,
        completed_milestone: Option<usize>,
    ) -> Option<QuestStatus> {
        let prev = self.records.get(&id).map(|r| r.status);
        self.records.insert(
            id,
            QuestRecord {
                status,
                completed_milestone,
            },
        );
        prev
    }

    pub fn iter(&self) -> impl Iterator<Item = (&QuestId, &QuestRecord)> {
        self.records.iter()
    }
}

/// Save-file slot key for [`QuestProgress`].
pub const QUEST_SLOT: &str = "quest.progress";
