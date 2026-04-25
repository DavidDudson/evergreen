use bevy::prelude::*;
use models::health::Health;

use crate::events::damage::{DamageEvent, DeathEvent};
use crate::events::death::DeathOccurred;
use crate::modifier::{DamageModifier, ModifyDamage};

/// Reads incoming `DamageEvent`s, applies any `DamageModifier` on the
/// target, mutates `Health`, and emits `DeathEvent` (legacy `Message`
/// channel) plus `DeathOccurred` (observer-targeted `EntityEvent`)
/// when the target's health hits zero.
pub fn apply_damage(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    mut targets: Query<(&mut Health, Option<&DamageModifier>)>,
) {
    damage_events.read().for_each(|event| {
        let Ok((mut health, modifier)) = targets.get_mut(event.target) else {
            return;
        };

        let base = event.attack.damage;
        let final_damage = modifier.map_or(base, |m| m.modify(base, &event.context));

        if *health > final_damage {
            *health -= final_damage;
            return;
        }

        health.0 = 0;
        let target = event.target;
        let attack = event.attack;
        let source = event.source;
        let context = event.context;

        death_events.write(DeathEvent {
            target,
            attack,
            source,
            context,
        });
        commands.trigger(DeathOccurred {
            entity: target,
            attack,
            source,
            context,
        });
    });
}

/// Default observer for `DeathOccurred`: despawn the dying entity.
/// Other crates can register additional observers (e.g. to trigger
/// game-over UI, drop loot, play SFX) without removing this one.
pub fn default_death_observer(on: On<DeathOccurred>, mut commands: Commands) {
    commands.entity(on.entity).despawn();
}
