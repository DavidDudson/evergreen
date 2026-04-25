//! Per-biome weather transition weights and leaf-particle assets.

use models::decoration::Biome;
use models::weather::{ParticleVariant, WeatherKind};

/// Number of biome transition weights per row.
pub const WEIGHT_COUNT: usize = 5;

/// Ordered table that maps a position in the weight arrays to a `WeatherKind`.
pub const WEATHER_KINDS: [WeatherKind; WEIGHT_COUNT] = [
    WeatherKind::Clear,
    WeatherKind::Breezy,
    WeatherKind::Windy,
    WeatherKind::Rain,
    WeatherKind::Storm,
];

const CITY_WEIGHTS: [u32; WEIGHT_COUNT] = [50, 25, 10, 10, 5];
const GREENWOOD_WEIGHTS: [u32; WEIGHT_COUNT] = [30, 30, 15, 15, 10];
const DARKWOOD_WEIGHTS: [u32; WEIGHT_COUNT] = [15, 20, 25, 25, 15];

pub fn weights_for(biome: Biome) -> &'static [u32; WEIGHT_COUNT] {
    match biome {
        Biome::City => &CITY_WEIGHTS,
        Biome::Greenwood => &GREENWOOD_WEIGHTS,
        Biome::Darkwood => &DARKWOOD_WEIGHTS,
    }
}

/// Sprite + variant for a leaf-style weather particle in a biome.
pub struct LeafParticle {
    pub path: &'static str,
    pub variant: ParticleVariant,
}

const CITY_LEAF: LeafParticle = LeafParticle {
    path: "sprites/particles/paper_scrap.webp",
    variant: ParticleVariant::PaperScrap,
};

const GREENWOOD_LEAF: LeafParticle = LeafParticle {
    path: "sprites/particles/green_leaf.webp",
    variant: ParticleVariant::GreenLeaf,
};

const DARKWOOD_LEAF: LeafParticle = LeafParticle {
    path: "sprites/particles/brown_leaf.webp",
    variant: ParticleVariant::BrownLeaf,
};

pub fn leaf_for(biome: Biome) -> &'static LeafParticle {
    match biome {
        Biome::City => &CITY_LEAF,
        Biome::Greenwood => &GREENWOOD_LEAF,
        Biome::Darkwood => &DARKWOOD_LEAF,
    }
}
