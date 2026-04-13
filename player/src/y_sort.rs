use bevy::prelude::*;
use models::layer::Layer;

use crate::spawning::Player;

/// Y-sort scale -- must match the value used in level scenery/decorations.
const Y_SORT_SCALE: f32 = 0.001;

/// Update the player's z-position each frame for correct y-sort ordering.
pub fn update_player_z(mut query: Query<&mut Transform, With<Player>>) {
    let Ok(mut tf) = query.single_mut() else {
        return;
    };
    tf.translation.z = Layer::World.z_f32() - tf.translation.y * Y_SORT_SCALE;
}
