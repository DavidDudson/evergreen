use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::decoration::Biome;
use models::layer::Layer;
use models::scenery::{Rustling, Scenery, SceneryCollider};

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::blending;
use crate::spawning::{TILE_SIZE_PX, area_world_offset};
use crate::terrain::{tile_hash, Terrain};
use crate::world::WorldMap;

// Trees are 2x2 tiles (32x32 px), anchored at BOTTOM_CENTER of the base tile.
const TREE_WIDTH_PX: f32 = 32.0;
const TREE_HEIGHT_PX: f32 = 32.0;

// Full-sprite AABB colliders (trees only).
// Tree entity is at BOTTOM_CENTER, so sprite centre is (0, +16) above entity.
const TREE_COLLIDER_HALF: Vec2 = Vec2::new(16.0, 16.0);
const TREE_COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, TREE_HEIGHT_PX / 2.0);

// Peak rotation angle for rustle animation (radians).
const RUSTLE_MAX_ANGLE: f32 = 0.15;

// Z sub-layer scale for back-to-front (y-sort) drawing.
// Lower world_y = lower on screen = closer to viewer = higher z.
const Y_SORT_SCALE: f32 = 0.001;

const TREE_ASSETS: [&str; 2] = [
    "sprites/scenery/trees/tree_oak.webp",
    "sprites/scenery/trees/tree_pine.webp",
];

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Despawn all scenery on game exit.
pub fn despawn_scenery(
    mut commands: Commands,
    query: Query<Entity, With<Scenery>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Spawn scenery for a single area at its absolute world position.
/// Called from `spawning::ensure_area_spawned`.
pub fn spawn_area_scenery_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    spawn_area_scenery(commands, asset_server, area, area_pos, world);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Tree spawn probability (0-100 hash threshold) by biome.
/// City: sparse. Greenwood: moderate. Darkwood: dense.
/// Denser near edges, sparser toward center.
fn tree_threshold(alignment: u8, ed: u32) -> usize {
    if ed > 6 {
        return 0;
    }

    // Base density per biome (out of 100).
    let base: usize = match Biome::from_alignment(alignment) {
        Biome::City => 8,
        Biome::Greenwood => 30,
        Biome::Darkwood => 65,
    };

    // Edge proximity bonus.
    if ed <= 2 {
        base + 20
    } else if ed <= 4 {
        base + 10
    } else {
        base
    }
}

fn spawn_area_scenery(
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

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);

            if area.terrain_at(xu, yu) != Some(Terrain::Grass) {
                continue;
            }

            let hash = tile_hash(xu, yu, area_seed) % 100;
            let ed = edge_dist(x, y);

            // Use blended alignment for biome-appropriate density near borders.
            let effective_alignment = blending::blended_alignment(
                area.alignment,
                xu,
                yu,
                area_pos,
                world,
            );
            let threshold = tree_threshold(effective_alignment, ed);

            if hash < threshold && clear_for_tree(area, xu, yu) {
                let variant =
                    tile_hash(xu, yu, area_seed.wrapping_add(10)) % TREE_ASSETS.len();
                let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
                let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
                spawn_tree(commands, asset_server, TREE_ASSETS[variant], world_x, world_y);
            }
        }
    }
}

/// Returns `true` when a 2x2 tree at `(xu, yu)` will not visually overlap any path tile.
fn clear_for_tree(area: &Area, xu: u32, yu: u32) -> bool {
    let grass = Some(Terrain::Grass);
    let above = yu + 1 >= u32::from(MAP_HEIGHT) || area.terrain_at(xu, yu + 1) == grass;
    let left = xu == 0 || area.terrain_at(xu - 1, yu) == grass;
    let right = xu + 1 >= u32::from(MAP_WIDTH) || area.terrain_at(xu + 1, yu) == grass;
    above && left && right
}

// ---------------------------------------------------------------------------
// Spawner
// ---------------------------------------------------------------------------

fn spawn_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::World.z_f32() - world_y * Y_SORT_SCALE;
    commands.spawn((
        Scenery,
        SceneryCollider {
            half_extents: TREE_COLLIDER_HALF,
            center_offset: TREE_COLLIDER_OFFSET,
        },
        Sprite {
            image: asset_server.load(path),
            custom_size: Some(Vec2::new(TREE_WIDTH_PX, TREE_HEIGHT_PX)),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(world_x, world_y, z),
    ));
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
