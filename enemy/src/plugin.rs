use avian2d::prelude::{Collisions, LinearVelocity};
use bevy::prelude::*;
use models::draggable::Dragged;
use models::game_states::GameState;
use models::scenery::Scenery;
use models::speed::Speed;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        let _ = app.add_systems(Update, move_enemy.run_if(in_state(GameState::Playing)));
    }
}

fn move_enemy(
    mut enemy: Query<(Entity, &Speed, &mut LinearVelocity), Without<Dragged>>,
    scenery: Query<Entity, With<Scenery>>,
    collisions: Collisions,
) {
    let Some(scenery_entity) = scenery.iter().next() else {
        return;
    };

    for (entity, speed, mut vel) in &mut enemy {
        if collisions.contains(entity, scenery_entity) {
            vel.x = speed.0;
        }
    }
}
