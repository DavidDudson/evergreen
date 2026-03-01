use bevy::prelude::Component;
use derive_more::{Add, AddAssign, Display, From, Mul, MulAssign, Sub, SubAssign};

#[derive(
    Component,
    From,
    Add,
    Mul,
    MulAssign,
    AddAssign,
    Sub,
    SubAssign,
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
pub struct Distance(pub u16);

/// Clamp a float to the valid `u16` range and round.
#[allow(clippy::as_conversions)]
fn f64_to_u16_saturating(value: f64) -> u16 {
    value.round().clamp(0.0, f64::from(u16::MAX)) as u16
}

impl From<f32> for Distance {
    fn from(value: f32) -> Self {
        Distance(f64_to_u16_saturating(f64::from(value)))
    }
}

impl From<f64> for Distance {
    fn from(value: f64) -> Self {
        Distance(f64_to_u16_saturating(value))
    }
}
