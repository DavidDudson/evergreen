use derive_more::{Add, AddAssign, AsRef, Display, From, Mul, MulAssign, Sub, SubAssign};
use models::hardness::Hardness;

/// Percent expressed in hundredths (0..=10_000 means 0%..=100%).
///
/// Stored as `u16` so a percentage value like 75% is `ImpactRatio(7_500)`.
/// Higher values denote a stronger impact / more aggressive scaling.
#[derive(
    From,
    Add,
    Sub,
    Mul,
    AddAssign,
    SubAssign,
    MulAssign,
    AsRef,
    Display,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Debug,
    Clone,
    Copy,
    Default,
)]
pub struct ImpactRatio(pub u16);

/// Per-event metadata describing the source/flavour of incoming damage.
///
/// Carried on `DamageEvent` / `DeathEvent` / `DeathOccurred` so that
/// downstream systems and damage modifiers can react to the cause of damage
/// without re-deriving it from the attack itself.
#[derive(Clone, Copy, Debug)]
pub enum DamageContext {
    /// Standard physical hit. `hardness_reduction` is the amount of damage
    /// already absorbed by the source's `Hardness` before the event was
    /// emitted (informational; further mitigation can be applied via
    /// `DamageModifier`).
    Physical { hardness_reduction: Hardness },
    /// Damage caused by impact / falling. `impact_ratio` records how
    /// severe the fall was (used by some `DamageModifier`s for scaling).
    Fall { impact_ratio: ImpactRatio },
    /// Catch-all for bespoke damage sources (scripts, traps, environment).
    Custom,
}

impl DamageContext {
    /// Convenience default for systems that emit a vanilla physical hit
    /// without any pre-computed hardness reduction.
    pub const DEFAULT_PHYSICAL: Self = Self::Physical {
        hardness_reduction: Hardness(0),
    };
}

impl Default for DamageContext {
    fn default() -> Self {
        Self::DEFAULT_PHYSICAL
    }
}
