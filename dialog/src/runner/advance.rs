use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;

use crate::asset::DialogueLine;
use crate::events::{ChoicesReady, DialogueEnded, DialogueLineReady};
use crate::flags::DialogueFlags;

use super::state::{DialogueRunner, RunnerState};

/// Advances the runner by one step, emitting presentation events for the UI.
pub fn advance_runner(
    mut runner: ResMut<DialogueRunner>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    flags: Res<DialogueFlags>,
    mut line_writer: MessageWriter<DialogueLineReady>,
    mut choice_writer: MessageWriter<ChoicesReady>,
    mut end_writer: MessageWriter<DialogueEnded>,
) {
    let is_done = matches!(
        &runner.state,
        RunnerState::Running { remaining, .. } if remaining.is_empty()
    );
    if is_done {
        end_writer.write(DialogueEnded);
        return;
    }

    let RunnerState::Running {
        ref script,
        ref mut remaining,
        ref mut seen,
        ref mut awaiting_advance,
        ref mut awaiting_choice,
    } = runner.state
    else {
        return;
    };

    if *awaiting_choice {
        return;
    }

    if *awaiting_advance {
        let advance = keyboard.just_pressed(bindings.key(Action::DialogAdvance))
            || keyboard.just_pressed(bindings.key(Action::Interact));
        if !advance {
            return;
        }
        *awaiting_advance = false;
    }

    let Some(line) = remaining.first().cloned() else {
        return;
    };

    match line {
        DialogueLine::Speech { ref text_key } => {
            seen.push(text_key.clone());
            line_writer.write(DialogueLineReady {
                speaker_key: Some(script.speaker_key.clone()),
                text_key: text_key.clone(),
            });
            remaining.remove(0);
            *awaiting_advance = true;
        }
        DialogueLine::PlayerChoice { ref options } => {
            let visible: Vec<(usize, String)> = options
                .iter()
                .enumerate()
                .filter(|(_, opt)| {
                    flags.all_set(&opt.flags_required) && opt.condition.is_satisfied(&flags)
                })
                .map(|(i, opt)| (i, opt.text_key.clone()))
                .collect();

            if visible.is_empty() {
                remaining.remove(0);
            } else {
                choice_writer.write(ChoicesReady { options: visible });
                *awaiting_choice = true;
            }
        }
    }
}
