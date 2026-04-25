//! Creature spawn data per biome.

use models::creature::MovementType;
use models::decoration::Biome;

/// Speeds (pixels per second).
const SPEED_MOUSE_PX: f32 = 32.0;
const SPEED_PIGEON_PX: f32 = 48.0;
const SPEED_CAT_PX: f32 = 40.0;
const SPEED_BUTTERFLY_PX: f32 = 24.0;
const SPEED_FROG_PX: f32 = 48.0;
const SPEED_RABBIT_PX: f32 = 56.0;
const SPEED_SONGBIRD_PX: f32 = 40.0;
const SPEED_COCKROACH_PX: f32 = 40.0;
const SPEED_CROW_PX: f32 = 48.0;
const SPEED_BAT_PX: f32 = 56.0;
const SPEED_SPIDER_PX: f32 = 32.0;

pub struct CreatureSpec {
    pub path: &'static str,
    pub movement: MovementType,
    pub speed: f32,
}

const CITY_CREATURES: &[CreatureSpec] = &[
    CreatureSpec {
        path: "sprites/creatures/city/mouse.webp",
        movement: MovementType::Ground,
        speed: SPEED_MOUSE_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/city/pigeon.webp",
        movement: MovementType::Flying,
        speed: SPEED_PIGEON_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/city/cat.webp",
        movement: MovementType::Ground,
        speed: SPEED_CAT_PX,
    },
];

const GREENWOOD_CREATURES: &[CreatureSpec] = &[
    CreatureSpec {
        path: "sprites/creatures/greenwood/butterfly.webp",
        movement: MovementType::Flying,
        speed: SPEED_BUTTERFLY_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/greenwood/frog.webp",
        movement: MovementType::Ground,
        speed: SPEED_FROG_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/greenwood/rabbit.webp",
        movement: MovementType::Ground,
        speed: SPEED_RABBIT_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/greenwood/songbird.webp",
        movement: MovementType::Flying,
        speed: SPEED_SONGBIRD_PX,
    },
];

const DARKWOOD_CREATURES: &[CreatureSpec] = &[
    CreatureSpec {
        path: "sprites/creatures/darkwood/cockroach.webp",
        movement: MovementType::Ground,
        speed: SPEED_COCKROACH_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/darkwood/crow.webp",
        movement: MovementType::Flying,
        speed: SPEED_CROW_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/darkwood/bat.webp",
        movement: MovementType::Flying,
        speed: SPEED_BAT_PX,
    },
    CreatureSpec {
        path: "sprites/creatures/darkwood/spider.webp",
        movement: MovementType::Ground,
        speed: SPEED_SPIDER_PX,
    },
];

pub fn pool_for(biome: Biome) -> &'static [CreatureSpec] {
    match biome {
        Biome::City => CITY_CREATURES,
        Biome::Greenwood => GREENWOOD_CREATURES,
        Biome::Darkwood => DARKWOOD_CREATURES,
    }
}
