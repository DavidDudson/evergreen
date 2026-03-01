use std::collections::{BTreeSet, HashMap};

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::area::{Area, Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::terrain::Terrain;

/// Fired when the player crosses an area boundary and the current area changes.
#[derive(Message, Clone, Copy)]
pub struct AreaChanged;

/// All generated areas in the world, keyed by grid position.
///
/// The origin (0, 0) is the starting area.  Positive Y is north; positive X is east.
#[derive(Resource)]
pub struct WorldMap {
    areas: HashMap<IVec2, Area>,
    pub current: IVec2,
    seed: u64,
}

impl WorldMap {
    /// Create the world and seed the starting 4-way cross area plus its neighbours.
    pub fn new(seed: u64) -> Self {
        let start = IVec2::ZERO;
        let mut map = Self {
            areas: HashMap::new(),
            current: start,
            seed,
        };

        // The origin area is always a 4-way cross (all exits open) so the
        // player can immediately explore in every direction.
        let all_exits = BTreeSet::from([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]);
        let start_seed = map.area_seed(start);
        // Starting area always has all 4 exits; area_count=0 is irrelevant here.
        let start_area = Area::generate(all_exits, BTreeSet::new(), start_seed, 0);
        map.areas.insert(start, start_area);
        map.ensure_neighbors(start);
        map
    }

    /// Borrow the area that the player currently occupies.
    pub fn current_area(&self) -> &Area {
        self.areas
            .get(&self.current)
            .expect("current area is always generated before the player enters it")
    }

    /// Borrow any generated area by world-grid position.
    pub fn get_area(&self, pos: IVec2) -> Option<&Area> {
        self.areas.get(&pos)
    }

    /// Look up terrain at `(local_x, local_y)` relative to `area_pos`.
    ///
    /// Coordinates outside the 32×18 area bounds wrap into the adjacent area.
    /// Returns `None` if the neighbouring area has not been generated yet.
    pub fn terrain_at_extended(&self, area_pos: IVec2, local_x: i32, local_y: i32) -> Option<Terrain> {
        let w = i32::from(MAP_WIDTH);
        let h = i32::from(MAP_HEIGHT);

        if (0..w).contains(&local_x) && (0..h).contains(&local_y) {
            return self
                .areas
                .get(&area_pos)?
                .terrain_at(u32::try_from(local_x).ok()?, u32::try_from(local_y).ok()?);
        }

        let dx: i32 = match local_x {
            x if x < 0 => -1,
            x if x >= w => 1,
            _ => 0,
        };
        let dy: i32 = match local_y {
            y if y < 0 => -1,
            y if y >= h => 1,
            _ => 0,
        };

        let neighbour_pos = area_pos + IVec2::new(dx, dy);
        let nx = u32::try_from(local_x - dx * w).ok()?;
        let ny = u32::try_from(local_y - dy * h).ok()?;
        self.areas.get(&neighbour_pos)?.terrain_at(nx, ny)
    }

    /// Move to the area in `dir`, generating it and its neighbours if needed.
    pub fn transition(&mut self, dir: Direction) {
        let new_pos = self.current + dir.grid_offset();
        self.current = new_pos;
        self.ensure_area(new_pos);
        self.ensure_neighbors(new_pos);
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn ensure_area(&mut self, pos: IVec2) {
        if self.areas.contains_key(&pos) {
            return;
        }

        let mut required: BTreeSet<Direction> = BTreeSet::new();
        let mut forbidden: BTreeSet<Direction> = BTreeSet::new();

        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let neighbour_pos = pos + dir.grid_offset();
            if let Some(neighbour) = self.areas.get(&neighbour_pos) {
                if neighbour.exits.contains(&dir.opposite()) {
                    required.insert(dir);
                } else {
                    forbidden.insert(dir);
                }
            }
        }

        let seed = self.area_seed(pos);
        let area_count = self.areas.len();
        let area = Area::generate(required, forbidden, seed, area_count);
        self.areas.insert(pos, area);
    }

    fn ensure_neighbors(&mut self, pos: IVec2) {
        // Collect exits first to avoid borrowing `self` while mutating it.
        let exits: Vec<Direction> = self
            .areas
            .get(&pos)
            .map(|a| a.exits.iter().copied().collect())
            .unwrap_or_default();

        for dir in exits {
            let neighbour_pos = pos + dir.grid_offset();
            self.ensure_area(neighbour_pos);
        }
    }

    /// Derive a deterministic seed for any grid position from the world seed.
    fn area_seed(&self, pos: IVec2) -> u64 {
        // Reinterpret i32 bits as u32 (bit-cast via to/from bytes) then widen.
        let px = u64::from(u32::from_ne_bytes(pos.x.to_ne_bytes()));
        let py = u64::from(u32::from_ne_bytes(pos.y.to_ne_bytes()));
        self.seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(px.wrapping_mul(1_442_695_040_888_963_407))
            .wrapping_add(py.wrapping_mul(2_654_435_761))
    }
}
