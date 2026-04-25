use bevy::prelude::{Entity, EntityEvent};
use models::attack::Attack;

use crate::events::context::DamageContext;

/// Entity-targeted observer event fired the moment an entity's health
/// drops to zero. Use `commands.add_observer(...)` (global) or
/// `commands.entity(target).observe(...)` (per-entity) to react.
///
/// Existing systems that consume the legacy `DeathEvent` `Message` keep
/// working unchanged; this event is additive.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct DeathOccurred {
    /// The entity that just died (the observer target).
    pub entity: Entity,
    /// The killing blow.
    pub attack: Attack,
    /// The entity responsible for the killing blow.
    pub source: Entity,
    /// Metadata about the damage that caused the death.
    pub context: DamageContext,
}
