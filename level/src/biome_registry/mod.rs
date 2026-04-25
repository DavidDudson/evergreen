//! Central registry of per-biome content (trees, decorations, grass,
//! creatures, weather weights, terrain tilesets, leaf-particle assets).
//!
//! `BiomeRegistry` is a `Resource`; the level plugin inserts a single
//! instance built from `BiomeRegistry::build()` at app startup. Spawn
//! systems consume it via `Res<BiomeRegistry>` instead of reaching into
//! private const arrays scattered across modules.

use bevy::prelude::Resource;
use models::decoration::Biome;

pub mod biomes;
pub mod creature;
pub mod decoration;
pub mod grass;
pub mod tree;
pub mod weather;

pub use creature::CreatureSpec;
pub use decoration::DecorationSpec;
pub use tree::TreeSpawnConfig;
pub use weather::{LeafParticle, WEATHER_KINDS, WEIGHT_COUNT};

/// Registry resource exposing per-biome spawn data.
#[derive(Resource, Default)]
pub struct BiomeRegistry;

impl BiomeRegistry {
    /// Construct a fresh registry. All data lives in static const arrays;
    /// the resource is just a typed entry point.
    pub fn build() -> Self {
        Self
    }

    /// Tree spawn config for a biome alignment value (0..=100).
    pub fn tree_config(&self, alignment: u8) -> &'static TreeSpawnConfig {
        tree::config_for(Biome::from_alignment(alignment))
    }

    /// Tree sprite-path pool for a biome alignment value.
    pub fn trees(&self, alignment: u8) -> &'static [&'static str] {
        self.tree_config(alignment).paths
    }

    /// Creature spawn pool for a biome alignment value.
    pub fn creatures(&self, alignment: u8) -> &'static [CreatureSpec] {
        creature::pool_for(Biome::from_alignment(alignment))
    }

    /// Decoration spawn pool for a biome alignment value.
    pub fn decorations(&self, alignment: u8) -> &'static [DecorationSpec] {
        decoration::pool_for(Biome::from_alignment(alignment))
    }

    /// Grass sprite-path pool for a biome alignment value.
    pub fn grass(&self, alignment: u8) -> &'static [&'static str] {
        grass::pool_for(Biome::from_alignment(alignment))
    }

    /// Per-biome weather transition weights.
    pub fn weather_weights(&self, alignment: u8) -> &'static [u32; WEIGHT_COUNT] {
        weather::weights_for(Biome::from_alignment(alignment))
    }

    /// Leaf-style weather particle sprite/variant for a biome alignment.
    pub fn leaf_particle(&self, alignment: u8) -> &'static LeafParticle {
        weather::leaf_for(Biome::from_alignment(alignment))
    }

    /// Asset path of the terrain tileset for a biome alignment.
    pub fn terrain_tileset(&self, alignment: u8) -> &'static str {
        biomes::terrain_path_for(Biome::from_alignment(alignment))
    }
}
