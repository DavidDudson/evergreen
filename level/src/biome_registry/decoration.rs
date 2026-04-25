//! Decoration spawn data per biome.

use models::decoration::Biome;
use models::tags::{tag, PlacementRequirement};

pub struct DecorationSpec {
    pub path: &'static str,
    pub width_px: f32,
    pub height_px: f32,
    pub rustleable: bool,
    pub revealable: bool,
    /// Placement constraint. Default = ground-only (legacy behaviour).
    pub placement: PlacementRequirement,
}

const ON_GROUND: PlacementRequirement = PlacementRequirement::requires(&[tag::GROUND]);

const DARKWOOD_DECORATIONS: &[DecorationSpec] = &[
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/poison_mushroom.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/thorn_bush.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: true,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/spider_web.webp",
        width_px: 32.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/dead_branch.webp",
        width_px: 24.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/glowing_fungus.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/skull_bones.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/darkwood/dark_flower.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: true,
        revealable: false,
        placement: ON_GROUND,
    },
];

const GREENWOOD_DECORATIONS: &[DecorationSpec] = &[
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/wildflower.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: true,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/herb_cluster.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: true,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/twig_pile.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/berry_bush.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: true,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/fern.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: true,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/mossy_rock.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: false,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/greenwood/fallen_log.webp",
        width_px: 24.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
];

const CITY_DECORATIONS: &[DecorationSpec] = &[
    DecorationSpec {
        path: "sprites/scenery/decorations/city/wooden_crate.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: false,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/barrel.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: false,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/hay_bale.webp",
        width_px: 24.0,
        height_px: 24.0,
        rustleable: false,
        revealable: true,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/sack.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/flower_pot.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/wooden_sign.webp",
        width_px: 16.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
    DecorationSpec {
        path: "sprites/scenery/decorations/city/cart.webp",
        width_px: 32.0,
        height_px: 16.0,
        rustleable: false,
        revealable: false,
        placement: ON_GROUND,
    },
];

pub fn pool_for(biome: Biome) -> &'static [DecorationSpec] {
    match biome {
        Biome::City => CITY_DECORATIONS,
        Biome::Greenwood => GREENWOOD_DECORATIONS,
        Biome::Darkwood => DARKWOOD_DECORATIONS,
    }
}
