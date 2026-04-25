//! Water bodies: ponds, hot springs, multi-area lakes, rivers, ocean.
//!
//! `tiles` owns the data model (`WaterMap`, `WaterKind`); `generate`
//! orchestrates pond/hot-spring/lake flood-fills; `rivers` and `ocean`
//! handle their respective bands; `shore` spawns sprites + colliders +
//! stepping stones; `animation` runs the per-frame surface alpha pulse.

mod animation;
mod generate;
mod ocean;
mod rivers;
mod shore;
mod tiles;

pub use animation::{animate_water_surface, AnimatedWater};
pub use generate::generate_water_bodies;
pub use shore::{despawn_stones, despawn_water, sand_mask, spawn_area_water, SteppingStone};
pub use tiles::{WaterKey, WaterKind, WaterMap, WaterTile};
