//! Read-only views over [`QuestProgress`] joined with [`QuestRegistry`],
//! shaped for the quest-log UI.

use crate::model::{Milestone, Quest};
use crate::progress::{QuestProgress, QuestStatus};
use crate::registry::QuestRegistry;

/// Snapshot of one quest as the UI wants to display it.
pub struct QuestLogEntry<'a> {
    pub quest: &'a Quest,
    pub status: QuestStatus,
    /// Milestones already completed (in order).
    pub completed: Vec<&'a Milestone>,
    /// The next milestone the player is working toward, if any.
    pub current: Option<&'a Milestone>,
}

/// Build a sorted list of every quest the player has interacted with, paired
/// with their current state. Quests not in [`QuestProgress`] are omitted.
pub fn build_log<'a>(
    registry: &'a QuestRegistry,
    progress: &QuestProgress,
) -> Vec<QuestLogEntry<'a>> {
    let mut entries: Vec<QuestLogEntry<'a>> = progress
        .iter()
        .filter_map(|(id, record)| {
            let quest = registry.get(id)?;
            let next_idx = record.completed_milestone.map_or(0, |i| i + 1);
            let completed: Vec<&Milestone> = quest
                .milestones
                .iter()
                .take(record.completed_milestone.map_or(0, |i| i + 1))
                .collect();
            let current = quest.milestones.get(next_idx);
            Some(QuestLogEntry {
                quest,
                status: record.status,
                completed,
                current,
            })
        })
        .collect();
    entries.sort_by(|a, b| a.quest.id.as_str().cmp(b.quest.id.as_str()));
    entries
}
