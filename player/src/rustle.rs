use bevy::prelude::*;
use models::scenery::{Rustleable, Rustling};

use crate::spawning::Player;

/// Proximity radius within which the player triggers a rustle.
const RUSTLE_RADIUS_PX: f32 = 14.0;

/// Inserts `Rustling` on any `Rustleable` entity overlapping the player.
pub fn trigger_rustle(
    player_q: Query<&Transform, With<Player>>,
    rustleable_q: Query<(Entity, &Transform), (With<Rustleable>, Without<Rustling>)>,
    mut commands: Commands,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let pp = player_tf.translation.truncate();
    for (entity, tf) in &rustleable_q {
        if pp.distance(tf.translation.truncate()) < RUSTLE_RADIUS_PX {
            commands.entity(entity).insert(Rustling::new());
        }
    }
}
