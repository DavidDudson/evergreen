//! Beaches, piers, and crab fauna that live around ocean tiles.
//!
//! Beaches: sand tiles stored on `WaterMap::sand` render as simple decorative
//! Sprites, no collider (player walks on sand).
//!
//! Piers: a dead-end area surrounded by ocean on 3 of its 4 cardinal sides
//! gets a row of wooden planks extending from the area's centre toward the
//! ocean, capped with a pier-post bollard at the outermost plank.
//!
//! Crabs: per-tile chance of spawning on a sand tile; scuttle sideways via a
//! sine phase.

use bevy::math::{IVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;
use models::scenery::Scenery;

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::tile_hash;
use crate::water::WaterKind;
use crate::world::WorldMap;

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

const PIER_PLANK_SPRITE: &str = "sprites/scenery/ponds/pier_plank.webp";
const PIER_POST_SPRITE: &str = "sprites/scenery/ponds/pier_post.webp";
const CRAB_SPRITE: &str = "sprites/creatures/water/crab.webp";

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

const PLANK_SIZE_PX: f32 = 20.0;
const CRAB_SIZE_PX: f32 = 10.0;

/// Per sand-tile chance (out of 100) to spawn a crab.
const CRAB_CHANCE: u32 = 20;

/// Crab horizontal scuttle amplitude + frequency.
const CRAB_SCUTTLE_AMPLITUDE_PX: f32 = 2.5;
const CRAB_SCUTTLE_FREQ_HZ: f32 = 1.1;

/// Pixel dimensions of one map area.
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct SandTile;

#[derive(Component)]
pub struct PierPiece;

#[derive(Component)]
pub struct Crab {
    pub phase: f32,
    pub base_x: f32,
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn spawn_area_beach(
    commands: &mut Commands,
    asset_server: &AssetServer,
    wang: &crate::wang::WangTilesets,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);

    // Sand tiles (wang-tiled) + crabs.
    let ax = u32::from_ne_bytes(area_pos.x.to_ne_bytes());
    let ay = u32::from_ne_bytes(area_pos.y.to_ne_bytes());
    let area_seed = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(1_013_904_223))
        .wrapping_add(0xBE_AC_30);

    let sand_tileset = &wang.sand_grass;
    for y in 0..u32::from(crate::area::MAP_HEIGHT) {
        for x in 0..u32::from(crate::area::MAP_WIDTH) {
            let mask = crate::water::sand_mask(&world.water, area_pos, x, y);
            if mask == 0 {
                continue;
            }
            let local = bevy::math::UVec2::new(x, y);
            let world_x = base_offset_x
                + f32::from(u16::try_from(x).unwrap_or(0)) * tile_px
                + tile_px / 2.0;
            let world_y = base_offset_y
                + f32::from(u16::try_from(y).unwrap_or(0)) * tile_px
                + tile_px / 2.0;
            commands.spawn((
                SandTile,
                Scenery,
                Sprite {
                    image: sand_tileset.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: sand_tileset.layout.clone(),
                        index: sand_tileset.lut[usize::from(mask)],
                    }),
                    custom_size: Some(Vec2::splat(tile_px)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.3),
            ));

            // Crabs only spawn on fully-sand tiles (mask = 0b1111).
            if mask == 0b1111 && world.water.has_sand(area_pos, local) {
                let hash = tile_hash(x, y, area_seed);
                #[allow(clippy::as_conversions)]
                let crab_roll = u32::try_from(hash % 100).unwrap_or(0);
                if crab_roll < CRAB_CHANCE {
                    #[allow(clippy::as_conversions)]
                    let phase = (hash.wrapping_mul(31) % 628) as f32 / 100.0;
                    commands.spawn((
                        SandTile,
                        Crab {
                            phase,
                            base_x: world_x,
                        },
                        Sprite {
                            image: asset_server.load(CRAB_SPRITE),
                            custom_size: Some(Vec2::splat(CRAB_SIZE_PX)),
                            ..default()
                        },
                        Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.8),
                    ));
                }
            }
        }
    }

    // Piers.
    if let Some(pier_dir) = pier_direction_for_area(world, area_pos) {
        spawn_pier(
            commands,
            asset_server,
            pier_dir,
            base_offset_x,
            base_offset_y,
            tile_px,
        );
    }
}

// ---------------------------------------------------------------------------
// Pier detection + placement
// ---------------------------------------------------------------------------

/// If the area is a dead-end (exactly one road exit) whose other three sides
/// are all world-edge-facing ocean, return the direction the pier should point
/// (opposite the exit).
fn pier_direction_for_area(world: &WorldMap, area_pos: IVec2) -> Option<Direction> {
    let area = world.get_area(area_pos)?;
    if area.exits.len() != 1 {
        return None;
    }
    let exit = *area.exits.iter().next()?;
    if !world.has_ocean {
        return None;
    }
    let missing: Vec<Direction> = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ]
    .into_iter()
    .filter(|d| world.get_area(area_pos + d.grid_offset()).is_none())
    .collect();
    if missing.len() != 3 {
        return None;
    }
    // Pier points in the cardinal opposite to the exit (out into the ocean).
    Some(exit.opposite())
}

fn spawn_pier(
    commands: &mut Commands,
    asset_server: &AssetServer,
    direction: Direction,
    base_offset_x: f32,
    base_offset_y: f32,
    tile_px: f32,
) {
    let mid_x = f32::from(MAP_WIDTH) * 0.5;
    let mid_y = f32::from(MAP_HEIGHT) * 0.5;

    let (start, step, planks): ((f32, f32), (f32, f32), u16) = match direction {
        Direction::North => ((mid_x, mid_y), (0.0, 1.0), 8),
        Direction::South => ((mid_x, mid_y), (0.0, -1.0), 8),
        Direction::East => ((mid_x, mid_y), (1.0, 0.0), 12),
        Direction::West => ((mid_x, mid_y), (-1.0, 0.0), 12),
    };

    for i in 0..planks {
        let tx = start.0 + step.0 * f32::from(i);
        let ty = start.1 + step.1 * f32::from(i);
        let world_x = base_offset_x + tx * tile_px;
        let world_y = base_offset_y + ty * tile_px;
        let path = if i + 1 == planks {
            PIER_POST_SPRITE
        } else {
            PIER_PLANK_SPRITE
        };
        commands.spawn((
            PierPiece,
            Scenery,
            Sprite {
                image: asset_server.load(path),
                custom_size: Some(Vec2::splat(PLANK_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.9),
        ));
    }
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

pub fn animate_crabs(time: Res<Time>, mut crabs: Query<(&Crab, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (crab, mut tf) in &mut crabs {
        tf.translation.x = crab.base_x
            + (t * CRAB_SCUTTLE_FREQ_HZ + crab.phase).sin() * CRAB_SCUTTLE_AMPLITUDE_PX;
    }
}

// ---------------------------------------------------------------------------
// Teardown
// ---------------------------------------------------------------------------

pub fn despawn_sand(mut commands: Commands, q: Query<Entity, With<SandTile>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn despawn_piers(mut commands: Commands, q: Query<Entity, With<PierPiece>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

// Suppress dead-code warnings on the ocean-only WaterKind imported to keep
// compile-time confidence across future refactors (the variant stays used
// via pattern matches elsewhere, but not referenced directly in this file).
const _: WaterKind = WaterKind::Ocean;
