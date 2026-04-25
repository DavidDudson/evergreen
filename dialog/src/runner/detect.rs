use bevy::prelude::*;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;
use models::speed::Speed;

use crate::components::{DialogueTrigger, Talker};
use crate::events::StartDialogue;

const INTERACT_RADIUS_PX: f32 = 48.0;

/// Detects when the player is in range of a Talker and marks them with
/// [`DialogueTrigger`]. Removes the marker when out of range.
#[allow(clippy::type_complexity)]
pub fn detect_interact_range(
    talker_q: Query<(Entity, &GlobalTransform), With<Talker>>,
    player_q: Query<(Entity, &GlobalTransform), (With<Speed>, Without<Talker>)>,
    mut commands: Commands,
    trigger_q: Query<(Entity, &DialogueTrigger)>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

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
            commands
                .entity(player_entity)
                .insert(DialogueTrigger { npc });
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

/// When the player presses Interact near a Talker, emit [`StartDialogue`].
pub fn detect_interact_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    player_q: Query<&DialogueTrigger>,
    mut writer: MessageWriter<StartDialogue>,
) {
    if !keyboard.just_pressed(bindings.key(Action::Interact)) {
        return;
    }
    let Ok(trigger) = player_q.single() else {
        return;
    };
    writer.write(StartDialogue { npc: trigger.npc });
}
