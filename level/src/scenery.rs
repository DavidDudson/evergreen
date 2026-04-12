use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::layer::Layer;
use models::scenery::{Rustleable, Rustling, Scenery, SceneryCollider};

use crate::area::{Area, AreaAlignment, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{TILE_SIZE_PX, area_world_offset};
use crate::terrain::{tile_hash, Terrain};

// Trees are 2x2 tiles (32x32 px), anchored at BOTTOM_CENTER of the base tile.
const TREE_WIDTH_PX: f32 = 32.0;
const TREE_HEIGHT_PX: f32 = 32.0;
const BUSH_SIZE_PX: f32 = 24.0;
const FLOWER_SIZE_PX: f32 = 16.0;

// Full-sprite AABB colliders (trees only).
// Tree entity is at BOTTOM_CENTER, so sprite centre is (0, +16) above entity.
const TREE_COLLIDER_HALF: Vec2 = Vec2::new(16.0, 16.0);
const TREE_COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, TREE_HEIGHT_PX / 2.0);

// Peak rotation angle for rustle animation (radians).
const RUSTLE_MAX_ANGLE: f32 = 0.15;

// Z sub-layer scale for back-to-front (y-sort) drawing.
// Lower world_y = lower on screen = closer to viewer = higher z.
const Y_SORT_SCALE: f32 = 0.001;

// Spawn-density thresholds -- cumulative 0..100 ranges.
// Corner zone (cd <= 4): heavily weighted toward trees.
const CORNER_CD: u32 = 4;
// Edge zone (ed <= 5): trees thin out, bushes rise.
const EDGE_ED: u32 = 5;
// Middle zone (ed <= 7): dominated by bushes.
const MID_ED: u32 = 7;
// Centre (ed > 7, max ed on 32x18 map is 8): mostly flowers.

const TREE_ASSETS: [&str; 2] = ["sprites/scenery/trees/tree_oak.webp", "sprites/scenery/trees/tree_pine.webp"];
const BUSH_ASSETS: [&str; 2] = ["sprites/scenery/bushes/bush_round.webp", "sprites/scenery/bushes/bush_flower.webp"];
const FLOWER_ASSETS: [&str; 2] = ["sprites/scenery/flowers/flower_yellow.webp", "sprites/scenery/flowers/flower_purple.webp"];

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
) {
    spawn_area_scenery(commands, asset_server, area, area_pos);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Compute (tree, bush, flower) density thresholds for a tile position
/// based on area alignment and distance from edges/corners.
///
/// Interpolates between city (sparse), greenwood (balanced), and darkwood
/// (dense) profiles.
fn zone_thresholds(alignment: AreaAlignment, cd: u32, ed: u32) -> (usize, usize, usize) {
    // (tree, bush, flower) at city / greenwood / darkwood
    let (city, green, dark) = if cd <= CORNER_CD {
        ((15, 20, 25), (45, 55, 70), (92, 95, 97))
    } else if ed <= EDGE_ED {
        ((5, 10, 15), (15, 40, 60), (75, 85, 88))
    } else if ed <= MID_ED {
        ((0, 5, 10), (3, 30, 55), (55, 70, 75))
    } else {
        ((0, 3, 8), (0, 15, 50), (40, 55, 60))
    };

    lerp_thresholds(alignment, city, green, dark)
}

/// Linearly interpolate three anchor points at alignment 1, 50, 100.
fn lerp_thresholds(
    alignment: AreaAlignment,
    city: (usize, usize, usize),
    green: (usize, usize, usize),
    dark: (usize, usize, usize),
) -> (usize, usize, usize) {
    let a = f32::from(alignment.clamp(1, 100));

    let lerp = |lo: usize, hi: usize, t: f32| -> usize {
        #[allow(clippy::as_conversions)]
        let v = lo as f32 + (hi as f32 - lo as f32) * t;
        #[allow(clippy::as_conversions)]
        let r = v.round() as usize;
        r
    };

    if a <= 50.0 {
        let t = (a - 1.0) / 49.0;
        (
            lerp(city.0, green.0, t),
            lerp(city.1, green.1, t),
            lerp(city.2, green.2, t),
        )
    } else {
        let t = (a - 50.0) / 50.0;
        (
            lerp(green.0, dark.0, t),
            lerp(green.1, dark.1, t),
            lerp(green.2, dark.2, t),
        )
    }
}

fn spawn_area_scenery(
    commands: &mut Commands,
    asset_server: &AssetServer,
    area: &Area,
    area_pos: IVec2,
) {
    let tile_px = f32::from(TILE_SIZE_PX);
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;

    // Unique seed per area from grid position.
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax.wrapping_mul(2_654_435_761).wrapping_add(ay.wrapping_mul(1_013_904_223));

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let xu = u32::from(x);
            let yu = u32::from(y);

            if area.terrain_at(xu, yu) != Some(Terrain::Grass) {
                continue;
            }

            let hash = tile_hash(xu, yu, area_seed) % 100;
            let ed = edge_dist(x, y);
            let cd = corner_dist_min(x, y);

            let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
            let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;

            let (tree_t, bush_t, flower_t) = zone_thresholds(area.alignment, cd, ed);

            if cd <= CORNER_CD && hash < tree_t && !clear_for_tree(area, xu, yu) {
                // Corner fallback: tree blocked, try bush instead.
                if hash < bush_t {
                    let v = tile_hash(xu, yu, area_seed.wrapping_add(11)) % BUSH_ASSETS.len();
                    spawn_bush(commands, asset_server, BUSH_ASSETS[v], world_x, world_y);
                }
            } else {
                spawn_by_hash(hash, tree_t, bush_t, flower_t, xu, yu, world_x, world_y, area_seed, area, asset_server, commands);
            }
        }
    }
}

