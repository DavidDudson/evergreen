use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::layer::Layer;
use models::scenery::{Rustleable, Rustling, Scenery, SceneryCollider};

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::TILE_SIZE_PX;
use crate::terrain::{tile_hash, Terrain};
use crate::world::{AreaChanged, WorldMap};

// Trees are 2×2 tiles (32×32 px), anchored at BOTTOM_CENTER of the base tile.
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

// Spawn-density thresholds — cumulative 0..100 ranges.
// Corner zone (cd ≤ 4): heavily weighted toward trees.
const CORNER_CD: u32 = 4;
// Edge zone (ed ≤ 5): trees thin out, bushes rise.
const EDGE_ED: u32 = 5;
// Middle zone (ed ≤ 7): dominated by bushes.
const MID_ED: u32 = 7;
// Centre (ed > 7, max ed on 32×18 map is 8): mostly flowers.

const TREE_ASSETS: [&str; 2] = ["tree_oak.png", "tree_pine.png"];
const BUSH_ASSETS: [&str; 2] = ["bush_round.png", "bush_flower.png"];
const FLOWER_ASSETS: [&str; 2] = ["flower_yellow.png", "flower_purple.png"];

pub fn spawn_scenery(mut commands: Commands, asset_server: Res<AssetServer>, world: Res<WorldMap>) {
    spawn_area_scenery(&mut commands, &asset_server, &world);
}

pub fn despawn_scenery(mut commands: Commands, query: Query<Entity, With<Scenery>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn respawn_scenery_on_area_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldMap>,
    query: Query<Entity, With<Scenery>>,
    mut events: MessageReader<AreaChanged>,
) {
    if events.read().next().is_none() {
        return;
    }
    for entity in &query {
        commands.entity(entity).despawn();
    }
    spawn_area_scenery(&mut commands, &asset_server, &world);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn spawn_area_scenery(commands: &mut Commands, asset_server: &AssetServer, world: &WorldMap) {
    let tile_px = f32::from(TILE_SIZE_PX);
    let offset_x = -(f32::from(MAP_WIDTH) * tile_px) / 2.0;
    let offset_y = -(f32::from(MAP_HEIGHT) * tile_px) / 2.0;

    // Unique seed per area from grid position (bit-reinterpret pattern from WorldMap::area_seed).
    let ax = u32::from_ne_bytes(world.current.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(world.current.y.to_ne_bytes());
    let area_seed = ax.wrapping_mul(2_654_435_761).wrapping_add(ay.wrapping_mul(1_013_904_223));

    let area = world.current_area();

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

            let world_x = offset_x + f32::from(x) * tile_px + tile_px / 2.0;
            let world_y = offset_y + f32::from(y) * tile_px + tile_px / 2.0;

            // Corner zone: aggressive trees.
            if cd <= CORNER_CD {
                let is_tree = hash < 65;
                if is_tree && !clear_for_tree(area, xu, yu) {
                    // Fall back to a bush rather than skipping entirely.
                    if hash < 70 {
                        let v = tile_hash(xu, yu, area_seed.wrapping_add(11)) % BUSH_ASSETS.len();
                        spawn_bush(commands, asset_server, BUSH_ASSETS[v], world_x, world_y);
                    }
                } else {
                    spawn_by_hash(hash, 65, 75, 80, xu, yu, world_x, world_y, area_seed, area, asset_server, commands);
                }
            // Edge zone: trees with bushes.
            } else if ed <= EDGE_ED {
                spawn_by_hash(hash, 25, 55, 65, xu, yu, world_x, world_y, area_seed, area, asset_server, commands);
            // Middle zone: bushes dominant.
            } else if ed <= MID_ED {
                spawn_by_hash(hash, 3, 48, 68, xu, yu, world_x, world_y, area_seed, area, asset_server, commands);
            // Centre: flowers dominant.
            } else {
                spawn_by_hash(hash, 0, 15, 55, xu, yu, world_x, world_y, area_seed, area, asset_server, commands);
            }
        }
    }
}

/// Returns `true` when a 2×2 tree at `(xu, yu)` will not visually overlap any path tile.
///
/// The 32×32 sprite (BOTTOM_CENTER at tile centre) covers:
/// - Horizontally: 8 px into tiles x-1 and x+1
/// - Vertically: from tile y centre up through tile y+1 entirely
///
/// Only checks tiles that are inside map bounds; border overflow is harmless.
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

/// Tree: 2×2 tiles (32×32 px), BOTTOM_CENTER so it grows upward from the tile centre.
/// Full-sprite collider offset to the sprite's visual centre.
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
