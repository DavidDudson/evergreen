//! Pier generation: place a 2-3 tile wide x 6-10 tile long rectangular run of
//! pier tiles extending from a dead-end coastal area's land outward into the
//! ocean. The pier overlays ocean tiles (still walkable for the player) and
//! its end-cap is validated to be surrounded by ocean -- no land tiles
//! adjacent to the outermost pier row.

use bevy::math::{IVec2, UVec2};

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

use super::tiles::WaterMap;

/// Pier length range (tiles, axial).
const PIER_LEN_MIN: i32 = 6;
const PIER_LEN_MAX: i32 = 10;
/// Width range (tiles, lateral).
const PIER_WIDTH_MIN: i32 = 2;
const PIER_WIDTH_MAX: i32 = 3;
/// How many of the leading pier tiles must rest on land/sand (the rest float
/// over ocean).
const PIER_LAND_TILES_MIN: i32 = 1;
const PIER_LAND_TILES_MAX: i32 = 2;

pub(super) fn generate_piers(map: &mut WaterMap, world: &WorldMap, rng: &mut u64) {
    if !world.has_ocean {
        return;
    }
    for pos in world.area_positions() {
        let Some(direction) = pier_direction_for_area(world, pos) else {
            continue;
        };
        *rng = super::generate::lcg(*rng);
        let length = sample_range(rng, PIER_LEN_MIN, PIER_LEN_MAX);
        *rng = super::generate::lcg(*rng);
        let width = sample_range(rng, PIER_WIDTH_MIN, PIER_WIDTH_MAX);
        *rng = super::generate::lcg(*rng);
        let land_tiles = sample_range(rng, PIER_LAND_TILES_MIN, PIER_LAND_TILES_MAX);
        carve_pier(map, world, pos, direction, length, width, land_tiles);
    }
}

/// Returns the direction the pier should extend (opposite the area's single
/// road exit), if this is a dead-end ocean-facing area.
fn pier_direction_for_area(world: &WorldMap, area_pos: IVec2) -> Option<Direction> {
    let area = world.get_area(area_pos)?;
    if area.exits.len() != 1 {
        return None;
    }
    let exit = *area.exits.iter().next()?;
    let missing = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ]
    .into_iter()
    .filter(|d| world.get_area(area_pos + d.grid_offset()).is_none())
    .count();
    if missing != 3 {
        return None;
    }
    Some(exit.opposite())
}

fn carve_pier(
    map: &mut WaterMap,
    world: &WorldMap,
    area_pos: IVec2,
    direction: Direction,
    length: i32,
    width: i32,
    land_tiles: i32,
) {
    let (axial_step, lateral_step) = direction_axes(direction);
    // Pick the start tile -- centred laterally, positioned axially so that
    // exactly `land_tiles` of the run sits on land/sand inland of the ocean.
    let Some((start_x, start_y)) = pier_start_tile(world, area_pos, direction, length, land_tiles)
    else {
        return;
    };

    // Build candidate footprint: `length` along axial, centred over `width`.
    let half_lo = width / 2;
    let half_hi = (width - 1) - half_lo;
    let mut footprint: Vec<(i32, i32)> = Vec::with_capacity(usize::try_from(length * width).unwrap_or(0));
    for axial in 0..length {
        for lat in -half_lo..=half_hi {
            let tx = start_x + axial_step.0 * axial + lateral_step.0 * lat;
            let ty = start_y + axial_step.1 * axial + lateral_step.1 * lat;
            footprint.push((tx, ty));
        }
    }

    // Validate end cap: forward + lateral neighbours of the outermost row must
    // not be land. Otherwise we'd have land beside the end of the pier.
    if !end_cap_surrounded_by_water(world, area_pos, direction, start_x, start_y, length, half_lo, half_hi) {
        return;
    }

    for (tx, ty) in footprint {
        if let Some(local) = local_in_area(tx, ty) {
            map.piers.insert((area_pos, local));
        }
    }
}

