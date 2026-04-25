pub mod events;
pub mod modifier;
pub mod plugin;
mod systems;

pub use events::context::{DamageContext, ImpactRatio};
pub use events::damage::{DamageEvent, DeathEvent};
pub use events::death::DeathOccurred;
pub use modifier::{DamageModifier, ModifyDamage, ResistPct, MAX_RESIST_PCT};
pub use plugin::CombatPlugin;
