//! Weather plugin internals: state machine, wind syncing, and the various
//! particle effects (leaves, raindrops, fireflies, dust motes, fog).
//!
//! All per-biome content (transition weights, leaf sprite paths) is sourced
//! from `crate::biome_registry::BiomeRegistry`.

mod dust;
mod firefly;
mod fog;
mod helpers;
mod leaves_rain;
mod state;
mod wind;

pub use dust::{dust_mote_active, spawn_dust_motes};
pub use firefly::{animate_fireflies, firefly_active, spawn_fireflies, Firefly};
pub use fog::{fog_active, spawn_fog_patches};
pub use leaves_rain::{
    despawn_weather_particles, spawn_weather_particles, update_weather_particles,
};
pub use state::weather_state_machine;
pub use wind::sync_wind_strength;
