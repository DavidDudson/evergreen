//! Water-tile data structures: `WaterKind`, `WaterMap`, plus the cardinal
//! neighbour helpers used by both generation and shore spawning.
//!
//! Generation logic itself lives in `super::generate`; spawning in
//! `super::shore`; per-frame animation in `super::animation`.

use std::collections::{HashMap, HashSet};

use bevy::math::{IVec2, UVec2};
use bevy::prelude::Component;
use models::tags::{tag, Tag, TerrainTags};

use crate::area::{MAP_HEIGHT, MAP_WIDTH};

use super::depth::WaterDepth;

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

/// Which flavour of water a tile belongs to. Controls tint, fauna spawns,
/// and particle effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterKind {
    /// Plain freshwater pond. Spawns frogs + lily pads.
    Plain,
    /// Geothermal pool. Teal tint, steam particles, no frogs.
    HotSpring,
    /// Large multi-area body of water. Frogs, lily pads, striders.
    Lake,
    /// River flowing vertically (north-south).
    RiverNS,
    /// River flowing horizontally (east-west).
    RiverEW,
    /// River pouring over the top edge of the world. Rendered differently
    /// from regular river tiles (vertical falling sprite).
    Waterfall,
    /// Ocean at the world edge. Fish shadows drift across these tiles.
    Ocean,
}

impl WaterKind {
    pub fn spawns_frogs(self) -> bool {
        matches!(self, Self::Plain | Self::Lake)
    }

    pub fn spawns_lily_pads(self) -> bool {
        matches!(self, Self::Plain | Self::Lake)
    }

    pub fn spawns_steam(self) -> bool {
        matches!(self, Self::HotSpring)
    }

    /// Whether this tile is a flowing river segment (or waterfall).
    pub fn is_river(self) -> bool {
        matches!(self, Self::RiverNS | Self::RiverEW | Self::Waterfall)
    }

    /// Still-water kinds where insects can rest on the surface.
    pub fn is_still(self) -> bool {
        matches!(self, Self::Plain | Self::HotSpring | Self::Lake)
    }

    /// True for ocean tiles. Used by fauna rules.
    pub fn is_ocean(self) -> bool {
        matches!(self, Self::Ocean)
    }

    /// Tag-based terrain descriptor used by the placement system.
    /// Add new tags here when introducing new water flavours; placeables can
    /// then opt-in via `PlacementRequirement::requires/forbids/tolerates`.
    ///
    /// Note: depth tags (`shallow`/`deep`) are NOT applied here -- query
    /// [`WaterMap::terrain_tags_at`] for the per-tile combined view.
    pub fn terrain_tags(self) -> TerrainTags {
        match self {
            Self::Plain => TerrainTags::new(&[tag::WATER, tag::STILL], &[]),
            Self::Lake => TerrainTags::new(&[tag::WATER, tag::STILL], &[]),
            Self::HotSpring => {
                TerrainTags::new(&[tag::WATER, tag::STILL, tag::HOT], &[tag::HOT])
            }
            Self::RiverNS | Self::RiverEW => {
                TerrainTags::new(&[tag::WATER, tag::FLOWING], &[tag::FLOWING])
            }
            Self::Waterfall => {
                TerrainTags::new(&[tag::WATER, tag::FLOWING], &[tag::FLOWING])
            }
            Self::Ocean => {
                TerrainTags::new(&[tag::WATER, tag::STILL, tag::SALT], &[tag::SALT])
            }
        }
    }
}

/// Concrete `[Tag; 8]` whose populated prefix is referenced as a `&'static`-
/// shaped slice for [`TerrainTags::tags`]. We can't return a true `'static`
/// slice when depth is dynamic, so callers must use the returned value
/// inline (don't store the inner slice past the [`TileTags`] drop).
pub struct TileTags {
    storage: [Tag; 8],
    len: usize,
    taints_storage: [Tag; 4],
    taints_len: usize,
}

impl TileTags {
    pub fn tags(&self) -> &[Tag] {
        &self.storage[..self.len]
    }

