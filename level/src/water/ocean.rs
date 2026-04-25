//! Ocean + sand band generation. Edges of areas that face missing world
//! neighbours become ocean (and the strip just inland becomes sand).

use bevy::math::{IVec2, UVec2};

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

use super::tiles::{WaterKind, WaterMap};

/// Ocean band depth (tiles) along the outermost edge of an edge-facing area.
const OCEAN_DEPTH: u32 = 4;
/// Sand band depth (tiles) inland of the ocean band.
const SAND_DEPTH: u32 = 2;

pub(super) fn generate_ocean_and_sand(map: &mut WaterMap, world: &WorldMap) {
    let width = u32::from(MAP_WIDTH);
    let height = u32::from(MAP_HEIGHT);
    for pos in world.area_positions() {
        let missing = missing_neighbours(world, pos);
        if missing.is_empty() {
            continue;
        }
        for y in 0..height {
            for x in 0..width {
                let dist = edge_distance_to_missing(x, y, width, height, &missing);
                let Some(dist) = dist else {
                    continue;
                };
                let local = UVec2::new(x, y);
                let key = (pos, local);
                if dist < OCEAN_DEPTH {
                    // Ocean tiles overwrite anything except stepping stones.
                    if !map.stones.contains(&key) {
                        map.tiles.insert(key, WaterKind::Ocean);
                    }
                } else if dist < OCEAN_DEPTH + SAND_DEPTH && !map.tiles.contains_key(&key) {
                    map.sand.insert(key);
                }
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

/// Shortest distance (tiles) from `(x,y)` to the nearest edge that faces a
/// missing neighbour. Returns `None` when there's no such edge.
fn edge_distance_to_missing(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    missing: &[Direction],
) -> Option<u32> {
    missing
        .iter()
        .map(|dir| match dir {
            Direction::North => height.saturating_sub(1).saturating_sub(y),
            Direction::South => y,
            Direction::East => width.saturating_sub(1).saturating_sub(x),
            Direction::West => x,
        })
        .min()
}
