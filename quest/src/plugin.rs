//! Quest plugin: wires assets, resources, messages, and the flag-watcher
//! system that turns dialogue flag transitions into quest lifecycle events.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use dialog::flags::DialogueFlags;
use models::alignment::PlayerAlignment;

use crate::asset::{QuestAsset, QuestAssetLoader};
use crate::events::{MilestoneAdvanced, QuestAccepted, QuestCompleted, QuestOffered};
use crate::model::Unlock;
use crate::progress::{QuestProgress, QuestStatus};
use crate::registry::{
    drain_quest_assets, load_quest_manifest, QuestHandles, QuestRegistry,
};

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<QuestAsset>()
            .init_asset_loader::<QuestAssetLoader>();

        app.init_resource::<QuestRegistry>()
            .init_resource::<QuestHandles>()
            .init_resource::<QuestProgress>();

        app.add_message::<QuestOffered>()
            .add_message::<QuestAccepted>()
            .add_message::<MilestoneAdvanced>()
            .add_message::<QuestCompleted>();

        app.add_systems(Startup, load_quest_manifest);
        app.add_systems(Update, (drain_quest_assets, watch_quest_flags).chain());
    }
}

/// All four lifecycle writers grouped so [`watch_quest_flags`] stays under
/// the clippy `too_many_arguments` threshold.
#[derive(SystemParam)]
pub struct QuestEventWriters<'w> {
    pub offered: MessageWriter<'w, QuestOffered>,
    pub accepted: MessageWriter<'w, QuestAccepted>,
    pub advanced: MessageWriter<'w, MilestoneAdvanced>,
    pub completed: MessageWriter<'w, QuestCompleted>,
}

/// Each frame: scan every loaded quest, compare its flag state to the
/// recorded [`QuestProgress`], emit lifecycle messages on transitions, and
/// apply rewards on completion.
pub fn watch_quest_flags(
    registry: Res<QuestRegistry>,
    flags: Res<DialogueFlags>,
    mut progress: ResMut<QuestProgress>,
    mut alignment: ResMut<PlayerAlignment>,
    mut events: QuestEventWriters,
) {
    for (id, quest) in registry.iter() {
        if progress.is_completed(id) {
            continue;
        }

        let offer = flags.is_set(&quest.offer_flag);
        let accept = flags.is_set(&quest.accept_flag);
        let prev_status = progress.status(id);

        // Status transitions.
        if offer && prev_status.is_none() {
            events.offered.write(QuestOffered { quest: id.clone() });
            progress.upsert(id.clone(), QuestStatus::Offered, None);
        }
        if accept && !matches!(prev_status, Some(QuestStatus::Active | QuestStatus::Completed)) {
            events.accepted.write(QuestAccepted { quest: id.clone() });
            progress.upsert(id.clone(), QuestStatus::Active, None);
        }

        // Milestone advancement: only relevant once accepted.
        if !flags.is_set(&quest.accept_flag) {
            continue;
        }
        let prev_milestone = progress
            .records
            .get(id)
            .and_then(|r| r.completed_milestone);
        let highest_set = quest
            .milestones
            .iter()
            .enumerate()
            .filter(|(_, m)| flags.is_set(&m.flag))
            .map(|(i, _)| i)
            .next_back();
        if let Some(new_top) = highest_set {
            let starting = prev_milestone.map_or(0, |p| p + 1);
            for idx in starting..=new_top {
                events.advanced.write(MilestoneAdvanced {
                    quest: id.clone(),
                    milestone: idx,
                });
            }
            progress.upsert(id.clone(), QuestStatus::Active, Some(new_top));

            if new_top + 1 == quest.milestones.len() {
                for unlock in &quest.unlocks {
                    match unlock {
                        Unlock::Alignment(faction) => alignment.grant(*faction),
                        Unlock::Flag(_) => {
                            // Flag unlocks are advisory: we cannot mutate
                            // DialogueFlags here without a write-borrow that
                            // would conflict. Authors should set these flags
                            // directly from a dialogue option's `flags_set`
                            // when the final milestone branch fires.
                        }
                    }
                }
                events.completed.write(QuestCompleted { quest: id.clone() });
                progress.upsert(id.clone(), QuestStatus::Completed, Some(new_top));
            }
        }
    }
}
