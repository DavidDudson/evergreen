use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::asset::DialogueLine;
use crate::asset::DialogueScript;
use crate::components::BarkPool;
use crate::events::BarkFired;

/// Strategy for picking which bark to fire from a pool. Default is uniform
/// random; replace via `app.insert_resource(BarkSelector::Custom(...))` for
/// weighted, mood-based, or scripted selection.
#[derive(Resource, Clone, Default)]
pub enum BarkSelector {
    #[default]
    Uniform,
    Custom(fn(&[Handle<DialogueScript>]) -> Option<Handle<DialogueScript>>),
}

impl BarkSelector {
    pub fn select(
        &self,
        barks: &[Handle<DialogueScript>],
    ) -> Option<Handle<DialogueScript>> {
        match self {
            Self::Uniform => {
                let mut rng = rand::rng();
                barks.choose(&mut rng).cloned()
            }
            Self::Custom(f) => f(barks),
        }
    }
}

/// System: fires a bark from nearby NPCs with a [`BarkPool`].
///
/// Only runs in `GameState::Playing`. The bark is emitted as a [`BarkFired`]
/// message; the UI crate decides how to display it (floating text above the NPC).
pub fn tick_barks(
    time: Res<Time>,
    selector: Res<BarkSelector>,
    player_q: Query<&GlobalTransform, Without<BarkPool>>,
    mut bark_q: Query<(Entity, &mut BarkPool, &GlobalTransform)>,
    scripts: Res<Assets<DialogueScript>>,
    mut writer: MessageWriter<BarkFired>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

    for (entity, mut pool, tf) in &mut bark_q {
        pool.cooldown.tick(time.delta());
        if !pool.cooldown.just_finished() {
            continue;
        }

        let npc_pos = tf.translation().truncate();
        if player_pos.distance(npc_pos) > pool.trigger_radius_px {
            continue;
        }

        let Some(handle) = selector.select(&pool.barks) else {
            continue;
        };
        let Some(script) = scripts.get(handle.id()) else {
            continue;
        };

        let text_key = script.lines.iter().find_map(|line| {
            if let DialogueLine::Speech { text_key } = line {
                Some(text_key.clone())
            } else {
                None
            }
        });

        let Some(text_key) = text_key else { continue };

        writer.write(BarkFired {
            npc: entity,
            text_key,
        });
    }
}
