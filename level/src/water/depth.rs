//! Water depth classification.
//!
//! Each water tile has a [`WaterDepth`] (`Shallow` or `Deep`) computed after
//! the base water layout is generated. Depth drives:
//!   - whether the player can walk on the tile (shallow = walkable),
//!   - which fauna spawn (crabs vs fish),
//!   - which wang tileset / blend is used.
//!
//! Classification rules (per [`WaterKind`]):
//!   - `Plain`            -> all shallow.
//!   - `HotSpring`        -> all deep (dangerous; player blocked).
//!   - `Lake`             -> wavy shallow band along edges, deep interior.
//!   - `RiverNS`/`RiverEW` -> assigned at carve time via [`RiverDepthMode`].
//!   - `Waterfall`        -> all deep.
//!   - `Ocean`            -> wavy shallow band hugging shoreline; deep beyond.
//!
//! Wavy boundaries come from a deterministic per-tile hash: the threshold for
//! "still shallow" is jittered by +/- 1 tile so the boundary line meanders.

use bevy::math::{IVec2, UVec2};

use super::tiles::{neighbour_key, WaterKind, WaterMap, TILE_NEIGHBOURS_4};

/// Per-tile water depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaterDepth {
    Shallow,
    Deep,
}

/// Base shallow band thickness for ocean / lake (in tiles).
const SHALLOW_BAND_TILES: i32 = 2;
/// Per-tile jitter applied to the shallow threshold so the boundary is wavy.
const SHALLOW_JITTER_TILES: i32 = 1;

/// Compute depth for every tile in `map`. Existing `depths` are overwritten;
/// tiles whose `WaterKind` mandates a single depth (e.g. `HotSpring`) ignore
/// any pre-set value. Per-river depth assignments inserted during carving are
/// preserved (river tiles are skipped here).
pub fn classify_depths(map: &mut WaterMap) {
    let keys: Vec<(IVec2, UVec2, WaterKind)> = map
        .tiles
        .iter()
        .map(|(&(a, l), &k)| (a, l, k))
        .collect();
    for (area, local, kind) in keys {
        // Rivers + waterfall already have their depth assigned during carving.
        if kind.is_river() {
            continue;
        }
        // Tiles with a pre-assigned depth (e.g. off-map ocean forced Deep)
        // are left alone.
        if map.depths.contains_key(&(area, local)) {
            continue;
        }
        let depth = match kind {
            WaterKind::Plain => WaterDepth::Shallow,
            WaterKind::HotSpring | WaterKind::Waterfall => WaterDepth::Deep,
            WaterKind::Lake | WaterKind::Ocean => classify_by_distance(map, area, local, kind),
            // River variants handled by `is_river()` early-return above.
            WaterKind::RiverNS | WaterKind::RiverEW => continue,
        };
        map.depths.insert((area, local), depth);
    }
}

/// Walk outward from `(area, local)` in a BFS, returning manhattan-distance
/// to the nearest non-matching-kind tile (i.e. the shore for `Ocean`, or any
/// non-lake tile for `Lake`). Caps at `SHALLOW_BAND_TILES + 2` -- we don't
/// care about exact distance once we're well past the shallow band.
fn distance_to_non_matching(
    map: &WaterMap,
    start_area: IVec2,
    start_local: UVec2,
    kind: WaterKind,
) -> i32 {
    let cap: i32 = SHALLOW_BAND_TILES + 2;
    let mut frontier: Vec<(IVec2, UVec2, i32)> = vec![(start_area, start_local, 0)];
    let mut best: i32 = cap;
    while let Some((a, l, d)) = frontier.pop() {
        if d >= best {
            continue;
        }
        for &(dx, dy) in &TILE_NEIGHBOURS_4 {
            let Some(nbr) = neighbour_key(a, l, dx, dy) else {
                continue;
            };
            match map.tiles.get(&nbr) {
                Some(&k) if k == kind => {
                    if d + 1 < cap {
                        frontier.push((nbr.0, nbr.1, d + 1));
                    }
                }
                _ => {
                    // A non-matching-kind neighbour (or empty/ground) -- this
                    // is the shore from `start`'s perspective.
                    best = best.min(d + 1);
                }
            }
        }
    }
    best
}

fn classify_by_distance(
    map: &WaterMap,
    area: IVec2,
    local: UVec2,
    kind: WaterKind,
) -> WaterDepth {
    let dist = distance_to_non_matching(map, area, local, kind);
    let jitter = jitter_for(area, local);
    let threshold = SHALLOW_BAND_TILES + jitter;
    if dist <= threshold {
        WaterDepth::Shallow
    } else {
        WaterDepth::Deep
    }
}

/// Deterministic per-tile jitter in `[-SHALLOW_JITTER_TILES, +SHALLOW_JITTER_TILES]`.
fn jitter_for(area: IVec2, local: UVec2) -> i32 {
    #[allow(clippy::as_conversions)] // i32 bit-pattern reuse for hashing
    let ax = area.x as u32;
    #[allow(clippy::as_conversions)] // i32 bit-pattern reuse for hashing
    let ay = area.y as u32;
    let h = ax
        .wrapping_mul(2_654_435_761)
        .wrapping_add(ay.wrapping_mul(40_503))
        .wrapping_add(local.x.wrapping_mul(73_856_093))
        .wrapping_add(local.y.wrapping_mul(19_349_663))
        .wrapping_add(0x00C0_FFEE);
    let modulo = u32::try_from(SHALLOW_JITTER_TILES * 2 + 1).unwrap_or(1);
    let r = h % modulo;
    i32::try_from(r).unwrap_or(0) - SHALLOW_JITTER_TILES
}
