use bevy::prelude::*;
use models::game_states::GameState;

use crate::events::damage::{DamageEvent, DeathEvent};
use crate::systems;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        let _ = app
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_systems(
                Update,
                systems::apply_damage.run_if(in_state(GameState::Playing)),
            );
        app.add_observer(systems::default_death_observer);
    }
}
