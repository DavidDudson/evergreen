use bevy::prelude::Component;
use derive_more::{AsRef, Display};
use models::health::Health;

use crate::events::context::DamageContext;

/// Maximum value (inclusive) that a `ResistPct` can hold. 100 means
/// "all incoming damage is absorbed" once a percent-based modifier hits
/// the cap.
pub const MAX_RESIST_PCT: u16 = 100;

/// Percent-based resistance, clamped to `0..=MAX_RESIST_PCT` at construction.
///
/// Use [`ResistPct::new`] / [`ResistPct::try_from`] to build a value --
/// both will saturate / reject anything above the cap, so internal
/// arithmetic can rely on the invariant `0 <= self.0 <= MAX_RESIST_PCT`.
#[derive(AsRef, Display, Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct ResistPct(u16);

/// Error returned when a value outside `0..=MAX_RESIST_PCT` is passed to
/// `ResistPct::try_from`.
#[derive(Debug, Clone, Copy, Display)]
#[display("ResistPct must be 0..={MAX_RESIST_PCT}, got {_0}")]
pub struct ResistPctOutOfRange(pub u16);

impl ResistPct {
    /// Saturating constructor: any value above [`MAX_RESIST_PCT`] is
    /// clamped to the cap.
    pub const fn new(value: u16) -> Self {
        let clamped = if value > MAX_RESIST_PCT {
            MAX_RESIST_PCT
        } else {
            value
        };
        Self(clamped)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ResistPct {
    type Error = ResistPctOutOfRange;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > MAX_RESIST_PCT {
            Err(ResistPctOutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl From<ResistPct> for u16 {
    fn from(value: ResistPct) -> Self {
        value.0
    }
}

/// Trait implemented by any type that can transform incoming damage
/// based on the supplied [`DamageContext`].
pub trait ModifyDamage {
    /// Return the (possibly reduced) `Health` damage that should be
    /// applied to the target after this modifier runs.
    fn modify(&self, base: Health, ctx: &DamageContext) -> Health;
}

/// Component-level damage modifier. Multiple modifiers can be composed
/// at the system level if richer chains are needed later.
#[derive(Component, Clone, Copy, Debug)]
pub enum DamageModifier {
    /// Subtract a flat amount from incoming damage (saturating).
    Flat(Health),
    /// Subtract a percentage of incoming damage, regardless of context.
    PercentReduction(ResistPct),
    /// Subtract a percentage of incoming damage only when the context is
    /// `DamageContext::Physical { .. }`.
    PhysicalResist(ResistPct),
}

impl ModifyDamage for DamageModifier {
    fn modify(&self, base: Health, ctx: &DamageContext) -> Health {
        match self {
            Self::Flat(reduction) => saturating_sub_health(base, *reduction),
            Self::PercentReduction(pct) => apply_percent_reduction(base, *pct),
            Self::PhysicalResist(pct) => match ctx {
                DamageContext::Physical { .. } => apply_percent_reduction(base, *pct),
                _ => base,
            },
        }
    }
}

fn saturating_sub_health(base: Health, reduction: Health) -> Health {
    Health(base.0.saturating_sub(reduction.0))
}

/// Compute `base * (MAX_RESIST_PCT - pct) / MAX_RESIST_PCT` using only
/// `u32` arithmetic (no `as` casts) and saturating back into `u16`.
fn apply_percent_reduction(base: Health, pct: ResistPct) -> Health {
    let pct_u32 = u32::from(pct.get());
    let max_u32 = u32::from(MAX_RESIST_PCT);
    let kept_pct = max_u32.saturating_sub(pct_u32);
    let base_u32 = u32::from(base.0);
    let scaled = base_u32.saturating_mul(kept_pct) / max_u32;
    let final_u16 = u16::try_from(scaled).unwrap_or(u16::MAX);
    Health(final_u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::hardness::Hardness;

    const TEN_PCT: ResistPct = ResistPct::new(10);
    const FIFTY_PCT: ResistPct = ResistPct::new(50);
    const HUNDRED_PCT: ResistPct = ResistPct::new(MAX_RESIST_PCT);
    const HUNDRED_HEALTH: Health = Health(100);

    fn physical_ctx() -> DamageContext {
        DamageContext::Physical {
            hardness_reduction: Hardness(0),
        }
    }

    #[test]
    fn flat_saturates_at_zero() {
        let modifier = DamageModifier::Flat(Health(200));
        assert_eq!(modifier.modify(HUNDRED_HEALTH, &physical_ctx()), Health(0));
    }

    #[test]
    fn percent_reduction_halves_damage() {
        let modifier = DamageModifier::PercentReduction(FIFTY_PCT);
        assert_eq!(modifier.modify(HUNDRED_HEALTH, &physical_ctx()), Health(50));
    }

    #[test]
    fn physical_resist_only_applies_to_physical() {
        let modifier = DamageModifier::PhysicalResist(TEN_PCT);
        assert_eq!(modifier.modify(HUNDRED_HEALTH, &physical_ctx()), Health(90));
        assert_eq!(
            modifier.modify(HUNDRED_HEALTH, &DamageContext::Custom),
            HUNDRED_HEALTH
        );
    }

    #[test]
    fn full_resist_zeroes_damage() {
        let modifier = DamageModifier::PercentReduction(HUNDRED_PCT);
        assert_eq!(modifier.modify(HUNDRED_HEALTH, &physical_ctx()), Health(0));
    }

    #[test]
    fn try_from_rejects_over_cap() {
        assert!(ResistPct::try_from(101).is_err());
        let parsed = ResistPct::try_from(75).expect("75 is within range");
        assert_eq!(parsed.get(), 75);
    }

    #[test]
    fn new_clamps_over_cap() {
        assert_eq!(ResistPct::new(250).get(), MAX_RESIST_PCT);
    }
}