/// Returns `true` when a 2x2 tree at `(xu, yu)` will not visually overlap any path tile.
fn clear_for_tree(area: &Area, xu: u32, yu: u32) -> bool {
    let grass = Some(Terrain::Grass);
    let above = yu + 1 >= u32::from(MAP_HEIGHT)
        || area.terrain_at(xu, yu + 1) == grass;
    let left = xu == 0
        || area.terrain_at(xu - 1, yu) == grass;
    let right = xu + 1 >= u32::from(MAP_WIDTH)
        || area.terrain_at(xu + 1, yu) == grass;
    above && left && right
}

fn spawn_by_hash(
    hash: usize,
    tree_threshold: usize,
    bush_threshold: usize,
    flower_threshold: usize,
    xu: u32,
    yu: u32,
    world_x: f32,
    world_y: f32,
    area_seed: u32,
    area: &Area,
    asset_server: &AssetServer,
    commands: &mut Commands,
) {
    if hash < tree_threshold {
        if clear_for_tree(area, xu, yu) {
            let variant = tile_hash(xu, yu, area_seed.wrapping_add(10)) % TREE_ASSETS.len();
            spawn_tree(commands, asset_server, TREE_ASSETS[variant], world_x, world_y);
        }
    } else if hash < bush_threshold {
        let variant = tile_hash(xu, yu, area_seed.wrapping_add(11)) % BUSH_ASSETS.len();
        spawn_bush(commands, asset_server, BUSH_ASSETS[variant], world_x, world_y);
    } else if hash < flower_threshold {
        let variant = tile_hash(xu, yu, area_seed.wrapping_add(12)) % FLOWER_ASSETS.len();
        spawn_flower(commands, asset_server, FLOWER_ASSETS[variant], world_x, world_y);
    }
}

// ---------------------------------------------------------------------------
// Main area entity spawners (with collision and rustle)
// ---------------------------------------------------------------------------

fn spawn_tree(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::SceneryTree.z_f32() - world_y * Y_SORT_SCALE;
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

fn spawn_bush(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::SceneryBush.z_f32() - world_y * Y_SORT_SCALE;
    commands.spawn((
        Scenery,
        Rustleable,
        Sprite {
            image: asset_server.load(path),
            custom_size: Some(Vec2::splat(BUSH_SIZE_PX)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, z),
    ));
}

fn spawn_flower(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    world_x: f32,
    world_y: f32,
) {
    let z = Layer::SceneryFlower.z_f32() - world_y * Y_SORT_SCALE;
    commands.spawn((
        Scenery,
        Rustleable,
        Sprite {
            image: asset_server.load(path),
            custom_size: Some(Vec2::splat(FLOWER_SIZE_PX)),
            ..default()
        },
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

fn corner_dist_min(x: u16, y: u16) -> u32 {
    let xu = u32::from(x);
    let yu = u32::from(y);
    let right = u32::from(MAP_WIDTH).saturating_sub(1).saturating_sub(xu);
    let top = u32::from(MAP_HEIGHT).saturating_sub(1).saturating_sub(yu);
    (xu + yu).min(right + yu).min(xu + top).min(right + top)
}
