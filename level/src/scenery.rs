use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::decoration::Biome;
use models::layer::Layer;
use models::lighting::{
    TREE_CANOPY_HALF_PX, TREE_CANOPY_OFFSET_PX, TREE_TRUNK_HALF_PX, TREE_TRUNK_OFFSET_PX,
};
use models::reveal::{RevealState, Revealable};
use models::scenery::{Rustling, Scenery, SceneryCollider};
use models::shadow::{TREE_SHADOW_HALF_PX, TREE_SHADOW_OFFSET_Y_PX};

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::blending;
use crate::light_occluders::spawn_occluder;
use crate::shadows::{spawn_drop_shadow, DropShadowAssets};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::{tile_hash, Terrain};
use crate::world::WorldMap;

// Trees are 48x64 px, anchored at BOTTOM_CENTER.
const TREE_WIDTH_PX: f32 = 48.0;
const TREE_HEIGHT_PX: f32 = 64.0;
// Trunk-only collider: roughly 1x1 tile at the base.
const TREE_COLLIDER_HALF: Vec2 = Vec2::new(8.0, 8.0);
const TREE_COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, 4.0);

// Peak rotation angle for rustle animation (radians).
const RUSTLE_MAX_ANGLE: f32 = 0.15;

// Z sub-layer scale for back-to-front (y-sort) drawing.
const Y_SORT_SCALE: f32 = 0.001;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

// ---------------------------------------------------------------------------
// Tree definitions
// ---------------------------------------------------------------------------

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

fn tree_pool(alignment: u8) -> &'static [&'static str] {
    match Biome::from_alignment(alignment) {
        Biome::City => CITY_TREES,
        Biome::Greenwood => GREENWOOD_TREES,
        Biome::Darkwood => DARKWOOD_TREES,
    }
}

// ---------------------------------------------------------------------------
// Density
// ---------------------------------------------------------------------------

/// Tree spawn probability (0-100 hash threshold) by biome.
fn tree_threshold(alignment: u8, ed: u32) -> usize {
    if ed > 6 {
        return 0;
    }
    let base: usize = match Biome::from_alignment(alignment) {
        Biome::City => 5,
        Biome::Greenwood => 45,
        Biome::Darkwood => 80,
    };
    if ed <= 2 {
        base + 15
    } else if ed <= 4 {
        base + 10
    } else {
        base
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Despawn all scenery on game exit.
pub fn despawn_scenery(mut commands: Commands, query: Query<Entity, With<Scenery>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Spawn scenery for a single area at its absolute world position.
/// Called from `spawning::ensure_area_spawned`.
pub fn spawn_area_scenery_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shadow_assets: &DropShadowAssets,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    spawn_area_scenery(commands, asset_server, shadow_assets, area, area_pos, world);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Simplified clear check -- 1x1 trunk footprint.
fn clear_for_tree(area: &Area, xu: u32, yu: u32) -> bool {
    area.terrain_at(xu, yu) == Some(Terrain::Grass)
}

fn spawn_area_scenery(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shadow_assets: &DropShadowAssets,
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

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);

            let hash = tile_hash(xu, yu, area_seed) % 100;
            let ed = edge_dist(x, y);

            let blend = blending::blend_at(area.alignment, xu, yu, area_pos, world);
            let threshold = tree_threshold(blend.alignment, ed);

            if hash < threshold && clear_for_tree(area, xu, yu) {
                // In the blend zone, probabilistically pick from the neighbor's
                // tree pool based on blend factor (Minecraft-style mixing).
                let mix_hash = tile_hash(xu, yu, area_seed.wrapping_add(20)) % 100;
                #[allow(clippy::as_conversions)]
                let mix_threshold = (blend.factor * 100.0) as usize;
                let pool_alignment = if mix_hash < mix_threshold {
                    blend.neighbor_alignment.unwrap_or(area.alignment)
                } else {
                    area.alignment
                };
                let pool = tree_pool(pool_alignment);
                let variant = tile_hash(xu, yu, area_seed.wrapping_add(10)) % pool.len();
                let def = &pool[variant];
                let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
                let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
                spawn_tree(commands, asset_server, shadow_assets, def, world_x, world_y);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Spawner
// ---------------------------------------------------------------------------

fn spawn_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shadow_assets: &DropShadowAssets,
    path: &'static str,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;
    let parent = commands
        .spawn((
            Scenery,
            SceneryCollider {
                half_extents: TREE_COLLIDER_HALF,
                center_offset: TREE_COLLIDER_OFFSET,
            },
            Revealable {
                canopy_height_px: TREE_HEIGHT_PX,
                half_width_px: TREE_WIDTH_PX / 2.0,
                revealed_full_alpha: 0.3,
            },
            RevealState::default(),
            Sprite {
                image: asset_server.load(path),
                custom_size: Some(Vec2::new(TREE_WIDTH_PX, TREE_HEIGHT_PX)),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(world_x, world_y, z),
        ))
        .id();

    spawn_occluder(commands, parent, TREE_TRUNK_HALF_PX, TREE_TRUNK_OFFSET_PX);
    spawn_occluder(commands, parent, TREE_CANOPY_HALF_PX, TREE_CANOPY_OFFSET_PX);
    spawn_drop_shadow(
        commands,
        shadow_assets,
        parent,
        TREE_SHADOW_HALF_PX,
        TREE_SHADOW_OFFSET_Y_PX,
    );
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

pub fn animate_rustle(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Rustling)>,
) {
    use std::f32::consts::PI;
    for (entity, mut tf, mut rustling) in &mut query {
        rustling.timer.tick(time.delta());
        let progress = rustling.timer.fraction();
        let angle = (progress * PI * 4.0).sin() * (1.0 - progress) * RUSTLE_MAX_ANGLE;
        tf.rotation = Quat::from_rotation_z(angle);
        if rustling.timer.is_finished() {
            tf.rotation = Quat::IDENTITY;
            commands.entity(entity).remove::<Rustling>();
        }
    }
}

fn edge_dist(x: u16, y: u16) -> u32 {
    let xu = u32::from(x);
    let yu = u32::from(y);
    let right_dist = u32::from(MAP_WIDTH).saturating_sub(1).saturating_sub(xu);
    let top_dist = u32::from(MAP_HEIGHT).saturating_sub(1).saturating_sub(yu);
    xu.min(right_dist).min(yu).min(top_dist)
}
