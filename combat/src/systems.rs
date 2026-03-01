use bevy::prelude::*;
use models::health::Health;

use crate::events::damage::{DamageEvent, DeathEvent};

pub fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    mut health_query: Query<&mut Health>,
) {
    damage_events.read().for_each(|event| {
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
    });
}

pub fn handle_deaths(mut death_events: MessageReader<DeathEvent>, mut commands: Commands) {
    death_events
        .read()
        .for_each(|event| commands.entity(event.target).despawn());
}
