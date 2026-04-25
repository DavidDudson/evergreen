//! River generation: scans every area for an N/S or E/W axis (perpendicular
//! to its road exits) and carves a strip of water tiles, with stepping stones
//! at the central crossing.
//!
//! Each river picks a [`RiverProfile`] which controls its width and per-tile
//! depth distribution. Profiles include narrow shallow streams, wide rivers
//! with a deep center, deep-only channels, and mixed shallow/deep pockets.

use bevy::math::{IVec2, UVec2};

use crate::area::{Area, Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

use super::depth::WaterDepth;
use super::tiles::{WaterKind, WaterMap};

/// Per-area chance (out of 100) to add a river when the axis is valid.
const RIVER_CHANCE: u64 = 45;

/// Central column / row of every river -- the crossing tile.
const RIVER_CENTER_X: u32 = 15;
const RIVER_CENTER_Y: u32 = 8;

/// Central 3x3 block where road crosses river -- stones go here.
const CROSSING_HALF: u32 = 1; // i.e. center +/- 1 tile

/// River flow axis. Always perpendicular to the area's road exits.
#[derive(Debug, Clone, Copy)]
enum RiverAxis {
    NorthSouth,
    EastWest,
}

/// Width + depth distribution for a river. Picked once per river.
#[derive(Debug, Clone, Copy)]
enum RiverProfile {
    /// 1 tile wide, all shallow.
    Shallow1,
    /// 2 tiles wide, all shallow.
    Shallow2,
    /// 3 tiles wide: edges shallow, middle deep.
    DeepCenter3,
    /// 5 tiles wide: edges shallow, inner 3 deep.
    DeepCenter5,
    /// 3 tiles wide, all deep.
    DeepOnly3,
    /// 3 tiles wide, random shallow/deep pockets per tile.
    MixedPockets3,
}

impl RiverProfile {
    fn width(self) -> u32 {
        match self {
            Self::Shallow1 => 1,
            Self::Shallow2 => 2,
            Self::DeepCenter3 | Self::DeepOnly3 | Self::MixedPockets3 => 3,
            Self::DeepCenter5 => 5,
        }
    }

    /// Depth for a tile at lateral offset `lateral` from the river's center.
    /// `lateral` is signed (negative = port, positive = starboard).
    fn depth_at(self, lateral: i32, hash: u32) -> WaterDepth {
        let abs = lateral.unsigned_abs();
        match self {
            Self::Shallow1 | Self::Shallow2 => WaterDepth::Shallow,
            Self::DeepOnly3 => WaterDepth::Deep,
            Self::DeepCenter3 => {
                if abs == 0 {
                    WaterDepth::Deep
                } else {
                    WaterDepth::Shallow
                }
            }
            Self::DeepCenter5 => {
                if abs <= 1 {
                    WaterDepth::Deep
                } else {
                    WaterDepth::Shallow
                }
            }
            Self::MixedPockets3 => {
                if hash.is_multiple_of(3) {
                    WaterDepth::Deep
                } else {
                    WaterDepth::Shallow
                }
            }
        }
    }
}

fn pick_profile(rng: &mut u64) -> RiverProfile {
    *rng = super::generate::lcg(*rng);
    match *rng % 6 {
        0 => RiverProfile::Shallow1,
        1 => RiverProfile::Shallow2,
        2 => RiverProfile::DeepCenter3,
        3 => RiverProfile::DeepCenter5,
        4 => RiverProfile::DeepOnly3,
        _ => RiverProfile::MixedPockets3,
    }
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
        let profile = pick_profile(rng);
        carve_river(map, world, pos, axis, profile);
    }
}

fn carve_river(
    map: &mut WaterMap,
    world: &WorldMap,
    pos: IVec2,
    axis: RiverAxis,
    profile: RiverProfile,
) {
    let waterfall_at_north = matches!(axis, RiverAxis::NorthSouth)
        && world
            .get_area(pos + Direction::North.grid_offset())
            .is_none();
    let width = profile.width();
    let half_lo = i32::try_from(width).unwrap_or(1) / 2;
    let half_hi = i32::try_from(width.saturating_sub(1)).unwrap_or(0) - half_lo;

    let lateral_range = -half_lo..=half_hi;
    let waterfall_start_row = u32::from(MAP_HEIGHT) * 4 / 5;

    match axis {
        RiverAxis::NorthSouth => {
            for y in 0..u32::from(MAP_HEIGHT) {
                for lat in lateral_range.clone() {
                    let Some(x) = signed_offset(RIVER_CENTER_X, lat, MAP_WIDTH) else {
                        continue;
                    };
                    let kind = if waterfall_at_north && y >= waterfall_start_row {
                        WaterKind::Waterfall
                    } else {
                        WaterKind::RiverNS
                    };
                    place_river_tile(map, pos, x, y, lat, kind, profile);
                }
            }
        }
        RiverAxis::EastWest => {
            for x in 0..u32::from(MAP_WIDTH) {
                for lat in lateral_range.clone() {
                    let Some(y) = signed_offset(RIVER_CENTER_Y, lat, MAP_HEIGHT) else {
                        continue;
                    };
                    place_river_tile(map, pos, x, y, lat, WaterKind::RiverEW, profile);
                }
            }
        }
    }
}

fn signed_offset(center: u32, offset: i32, max: u16) -> Option<u32> {
    let v = i32::try_from(center).ok()? + offset;
    if v < 0 || v >= i32::from(max) {
        None
    } else {
        u32::try_from(v).ok()
    }
}

#[allow(clippy::too_many_arguments)]
fn place_river_tile(
    map: &mut WaterMap,
    pos: IVec2,
    x: u32,
    y: u32,
    lateral: i32,
    kind: WaterKind,
    profile: RiverProfile,
) {
    let key = (pos, UVec2::new(x, y));
    if map.tiles.contains_key(&key) {
        return;
    }
    map.tiles.insert(key, kind);
    let depth = if matches!(kind, WaterKind::Waterfall) {
        WaterDepth::Deep
    } else {
        let h = pocket_hash(pos, x, y);
        profile.depth_at(lateral, h)
    };
    map.depths.insert(key, depth);

    // Stepping stones at the central 3x3 crossing for non-waterfall river tiles.
    let lat_abs = lateral.unsigned_abs();
    let in_crossing_lateral = lat_abs <= CROSSING_HALF;
    let in_crossing_axial = match kind {
        WaterKind::RiverNS => (RIVER_CENTER_Y.saturating_sub(CROSSING_HALF)
            ..=RIVER_CENTER_Y + CROSSING_HALF)
            .contains(&y),
        WaterKind::RiverEW => (RIVER_CENTER_X.saturating_sub(CROSSING_HALF)
            ..=RIVER_CENTER_X + CROSSING_HALF)
            .contains(&x),
        _ => false,
    };
    if in_crossing_lateral && in_crossing_axial && !matches!(kind, WaterKind::Waterfall) {
        map.stones.insert(key);
    }
}

fn pocket_hash(area: IVec2, x: u32, y: u32) -> u32 {
    #[allow(clippy::as_conversions)] // i32 bit-pattern reuse for hashing
    let ax = area.x as u32;
    #[allow(clippy::as_conversions)] // i32 bit-pattern reuse for hashing
    let ay = area.y as u32;
    ax.wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(40_503))
        .wrapping_add(x.wrapping_mul(73_856_093))
        .wrapping_add(y.wrapping_mul(19_349_663))
}