    pub fn taints(&self) -> &[Tag] {
        &self.taints_storage[..self.taints_len]
    }

    /// Convenience: does this tile have the given tag?
    pub fn has(&self, t: Tag) -> bool {
        self.tags().contains(&t)
    }
}

/// `(area grid position, local tile within area)` key for a single water tile.
pub type WaterKey = (IVec2, UVec2);

/// Marker on each spawned water-tile entity.
#[derive(Component)]
pub struct WaterTile {
    pub kind: WaterKind,
}

/// All generated water tiles in the world.
#[derive(Default, Debug)]
pub struct WaterMap {
    pub(super) tiles: HashMap<WaterKey, WaterKind>,
    /// Per-tile depth (`Shallow` or `Deep`). Computed by
    /// [`super::depth::classify_depths`] after all tiles are placed.
    pub(super) depths: HashMap<WaterKey, WaterDepth>,
    /// Tiles where a stepping stone sits on top of the water (walkable).
    pub(super) stones: HashSet<WaterKey>,
    /// Sand tiles inland of ocean tiles. Not water -- but stored here so
    /// spawn systems only need a single map to query.
    pub(super) sand: HashSet<WaterKey>,
    /// Pier tiles overlaying ocean / sand. Walkable, no collider.
    pub(super) piers: HashSet<WaterKey>,
}

impl WaterMap {
    pub fn get(&self, area_pos: IVec2, local: UVec2) -> Option<WaterKind> {
        self.tiles.get(&(area_pos, local)).copied()
    }

    pub fn depth_at(&self, area_pos: IVec2, local: UVec2) -> Option<WaterDepth> {
        self.depths.get(&(area_pos, local)).copied()
    }

    pub fn is_shallow(&self, area_pos: IVec2, local: UVec2) -> bool {
        matches!(self.depth_at(area_pos, local), Some(WaterDepth::Shallow))
    }

    pub fn is_deep(&self, area_pos: IVec2, local: UVec2) -> bool {
        matches!(self.depth_at(area_pos, local), Some(WaterDepth::Deep))
    }

    pub fn tiles_in_area(&self, area_pos: IVec2) -> Vec<(UVec2, WaterKind)> {
        self.tiles
            .iter()
            .filter(|((a, _), _)| *a == area_pos)
            .map(|((_, local), kind)| (*local, *kind))
            .collect()
    }

    /// Whether the given water tile has any non-water neighbour (i.e. is an
    /// edge tile where reeds / cattails should spawn).
    pub fn is_edge_tile(&self, area_pos: IVec2, local: UVec2) -> bool {
        TILE_NEIGHBOURS_4.iter().any(|&(dx, dy)| {
            neighbour_key(area_pos, local, dx, dy).is_some_and(|k| !self.tiles.contains_key(&k))
        })
    }

    /// Is there a stepping stone on this water tile (walkable)?
    pub fn has_stone(&self, area_pos: IVec2, local: UVec2) -> bool {
        self.stones.contains(&(area_pos, local))
    }

    /// Every stone in one area.
    pub fn stones_in_area(&self, area_pos: IVec2) -> Vec<UVec2> {
        self.stones
            .iter()
            .filter(|(a, _)| *a == area_pos)
            .map(|(_, local)| *local)
            .collect()
    }

    pub fn has_sand(&self, area_pos: IVec2, local: UVec2) -> bool {
        self.sand.contains(&(area_pos, local))
    }

    pub fn sand_in_area(&self, area_pos: IVec2) -> Vec<UVec2> {
        self.sand
            .iter()
            .filter(|(a, _)| *a == area_pos)
            .map(|(_, local)| *local)
            .collect()
    }

    pub fn has_pier(&self, area_pos: IVec2, local: UVec2) -> bool {
        self.piers.contains(&(area_pos, local))
    }

    pub fn piers_in_area(&self, area_pos: IVec2) -> Vec<UVec2> {
        self.piers
            .iter()
            .filter(|(a, _)| *a == area_pos)
            .map(|(_, local)| *local)
            .collect()
    }

