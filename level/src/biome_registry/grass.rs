//! Grass-tuft sprite paths per biome.

use models::decoration::Biome;

const CITY_GRASS: &[&str] = &[
    "sprites/scenery/grass/city/grass_small.webp",
    "sprites/scenery/grass/city/grass_medium.webp",
    "sprites/scenery/grass/city/grass_large.webp",
];

const GREENWOOD_GRASS: &[&str] = &[
    "sprites/scenery/grass/greenwood/grass_small.webp",
    "sprites/scenery/grass/greenwood/grass_medium.webp",
    "sprites/scenery/grass/greenwood/grass_large.webp",
];

const DARKWOOD_GRASS: &[&str] = &[
    "sprites/scenery/grass/darkwood/grass_small.webp",
    "sprites/scenery/grass/darkwood/grass_medium.webp",
    "sprites/scenery/grass/darkwood/grass_large.webp",
];

pub fn pool_for(biome: Biome) -> &'static [&'static str] {
    match biome {
        Biome::City => CITY_GRASS,
        Biome::Greenwood => GREENWOOD_GRASS,
        Biome::Darkwood => DARKWOOD_GRASS,
    }
}
