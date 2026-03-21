use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::asset::DialogueLine;
use crate::components::BarkPool;
use crate::asset::DialogueScript;
use crate::events::BarkFired;

/// System: fires a random bark from nearby NPCs with a [`BarkPool`].
///
/// Only runs in `GameState::Playing`. The bark is emitted as a [`BarkFired`]
/// message; the UI crate decides how to display it (floating text above the NPC).
pub fn tick_barks(
    time: Res<Time>,
    player_q: Query<&GlobalTransform, Without<BarkPool>>,
    mut bark_q: Query<(Entity, &mut BarkPool, &GlobalTransform)>,
    scripts: Res<Assets<DialogueScript>>,
    mut writer: MessageWriter<BarkFired>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

    let mut rng = rand::thread_rng();

    for (entity, mut pool, tf) in &mut bark_q {
        pool.cooldown.tick(time.delta());
        if !pool.cooldown.just_finished() {
            continue;
        }

        let npc_pos = tf.translation().truncate();
        if player_pos.distance(npc_pos) > pool.trigger_radius_px {
            continue;
        }

        let Some(handle) = pool.barks.choose(&mut rng) else {
            continue;
        };
        let Some(script) = scripts.get(handle.id()) else {
            continue;
        };

        // Use the first Speech line in the bark script as the text.
        let text_key = script.lines.iter().find_map(|line| {
            if let DialogueLine::Speech { text_key } = line {
                Some(text_key.clone())
            } else {
                None
            }
        });

        let Some(text_key) = text_key else { continue };

        writer.write(BarkFired { npc: entity, text_key });
    }
}
