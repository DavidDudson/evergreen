use bevy::prelude::{Entity, Message};
use derive_more::Display;
use models::attack::Attack;

use crate::events::context::DamageContext;

#[derive(Message, Clone, Copy, Debug, Display)]
#[display("{source}, attacked {target} for {attack}")]
pub struct DamageEvent {
    pub target: Entity,
    pub attack: Attack,
    pub source: Entity,
    pub context: DamageContext,
}

impl DamageEvent {
    /// Construct a default physical damage event with no extra mitigation
    /// metadata.
    pub fn new(target: Entity, attack: Attack, source: Entity) -> Self {
        Self::with_context(target, attack, source, DamageContext::DEFAULT_PHYSICAL)
    }

    /// Construct a damage event with a caller-supplied [`DamageContext`].
    pub fn with_context(
        target: Entity,
        attack: Attack,
        source: Entity,
        context: DamageContext,
    ) -> Self {
        Self {
            target,
            attack,
            source,
            context,
        }
    }
}

#[derive(Message, Clone, Copy, Debug, Display)]
#[display("{target} was killed by {source} with a final by {attack}")]
pub struct DeathEvent {
    pub target: Entity,
    pub attack: Attack,
    pub source: Entity,
    pub context: DamageContext,
}

impl DeathEvent {
    /// Construct a default physical death event with no extra mitigation
    /// metadata.
    pub fn new(target: Entity, attack: Attack, source: Entity) -> Self {
        Self::with_context(target, attack, source, DamageContext::DEFAULT_PHYSICAL)
    }

    /// Construct a death event with a caller-supplied [`DamageContext`].
    pub fn with_context(
        target: Entity,
        attack: Attack,
        source: Entity,
        context: DamageContext,
    ) -> Self {
        Self {
            target,
            attack,
            source,
            context,
        }
    }
}