    /// Per-tile terrain tags including depth. Returns [`TileTags`] (a small
    /// stack-allocated buffer) so callers don't depend on a single static
    /// slice -- the tag list varies per (kind, depth).
    pub fn terrain_tags_at(&self, area_pos: IVec2, local: UVec2) -> Option<TileTags> {
        let kind = self.get(area_pos, local)?;
        let base = kind.terrain_tags();
        let depth_tag = match self.depth_at(area_pos, local)? {
            WaterDepth::Shallow => tag::SHALLOW,
            WaterDepth::Deep => tag::DEEP,
        };
        let mut storage = [""; 8];
        let mut len = 0usize;
        for &t in base.tags {
            if len < storage.len() {
                storage[len] = t;
                len += 1;
            }
        }
        if len < storage.len() {
            storage[len] = depth_tag;
            len += 1;
        }
        let mut taints_storage = [""; 4];
        let mut taints_len = 0usize;
        for &t in base.taints {
            if taints_len < taints_storage.len() {
                taints_storage[taints_len] = t;
                taints_len += 1;
            }
        }
        Some(TileTags {
            storage,
            len,
            taints_storage,
            taints_len,
        })
    }
}

// ---------------------------------------------------------------------------
// Neighbour helpers (used by generation, shore, and the edge predicate above)
// ---------------------------------------------------------------------------

/// 4-neighbour deltas for flood-fill (N/S/E/W).
pub(super) const TILE_NEIGHBOURS_4: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

/// Step one tile in cardinal direction from `(area_pos, local)`. If we walk
/// off the area's `MAP_WIDTH x MAP_HEIGHT` grid, the returned key is in the
/// neighbouring area at the wrapped local coord. Returns `None` only when
/// `local` underflows negative for a start tile already at 0.
pub(super) fn neighbour_key(area_pos: IVec2, local: UVec2, dx: i32, dy: i32) -> Option<WaterKey> {
    let w = i32::from(MAP_WIDTH);
    let h = i32::from(MAP_HEIGHT);
    let lx = i32::try_from(local.x).ok()? + dx;
    let ly = i32::try_from(local.y).ok()? + dy;

    let area_dx = if lx < 0 {
        -1
    } else if lx >= w {
        1
    } else {
        0
    };
    let area_dy = if ly < 0 {
        -1
    } else if ly >= h {
        1
    } else {
        0
    };

    let new_area = area_pos + IVec2::new(area_dx, area_dy);
    let new_lx = u32::try_from(lx - area_dx * w).ok()?;
    let new_ly = u32::try_from(ly - area_dy * h).ok()?;
    Some((new_area, UVec2::new(new_lx, new_ly)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbour_within_area() {
        let key = neighbour_key(IVec2::ZERO, UVec2::new(5, 5), 1, 0).unwrap();
        assert_eq!(key, (IVec2::ZERO, UVec2::new(6, 5)));
    }

    #[test]
    fn neighbour_crosses_area_east() {
        let key =
            neighbour_key(IVec2::ZERO, UVec2::new(u32::from(MAP_WIDTH) - 1, 5), 1, 0).unwrap();
        assert_eq!(key, (IVec2::new(1, 0), UVec2::new(0, 5)));
    }

    #[test]
    fn neighbour_crosses_area_north() {
        let key =
            neighbour_key(IVec2::ZERO, UVec2::new(5, u32::from(MAP_HEIGHT) - 1), 0, 1).unwrap();
        assert_eq!(key, (IVec2::new(0, 1), UVec2::new(5, 0)));
    }

    #[test]
    fn water_kinds_have_correct_spawn_flags() {
        assert!(WaterKind::Plain.spawns_frogs());
        assert!(!WaterKind::HotSpring.spawns_frogs());
        assert!(WaterKind::HotSpring.spawns_steam());
        assert!(!WaterKind::Plain.spawns_steam());
        assert!(WaterKind::Lake.spawns_lily_pads());
    }
}
