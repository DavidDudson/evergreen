use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use models::layer::Layer;
use models::reveal::{RevealState, Revealable};
use models::scenery::{Rustling, Scenery, SceneryCollider};
use models::shadow::{TREE_SHADOW_HALF_PX, TREE_SHADOW_OFFSET_Y_PX};

use crate::area::{Area, MAP_HEIGHT, MAP_WIDTH};
use crate::biome_registry::BiomeRegistry;
use crate::blending;
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

/// Salt offset added to the per-area seed so blend-pool picks don't collide
/// with the variant pick.
const POOL_PICK_SALT: u32 = 20;
/// Salt offset added to the per-area seed for the variant pick.
const VARIANT_PICK_SALT: u32 = 10;

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
    registry: &BiomeRegistry,
    area: &Area,
    area_pos: IVec2,
    world: &WorldMap,
) {
    spawn_area_scenery(
        commands,
        asset_server,
        shadow_assets,
        registry,
        area,
        area_pos,
        world,
    );
}

/// Tree footprint covers the trunk tile plus the two horizontally adjacent
/// tiles (sprite is 3 tiles wide with BOTTOM_CENTER anchor). All three must
/// be grass with no water / sand so the canopy never overhangs paths or
/// pond shores.
fn clear_for_tree(_area: &Area, world: &WorldMap, area_pos: IVec2, xu: u32, yu: u32) -> bool {
    for dx in [-1_i32, 0, 1] {
        let lx = i32::try_from(xu).unwrap_or(0) + dx;
        let ly = i32::try_from(yu).unwrap_or(0);
        if !matches!(
            world.terrain_at_extended(area_pos, lx, ly),
            Some(Terrain::Grass)
        ) {
            return false;
        }
        let Ok(ux) = u32::try_from(lx) else {
            return false;
        };
        let local = bevy::math::UVec2::new(ux, yu);
        let key_pos = if (0..u32::from(MAP_WIDTH)).contains(&ux) {
            area_pos
        } else if lx < 0 {
            area_pos + bevy::math::IVec2::new(-1, 0)
        } else {
            area_pos + bevy::math::IVec2::new(1, 0)
        };
        let key_local = if (0..u32::from(MAP_WIDTH)).contains(&ux) {
            local
        } else if lx < 0 {
            bevy::math::UVec2::new(u32::from(MAP_WIDTH) - 1, yu)
        } else {
            bevy::math::UVec2::new(0, yu)
        };
        if world.water.get(key_pos, key_local).is_some()
            || world.water.has_sand(key_pos, key_local)
        {
            return false;
        }
    }
    true
}

fn spawn_area_scenery(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shadow_assets: &DropShadowAssets,
    registry: &BiomeRegistry,
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
            let threshold = registry.tree_config(blend.alignment).threshold_at(ed);

            if hash < threshold && clear_for_tree(area, world, area_pos, xu, yu) {
                // In the blend zone, probabilistically pick from the neighbor's
                // tree pool based on blend factor (Minecraft-style mixing).
                let mix_hash = tile_hash(xu, yu, area_seed.wrapping_add(POOL_PICK_SALT)) % 100;
                #[allow(clippy::as_conversions)]
                let mix_threshold = (blend.factor * 100.0) as usize;
                let pool_alignment = if mix_hash < mix_threshold {
                    blend.neighbor_alignment.unwrap_or(area.alignment)
                } else {
                    area.alignment
                };
                let pool = registry.trees(pool_alignment);
                let variant =
                    tile_hash(xu, yu, area_seed.wrapping_add(VARIANT_PICK_SALT)) % pool.len();
                let def = pool[variant];
                let world_x = base_offset_x + f32::from(x) * tile_px + tile_px / 2.0;
                let world_y = base_offset_y + f32::from(y) * tile_px + tile_px / 2.0;
                spawn_tree(commands, asset_server, shadow_assets, def, world_x, world_y);
            }
        }
    }
}

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

    spawn_drop_shadow(
        commands,
        shadow_assets,
        parent,
        TREE_SHADOW_HALF_PX,
        TREE_SHADOW_OFFSET_Y_PX,
    );
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
