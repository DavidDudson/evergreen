use bevy::math::IVec2;
use bevy::prelude::*;
use models::decoration::{Biome, Decoration};
use models::layer::Layer;
use models::reveal::{Revealable, RevealState};
use models::scenery::Rustleable;

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::blending;
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::world::WorldMap;

#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

const Y_SORT_SCALE: f32 = 0.001;

const MIN_DECORATIONS: usize = 10;
const MAX_DECORATIONS: usize = 15;

/// Inset from area edges where decorations can spawn (avoids tree-dense borders).
const EDGE_INSET: u16 = 2;

struct DecorationDef {
    path: &'static str,
    width_px: f32,
    height_px: f32,
    rustleable: bool,
    revealable: bool,
}

const DARKWOOD_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/darkwood/poison_mushroom.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/thorn_bush.webp", width_px: 24.0, height_px: 24.0, rustleable: true, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/spider_web.webp", width_px: 32.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/dead_branch.webp", width_px: 24.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/glowing_fungus.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/skull_bones.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/darkwood/dark_flower.webp", width_px: 16.0, height_px: 16.0, rustleable: true, revealable: false },
];

const GREENWOOD_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/greenwood/wildflower.webp", width_px: 16.0, height_px: 16.0, rustleable: true, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/herb_cluster.webp", width_px: 16.0, height_px: 16.0, rustleable: true, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/twig_pile.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/berry_bush.webp", width_px: 24.0, height_px: 24.0, rustleable: true, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/fern.webp", width_px: 24.0, height_px: 24.0, rustleable: true, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/mossy_rock.webp", width_px: 24.0, height_px: 24.0, rustleable: false, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/greenwood/fallen_log.webp", width_px: 24.0, height_px: 16.0, rustleable: false, revealable: false },
];

const CITY_DECORATIONS: &[DecorationDef] = &[
    DecorationDef { path: "sprites/scenery/decorations/city/wooden_crate.webp", width_px: 24.0, height_px: 24.0, rustleable: false, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/city/barrel.webp", width_px: 24.0, height_px: 24.0, rustleable: false, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/city/hay_bale.webp", width_px: 24.0, height_px: 24.0, rustleable: false, revealable: true },
    DecorationDef { path: "sprites/scenery/decorations/city/sack.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/flower_pot.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/wooden_sign.webp", width_px: 16.0, height_px: 16.0, rustleable: false, revealable: false },
    DecorationDef { path: "sprites/scenery/decorations/city/cart.webp", width_px: 32.0, height_px: 16.0, rustleable: false, revealable: false },
];

/// Spawn 10-15 decorations for a single area.
pub fn spawn_area_decorations(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {

    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223));

    // Decoration-specific salt to avoid overlapping tree positions.
    let deco_seed = area_seed.wrapping_add(999);

    // Collect candidate grass tiles (inset from edges where trees dominate).
    let mut candidates: Vec<(u32, u32)> = Vec::new();
    for x in EDGE_INSET..(MAP_WIDTH - EDGE_INSET) {
        for y in EDGE_INSET..(MAP_HEIGHT - EDGE_INSET) {
            let xu = u32::from(x);
            let yu = u32::from(y);
            if area.terrain_at(xu, yu) == Some(Terrain::Grass) {
                candidates.push((xu, yu));
            }
        }
    }

    if candidates.is_empty() {
        return;
    }

    // Deterministically shuffle candidates using area seed.
    let len = candidates.len();
    let mut rng = u64::from(deco_seed);
    for i in (1..len).rev() {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let j = (rng % u64::try_from(i + 1).expect("i+1 fits u64")) as usize;
        candidates.swap(i, j);
    }

    // Pick decoration count (10-15) deterministically.
    rng = lcg(rng);
    #[allow(clippy::as_conversions)]
    let range = (MAX_DECORATIONS - MIN_DECORATIONS + 1) as u64;
    #[allow(clippy::as_conversions)]
    let count = MIN_DECORATIONS + (rng % range) as usize;
    let count = count.min(candidates.len());

    for (i, &(xu, yu)) in candidates.iter().take(count).enumerate() {
        // Per-tile blended alignment for biome-appropriate decoration pool.
        let effective_alignment = blending::blended_alignment(
            area.alignment,
            xu,
            yu,
            area_pos,
            world,
        );
        let biome = Biome::from_alignment(effective_alignment);
        let pool = match biome {
            Biome::City => CITY_DECORATIONS,
            Biome::Greenwood => GREENWOOD_DECORATIONS,
            Biome::Darkwood => DARKWOOD_DECORATIONS,
        };

        let variant = tile_hash(
            xu,
            yu,
            deco_seed.wrapping_add(u32::try_from(i).expect("i fits u32")),
        ) % pool.len();
        let def = &pool[variant];

        let world_x = base_offset_x
            + f32::from(u16::try_from(xu).expect("xu fits u16")) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y
            + f32::from(u16::try_from(yu).expect("yu fits u16")) * tile_px
            + tile_px / 2.0;

        spawn_decoration(commands, asset_server, def, world_x, world_y);
    }
}

/// Despawn all decorations on game exit.
pub fn despawn_decorations(
    mut commands: Commands,
    query: Query<Entity, With<Decoration>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_decoration(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: &DecorationDef,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;
    let mut entity = commands.spawn((
        Decoration,
        Sprite {
            image: asset_server.load(def.path),
            custom_size: Some(Vec2::new(def.width_px, def.height_px)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, z),
    ));

    if def.revealable {
        entity.insert((
            Revealable {
                canopy_height_px: def.height_px,
                half_width_px: def.width_px / 2.0,
                revealed_full_alpha: 0.3,
            },
            RevealState::default(),
        ));
    }

    if def.rustleable {
        entity.insert(Rustleable);
    }
}

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
