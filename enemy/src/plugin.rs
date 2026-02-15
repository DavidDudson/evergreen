use bevy::prelude::*;
use bevy_rapier2d::plugin::ReadRapierContext;
use bevy_rapier2d::prelude::Velocity;
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
    mut enemy: Query<(Entity, &Speed, &mut Velocity)>, 
    scenery: Query<Entity, With<Scenery>>,
    rapier_context: ReadRapierContext,
) {
    let Some(scenery_entity) = scenery.iter().next() else {
        return;
    };

    let Ok(context) = rapier_context.single() else {
        return;
    };

    for (entity, speed, mut vel) in &mut enemy {
        if context.contact_pair(entity, scenery_entity).is_some() {
            vel.linvel.x = speed.0;
        }
    }
}
