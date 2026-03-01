use derive_more::{Add, AddAssign, Display, From, Mul, MulAssign, Sub, SubAssign};

/// A measurement in tiles (1 tile = 5ft).
#[derive(
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
pub struct Tile(pub u16);
