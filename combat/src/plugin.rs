use crate::events::damage::{DamageEvent, DeathEvent};
use bevy::prelude::*;
use models::game_states::GameState;
use models::health::Health;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        let _ = app
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_systems(
                Update,
                (
                    apply_damage,
                    handle_deaths,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// System to apply damage from events
fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    mut health_query: Query<&mut Health>,
) {
    for event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(event.target) {
            if *health > event.attack.damage {
                *health -= event.attack.damage;
            } else {
                health.0 = 0;
                death_events.write(DeathEvent {
                    target: event.target,
                    attack: event.attack,
                    source: event.source,
                });
            }
        }
    }
}

fn handle_deaths(
    mut damage_events: MessageReader<DeathEvent>,
    mut commands: Commands,
) {
    for event in damage_events.read() {
        commands.entity(event.target).despawn();
    }
}
