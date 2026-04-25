//! Persisted snapshot of the player's place in the multiverse.
//!
//! Captures just enough state to regenerate the current map verbatim on
//! reload (id + seed + alignment) plus the global `maps_traversed` counter
//! used to scale enemy difficulty.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Resource saved/loaded by `save::SavePlugin`. Sentinel `valid = false`
/// means a fresh game; a freshly-installed save file lands in this state
/// and triggers the normal random root-map bootstrap.
#[derive(Resource, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct MultiverseSave {
    /// `true` once any map has been generated and persisted. Used to
    /// distinguish a never-played save (skip restore) from one that's been
    /// touched (restore exact state).
    pub valid: bool,
    /// Identifier of the map the player was on when the save was last
    /// updated.
    pub current_id: u32,
    /// RNG seed for the current map -- regen produces an identical layout.
    pub current_seed: u64,
    /// Biome alignment the current map was generated at.
    pub current_alignment: u8,
    /// How many portals the player has crossed in total. Drives
    /// per-area enemy counts via `enemy_count_for_traversal`.
    pub maps_traversed: u32,
}
