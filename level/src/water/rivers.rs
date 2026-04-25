//! River generation: scans every area for an N/S or E/W axis (perpendicular
//! to its road exits) and carves a strip of water tiles, with stepping stones
//! at the central crossing.

use bevy::math::{IVec2, UVec2};

use crate::area::{Area, Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

use super::tiles::{WaterKind, WaterMap};

/// Per-area chance (out of 100) to add a river when the axis is valid.
const RIVER_CHANCE: u64 = 45;

/// Central 3x3 block where road crosses river -- stones go here.
const CROSSING_COL_START: u32 = 14;
const CROSSING_COL_END: u32 = 16;
const CROSSING_ROW_START: u32 = 7;
const CROSSING_ROW_END: u32 = 9;

/// River flow axis. Always perpendicular to the area's road exits.
#[derive(Debug, Clone, Copy)]
enum RiverAxis {
    NorthSouth,
    EastWest,
}

fn river_axis_for(area: &Area) -> Option<RiverAxis> {
    let n = area.exits.contains(&Direction::North);
    let s = area.exits.contains(&Direction::South);
    let e = area.exits.contains(&Direction::East);
    let w = area.exits.contains(&Direction::West);
    match (n, s, e, w) {
        // Road runs N/S -> river flows perpendicular E/W.
        (true, true, false, false) => Some(RiverAxis::EastWest),
        // Road runs E/W -> river flows perpendicular N/S.
        (false, false, true, true) => Some(RiverAxis::NorthSouth),
        _ => None,
    }
}

pub(super) fn generate_rivers(map: &mut WaterMap, world: &WorldMap, rng: &mut u64) {
    for pos in world.area_positions() {
        let Some(area) = world.get_area(pos) else {
            continue;
        };
        let Some(axis) = river_axis_for(area) else {
            continue;
        };
        *rng = super::generate::lcg(*rng);
        if *rng % 100 >= RIVER_CHANCE {
            continue;
        }
        carve_river(map, world, pos, axis);
    }
}

fn carve_river(map: &mut WaterMap, world: &WorldMap, pos: IVec2, axis: RiverAxis) {
    let waterfall_at_north = matches!(axis, RiverAxis::NorthSouth)
        && world
            .get_area(pos + Direction::North.grid_offset())
            .is_none();

    let iter: Vec<(u32, u32)> = match axis {
        RiverAxis::NorthSouth => (0..u32::from(MAP_HEIGHT))
            .flat_map(|y| (CROSSING_COL_START..=CROSSING_COL_END).map(move |x| (x, y)))
            .collect(),
        RiverAxis::EastWest => (0..u32::from(MAP_WIDTH))
            .flat_map(|x| (CROSSING_ROW_START..=CROSSING_ROW_END).map(move |y| (x, y)))
            .collect(),
    };

    for (x, y) in iter {
        let key = (pos, UVec2::new(x, y));
        // Skip if there's already a pond/lake here (river joins body naturally).
        if map.tiles.contains_key(&key) {
            continue;
        }
        // Top ~20% of the area becomes waterfall when the area faces the
        // north world edge (no neighbour beyond).
        let waterfall_start_row = u32::from(MAP_HEIGHT) * 4 / 5;
        let kind = if waterfall_at_north && y >= waterfall_start_row {
            WaterKind::Waterfall
        } else {
            match axis {
                RiverAxis::NorthSouth => WaterKind::RiverNS,
                RiverAxis::EastWest => WaterKind::RiverEW,
            }
        };
        map.tiles.insert(key, kind);

        // Central 3x3 crossing gets stepping stones (walkable).
        if (CROSSING_COL_START..=CROSSING_COL_END).contains(&x)
            && (CROSSING_ROW_START..=CROSSING_ROW_END).contains(&y)
            && !matches!(kind, WaterKind::Waterfall)
        {
            map.stones.insert(key);
        }
    }
}