/// Returns axial step (the direction the pier extends) and lateral step
/// (perpendicular, used to widen the pier).
fn direction_axes(direction: Direction) -> ((i32, i32), (i32, i32)) {
    match direction {
        Direction::North => ((0, 1), (1, 0)),
        Direction::South => ((0, -1), (1, 0)),
        Direction::East => ((1, 0), (0, 1)),
        Direction::West => ((-1, 0), (0, 1)),
    }
}

/// Locate the pier's start tile. The start sits `land_tiles` axially inside
/// the land/sand band, so the first `land_tiles` of the run are on solid
/// ground and the remainder extend out over ocean.
fn pier_start_tile(
    world: &WorldMap,
    area_pos: IVec2,
    direction: Direction,
    _length: i32,
    land_tiles: i32,
) -> Option<(i32, i32)> {
    // Start at the area's center, scan axially toward `direction` until we
    // hit the first ocean tile, then back off `land_tiles` to begin on land.
    let cx = i32::from(MAP_WIDTH) / 2;
    let cy = i32::from(MAP_HEIGHT) / 2;
    let (axial_step, _) = direction_axes(direction);
    let mut tx = cx;
    let mut ty = cy;
    for _ in 0..i32::from(MAP_WIDTH).max(i32::from(MAP_HEIGHT)) {
        if is_ocean_tile(world, area_pos, tx, ty) {
            // Step back `land_tiles` so the first row of the pier lies on land.
            return Some((tx - axial_step.0 * land_tiles, ty - axial_step.1 * land_tiles));
        }
        tx += axial_step.0;
        ty += axial_step.1;
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn end_cap_surrounded_by_water(
    world: &WorldMap,
    area_pos: IVec2,
    direction: Direction,
    start_x: i32,
    start_y: i32,
    length: i32,
    half_lo: i32,
    half_hi: i32,
) -> bool {
    let (axial_step, lateral_step) = direction_axes(direction);
    let end_axial = length - 1;
    let end_x = start_x + axial_step.0 * end_axial;
    let end_y = start_y + axial_step.1 * end_axial;

    // Tile directly forward of the end cap.
    let fx = end_x + axial_step.0;
    let fy = end_y + axial_step.1;
    if !is_water_or_off_area(world, area_pos, fx, fy) {
        return false;
    }
    // Lateral neighbours along the entire end row.
    for lat in -(half_lo + 1)..=(half_hi + 1) {
        if (-half_lo..=half_hi).contains(&lat) {
            continue; // skip pier's own cells
        }
        let tx = end_x + lateral_step.0 * lat;
        let ty = end_y + lateral_step.1 * lat;
        if !is_water_or_off_area(world, area_pos, tx, ty) {
            return false;
        }
    }
    true
}

fn is_ocean_tile(world: &WorldMap, area_pos: IVec2, tx: i32, ty: i32) -> bool {
    let Some(local) = local_in_area(tx, ty) else {
        return false;
    };
    matches!(
        world.water.get(area_pos, local),
        Some(super::tiles::WaterKind::Ocean)
    )
}

/// Treat off-area tiles as water -- they're outside the area and beyond the
/// world edge in a 3-missing-neighbours area, so they don't represent land.
/// Inside the area, only ocean (or sand) tiles count as "water" for the
/// end-cap test; grass / dirt counts as land.
fn is_water_or_off_area(world: &WorldMap, area_pos: IVec2, tx: i32, ty: i32) -> bool {
    let Some(local) = local_in_area(tx, ty) else {
        return true;
    };
    matches!(
        world.water.get(area_pos, local),
        Some(super::tiles::WaterKind::Ocean)
    ) || world.water.has_sand(area_pos, local)
}

fn local_in_area(tx: i32, ty: i32) -> Option<UVec2> {
    if tx < 0 || ty < 0 {
        return None;
    }
    if tx >= i32::from(MAP_WIDTH) || ty >= i32::from(MAP_HEIGHT) {
        return None;
    }
    Some(UVec2::new(
        u32::try_from(tx).ok()?,
        u32::try_from(ty).ok()?,
    ))
}

fn sample_range(rng: &mut u64, lo: i32, hi: i32) -> i32 {
    if hi <= lo {
        return lo;
    }
    let span = u64::try_from(hi - lo + 1).unwrap_or(1);
    let r = (*rng % span).try_into().unwrap_or(0);
    lo + r
}
