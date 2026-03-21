use bevy::prelude::*;
use models::game_states::GameState;

use crate::asset::{DialogueLine, DialogueScript};
use crate::components::{DialogueTrigger, Talker};
use crate::events::{
    ChoiceMade, ChoicesReady, DialogueEnded, DialogueLineReady, StartDialogue,
};
use crate::flags::DialogueFlags;
use crate::history::LoreBook;

const INTERACT_RADIUS_PX: f32 = 48.0;

// ---------------------------------------------------------------------------
// Runner state
// ---------------------------------------------------------------------------

/// Tracks the current position in an active dialogue script.
#[derive(Resource, Default)]
pub struct DialogueRunner {
    pub(crate) state: RunnerState,
}

pub(crate) enum RunnerState {
    Idle,
    Running {
        script: DialogueScript,
        /// Remaining lines to process (front = next line).
        remaining: Vec<DialogueLine>,
        /// Lines already presented (locale keys), for lore recording.
        seen: Vec<String>,
        /// Whether the runner is waiting for the player to press "next".
        awaiting_advance: bool,
    },
}

impl Default for RunnerState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---------------------------------------------------------------------------
// Systems (Playing state)
// ---------------------------------------------------------------------------

/// Detects when the player is in range of a Talker and marks them with
/// [`DialogueTrigger`]. Removes the marker when out of range.
pub fn detect_interact_range(
    talker_q: Query<(Entity, &GlobalTransform), With<Talker>>,
    player_q: Query<(Entity, &GlobalTransform), Without<Talker>>,
    mut commands: Commands,
    trigger_q: Query<(Entity, &DialogueTrigger)>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

    // Find the nearest Talker in range.
    let nearest = talker_q
        .iter()
        .filter_map(|(entity, tf)| {
            let dist = player_pos.distance(tf.translation().truncate());
            (dist <= INTERACT_RADIUS_PX).then_some((entity, dist))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
        .map(|(e, _)| e);

    let current_trigger = trigger_q.get(player_entity).ok().map(|(_, t)| t.npc);

    match (nearest, current_trigger) {
        (Some(npc), None) => {
            commands.entity(player_entity).insert(DialogueTrigger { npc });
        }
        (None, Some(_)) => {
            commands.entity(player_entity).remove::<DialogueTrigger>();
        }
        (Some(npc), Some(current)) if npc != current => {
            commands
                .entity(player_entity)
                .insert(DialogueTrigger { npc });
        }
        _ => {}
    }
}

/// When the player presses E near a Talker, emit [`StartDialogue`].
pub fn detect_interact_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_q: Query<&DialogueTrigger>,
    mut writer: MessageWriter<StartDialogue>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(trigger) = player_q.single() else {
        return;
    };
    writer.write(StartDialogue { npc: trigger.npc });
}

// ---------------------------------------------------------------------------
// Systems (any state)
// ---------------------------------------------------------------------------

/// Handles a [`StartDialogue`] event: loads the script and transitions state.
pub fn start_dialogue(
    mut events: MessageReader<StartDialogue>,
    mut talker_q: Query<&mut Talker>,
    scripts: Res<Assets<DialogueScript>>,
    mut runner: ResMut<DialogueRunner>,
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
    runner.state = RunnerState::Running {
        script: script.clone(),
        remaining: script.lines.clone(),
        seen: Vec::new(),
        awaiting_advance: false,
    };

    next_state.set(GameState::Dialogue);
}

// ---------------------------------------------------------------------------
// Systems (Dialogue state)
// ---------------------------------------------------------------------------

/// Advances the runner by one step, emitting presentation events for the UI.
pub fn advance_runner(
    mut runner: ResMut<DialogueRunner>,
    keyboard: Res<ButtonInput<KeyCode>>,
    flags: Res<DialogueFlags>,
    mut line_writer: MessageWriter<DialogueLineReady>,
    mut choice_writer: MessageWriter<ChoicesReady>,
    mut end_writer: MessageWriter<DialogueEnded>,
) {
    // Check if the script has finished (remaining is empty).
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
    } = runner.state
    else {
        return;
    };

    // If we're waiting for a keypress to advance past a speech line.
    if *awaiting_advance {
        if !keyboard.just_pressed(KeyCode::Space)
            && !keyboard.just_pressed(KeyCode::Enter)
            && !keyboard.just_pressed(KeyCode::KeyE)
        {
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
                .filter(|(_, opt)| flags.all_set(&opt.flags_required))
                .map(|(i, opt)| (i, opt.text_key.clone()))
                .collect();

            if visible.is_empty() {
                // No options available (all flag-gated); skip this choice node.
                remaining.remove(0);
            } else {
                choice_writer.write(ChoicesReady { options: visible });
                // Don't remove yet — handle_choice will splice in the branch.
            }
        }
    }
}

/// Handles a [`ChoiceMade`] event: applies flags and inlines the chosen branch.
pub fn handle_choice(
    mut events: MessageReader<ChoiceMade>,
    mut runner: ResMut<DialogueRunner>,
    mut flags: ResMut<DialogueFlags>,
) {
    let Some(event) = events.read().next() else {
        return;
    };
    let RunnerState::Running {
        ref mut remaining,
        ref mut seen,
        ..
    } = runner.state
    else {
        return;
    };

    // The first element of remaining is the PlayerChoice line.
    let Some(DialogueLine::PlayerChoice { ref options }) = remaining.first() else {
        return;
    };

    let Some(chosen) = options.get(event.index) else {
        return;
    };

    // Apply flags.
    for flag in &chosen.flags_set {
        flags.set(flag.clone());
    }

    // Record the player's choice key for lore.
    seen.push(chosen.text_key.clone());

    // Replace remaining with the chosen branch's lines followed by the rest.
    let mut new_remaining = chosen.next.clone();
    new_remaining.extend(remaining.iter().skip(1).cloned());
    *remaining = new_remaining;
}

/// Records lore and transitions back to Playing when dialogue ends.
pub fn on_dialogue_ended(
    mut events: MessageReader<DialogueEnded>,
    mut runner: ResMut<DialogueRunner>,
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
        lore_book.record(
            script.id.clone(),
            script.speaker_key.clone(),
            script.keyword_tags.clone(),
            seen.clone(),
            time.elapsed_secs(),
        );
    }

    runner.state = RunnerState::Idle;
    next_state.set(GameState::Playing);
}
