use bevy::prelude::*;
use models::game_states::GameState;

use crate::events::DialogueEnded;
use crate::history::LoreBook;

use super::state::{DialogueRunner, DialogueTarget, RunnerState};

/// Records lore and transitions back to Playing when dialogue ends.
pub fn on_dialogue_ended(
    mut events: MessageReader<DialogueEnded>,
    mut runner: ResMut<DialogueRunner>,
    mut target: ResMut<DialogueTarget>,
    mut lore_book: ResMut<LoreBook>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    if events.read().next().is_none() {
        return;
    }

    if let RunnerState::Running {
        ref script,
        ref seen,
        ..
    } = runner.state
    {
        if let Some(ref lore) = script.lore {
            lore_book.record(crate::history::LoreEntry {
                script_id: script.id.clone(),
                speaker_key: script.speaker_key.clone(),
                keyword_tags: script.keyword_tags.clone(),
                category: lore.category,
                topic: lore.topic.clone(),
                image: lore.image.clone(),
                lines_seen: seen.clone(),
                game_time: time.elapsed_secs(),
            });
        }
    }

    runner.state = RunnerState::Idle;
    target.0 = None;
    next_state.set(GameState::Playing);
}
