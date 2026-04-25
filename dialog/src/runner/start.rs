use bevy::prelude::*;
use models::game_states::GameState;

use crate::asset::DialogueScript;
use crate::components::Talker;
use crate::events::StartDialogue;

use super::state::{DialogueRunner, DialogueTarget, RunnerState};

/// Handles a [`StartDialogue`] event: loads the script and transitions state.
pub fn start_dialogue(
    mut events: MessageReader<StartDialogue>,
    mut talker_q: Query<&mut Talker>,
    scripts: Res<Assets<DialogueScript>>,
    mut runner: ResMut<DialogueRunner>,
    mut target: ResMut<DialogueTarget>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(event) = events.read().next() else {
        return;
    };
    let Ok(mut talker) = talker_q.get_mut(event.npc) else {
        return;
    };
    if talker.has_greeted && !talker.repeat_greeting {
        return;
    }

    let Some(script) = scripts.get(talker.greeting.id()) else {
        return;
    };

    talker.has_greeted = true;
    target.0 = Some(event.npc);
    runner.state = RunnerState::Running {
        script: script.clone(),
        remaining: script.lines.clone(),
        seen: Vec::new(),
        awaiting_advance: false,
        awaiting_choice: false,
    };

    next_state.set(GameState::Dialogue);
}
