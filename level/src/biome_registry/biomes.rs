//! Per-biome terrain tileset paths.

use models::decoration::Biome;

const CITY_TERRAIN: &str = "sprites/terrain/terrain_wang_city.webp";
const GREENWOOD_TERRAIN: &str = "sprites/terrain/terrain_wang.webp";
const DARKWOOD_TERRAIN: &str = "sprites/terrain/terrain_wang_darkwood.webp";

pub fn terrain_path_for(biome: Biome) -> &'static str {
    match biome {
        Biome::City => CITY_TERRAIN,
        Biome::Greenwood => GREENWOOD_TERRAIN,
        Biome::Darkwood => DARKWOOD_TERRAIN,
    }
}
