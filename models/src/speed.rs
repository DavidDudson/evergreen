use bevy::prelude::Component;
use derive_more::{Add, AddAssign, Display, From, Mul, MulAssign};

// Speed in tiles per second (1 tile = 5ft)
#[derive(Component, From, Add, Mul, MulAssign, AddAssign, Display, Debug, Default)]
pub struct Speed(pub u16);
