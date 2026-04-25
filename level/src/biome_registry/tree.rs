//! Tree spawn data per biome.

use models::decoration::Biome;

/// Edge-distance threshold beyond which trees do not spawn (used as a
/// "dead zone" carve-out at the centre of each area).
const DEAD_ZONE_DARKWOOD: u32 = 4;
const DEAD_ZONE_DEFAULT: u32 = 6;

/// Base spawn threshold (out of 100) by biome.
const BASE_THRESHOLD_CITY: usize = 3;
const BASE_THRESHOLD_GREENWOOD: usize = 35;
const BASE_THRESHOLD_DARKWOOD: usize = 65;

/// Bonus applied to threshold inside the densest border ring (`ed <= 2`).
const BORDER_BONUS: usize = 12;
/// Bonus applied to threshold in the secondary border ring (`ed <= 4`).
const NEAR_BORDER_BONUS: usize = 4;
/// Divisor applied to threshold in the deep middle band.
const MIDDLE_BAND_DIVISOR: usize = 3;

/// Edge-distance limit for the densest border ring.
const ED_BORDER: u32 = 2;
/// Edge-distance limit for the secondary border ring.
const ED_NEAR_BORDER: u32 = 4;

/// Tree spawn configuration for one biome.
pub struct TreeSpawnConfig {
    pub paths: &'static [&'static str],
    pub base_threshold: usize,
    pub dead_zone_ed: u32,
}

impl TreeSpawnConfig {
    /// Spawn probability (0..=100 hash threshold) at edge-distance `ed`.
    pub fn threshold_at(&self, ed: u32) -> usize {
        if ed > self.dead_zone_ed {
            return 0;
        }
        if ed <= ED_BORDER {
            self.base_threshold + BORDER_BONUS
        } else if ed <= ED_NEAR_BORDER {
            self.base_threshold + NEAR_BORDER_BONUS
        } else {
            self.base_threshold / MIDDLE_BAND_DIVISOR
        }
    }
}

const CITY_TREES: &[&str] = &[
    "sprites/scenery/trees/city/tree_city_ornamental.webp",
    "sprites/scenery/trees/city/tree_city_fruit.webp",
];

const GREENWOOD_TREES: &[&str] = &[
    "sprites/scenery/trees/greenwood/tree_green_oak.webp",
    "sprites/scenery/trees/greenwood/tree_green_birch.webp",
    "sprites/scenery/trees/greenwood/tree_green_maple.webp",
];

const DARKWOOD_TREES: &[&str] = &[
    "sprites/scenery/trees/darkwood/tree_dark_gnarled.webp",
    "sprites/scenery/trees/darkwood/tree_dark_dead.webp",
    "sprites/scenery/trees/darkwood/tree_dark_willow.webp",
];

pub const CITY_TREE_CONFIG: TreeSpawnConfig = TreeSpawnConfig {
    paths: CITY_TREES,
    base_threshold: BASE_THRESHOLD_CITY,
    dead_zone_ed: DEAD_ZONE_DEFAULT,
};

pub const GREENWOOD_TREE_CONFIG: TreeSpawnConfig = TreeSpawnConfig {
    paths: GREENWOOD_TREES,
    base_threshold: BASE_THRESHOLD_GREENWOOD,
    dead_zone_ed: DEAD_ZONE_DEFAULT,
};

pub const DARKWOOD_TREE_CONFIG: TreeSpawnConfig = TreeSpawnConfig {
    paths: DARKWOOD_TREES,
    base_threshold: BASE_THRESHOLD_DARKWOOD,
    dead_zone_ed: DEAD_ZONE_DARKWOOD,
};

pub fn config_for(biome: Biome) -> &'static TreeSpawnConfig {
    match biome {
        Biome::City => &CITY_TREE_CONFIG,
        Biome::Greenwood => &GREENWOOD_TREE_CONFIG,
        Biome::Darkwood => &DARKWOOD_TREE_CONFIG,
    }
}
