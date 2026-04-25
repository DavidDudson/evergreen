//! Ocean + sand band generation. Edges of areas that face missing world
//! neighbours become ocean (and the strip just inland becomes sand). Areas
//! beyond the world edge -- the off-map neighbours of any ocean-facing area
//! -- are filled completely with ocean tiles so the world melts into open
//! water rather than reverting to dense forest.

use std::collections::HashSet;

use bevy::math::{IVec2, UVec2};

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

use super::tiles::{WaterKind, WaterMap};

/// Ocean band depth (tiles) along the outermost edge of an edge-facing area.
const OCEAN_DEPTH: i32 = 4;
/// Sand band depth (tiles) inland of the ocean band.
const SAND_DEPTH: i32 = 2;
/// How far the ocean / sand band thresholds may jitter from their nominal
/// value (in tiles) to give an organic, wavy shoreline. Sampled per
/// shoreline-tangent coordinate so the boundary varies along the shore but
/// remains continuous across area seams.
const SHORE_JITTER_TILES: i32 = 2;

pub(super) fn generate_ocean_and_sand(map: &mut WaterMap, world: &WorldMap) {
    let width = u32::from(MAP_WIDTH);
    let height = u32::from(MAP_HEIGHT);
    let mut off_map_ocean: HashSet<IVec2> = HashSet::new();
    for pos in world.area_positions() {
        let missing = missing_neighbours(world, pos);
        if missing.is_empty() {
            continue;
        }
        for &dir in &missing {
            off_map_ocean.insert(pos + dir.grid_offset());
        }
        for y in 0..height {
            for x in 0..width {
                let Some((dist, dir)) = closest_missing(x, y, width, height, &missing) else {
                    continue;
                };
                // Jitter ocean and sand thresholds along the shoreline tangent
                // (so neighbouring tiles share noise) using world-absolute
                // coords, keeping the boundary continuous across area seams.
                let tangent = shoreline_tangent(pos, x, y, dir);
                let ocean_thresh = OCEAN_DEPTH + jitter(tangent, 0xA1);
                let sand_thresh = ocean_thresh + SAND_DEPTH + jitter(tangent, 0xB7);
                let dist_i = i32::try_from(dist).unwrap_or(i32::MAX);
                let local = UVec2::new(x, y);
                let key = (pos, local);
                if dist_i < ocean_thresh {
                    if !map.stones.contains(&key) {
                        map.tiles.insert(key, WaterKind::Ocean);
                    }
                } else if dist_i < sand_thresh && !map.tiles.contains_key(&key) {
                    map.sand.insert(key);
                }
            }
        }
    }

    // Off-map ocean areas: fill every tile with ocean (forced Deep) so the
    // world melts into open water rather than dense-forest placeholder.
    for off_pos in off_map_ocean {
        for y in 0..height {
            for x in 0..width {
                let key = (off_pos, UVec2::new(x, y));
                map.tiles.entry(key).or_insert(WaterKind::Ocean);
                map.depths
                    .insert(key, super::depth::WaterDepth::Deep);
            }
        }
    }
}

/// Directions (from a given area) whose neighbour area does not exist --
/// those become world-edge facing and are where ocean spills outward.
fn missing_neighbours(world: &WorldMap, pos: IVec2) -> Vec<Direction> {
    [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ]
    .into_iter()
    .filter(|d| world.get_area(pos + d.grid_offset()).is_none())
    .collect()
}

/// Shortest distance (tiles) from `(x,y)` to any missing-neighbour edge,
/// along with the direction of that closest edge. The direction lets the
/// caller compute a shoreline-tangent coordinate for noise sampling.
fn closest_missing(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    missing: &[Direction],
) -> Option<(u32, Direction)> {
    missing
        .iter()
        .map(|dir| {
            let d = match dir {
                Direction::North => height.saturating_sub(1).saturating_sub(y),
                Direction::South => y,
                Direction::East => width.saturating_sub(1).saturating_sub(x),
                Direction::West => x,
            };
            (d, *dir)
        })
        .min_by_key(|(d, _)| *d)
}

/// World-absolute coordinate along the shoreline tangent. For a north-facing
/// missing edge the tangent runs east-west, so the tangent is the absolute
/// world-x. East/west missing edges use absolute world-y. This makes the
/// noise pattern continuous across area seams and yields a wavy shoreline
/// rather than an aligned-per-area one.
fn shoreline_tangent(area_pos: IVec2, x: u32, y: u32, dir: Direction) -> i32 {
    let map_w = i32::from(MAP_WIDTH);
    let map_h = i32::from(MAP_HEIGHT);
    let lx = i32::try_from(x).unwrap_or(0);
    let ly = i32::try_from(y).unwrap_or(0);
    match dir {
        Direction::North | Direction::South => area_pos.x * map_w + lx,
        Direction::East | Direction::West => area_pos.y * map_h + ly,
    }
}

/// Deterministic noise in `[-SHORE_JITTER_TILES, +SHORE_JITTER_TILES]`.
/// `salt` lets independent boundaries (ocean vs sand) jitter on
/// uncorrelated patterns.
fn jitter(tangent: i32, salt: u32) -> i32 {
    #[allow(clippy::as_conversions)] // i32 bit-pattern reuse for hashing
    let t = tangent as u32;
    let mut h = t.wrapping_mul(2_654_435_761).wrapping_add(salt);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    let modulo = u32::try_from(SHORE_JITTER_TILES * 2 + 1).unwrap_or(1);
    let r = h % modulo;
    i32::try_from(r).unwrap_or(0) - SHORE_JITTER_TILES
}
