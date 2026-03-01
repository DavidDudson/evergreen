use crate::damage_type::DamageType;
use crate::distance::Distance;
use crate::health::Health;
use crate::seconds::Seconds;
use bevy::prelude::Component;
use derive_more::Display;

const MELEE_RANGE: Distance = Distance(200);
const MELEE_COOLDOWN: Seconds = Seconds(1);
const DEFAULT_DAMAGE: Health = Health(1);

#[derive(Component, Display, Clone, Copy, Debug)]
#[display("{damage}({damage_type})")]
pub struct Attack {
    pub damage: Health,
    pub damage_type: DamageType,
    pub range: Distance,
    pub cooldown: Seconds,
    pub last: Option<f32>,
}

impl Attack {
    pub fn melee(damage: Health) -> Self {
        Self {
            damage,
            damage_type: DamageType::Physical,
            range: MELEE_RANGE,
            cooldown: MELEE_COOLDOWN,
            last: None,
        }
    }

    pub fn fall(damage: Health) -> Self {
        Self {
            damage,
            damage_type: DamageType::Fall,
            range: Distance(0),
            cooldown: Seconds(0),
            last: None,
        }
    }
}

impl Default for Attack {
    fn default() -> Self {
        Self::melee(DEFAULT_DAMAGE)
    }
}
