use bevy::prelude::*;
use models::alignment::PlayerAlignment;

use crate::asset::DialogueLine;
use crate::events::ChoiceMade;
use crate::flags::DialogueFlags;

use super::state::{DialogueRunner, RunnerState};

/// Handles a [`ChoiceMade`] event: applies flags and inlines the chosen branch.
pub fn handle_choice(
    mut events: MessageReader<ChoiceMade>,
    mut runner: ResMut<DialogueRunner>,
    mut flags: ResMut<DialogueFlags>,
    mut alignment: ResMut<PlayerAlignment>,
) {
    let Some(event) = events.read().next() else {
        return;
    };
    let RunnerState::Running {
        ref mut remaining,
        ref mut seen,
        ref mut awaiting_choice,
        ..
    } = runner.state
    else {
        return;
    };

    let Some(DialogueLine::PlayerChoice { ref options }) = remaining.first() else {
        return;
    };

    let Some(chosen) = options.get(event.index) else {
        return;
    };

    for flag in &chosen.flags_set {
        flags.set(flag.clone());
    }

    if let Some(faction) = chosen.alignment_grant {
        alignment.grant(faction);
    }

    seen.push(chosen.text_key.clone());

    let mut new_remaining = chosen.next.clone();
    new_remaining.extend(remaining.iter().skip(1).cloned());
    *remaining = new_remaining;
    *awaiting_choice = false;
}
