use std::collections::BTreeSet;

use bevy::math::IVec2;

use crate::terrain::Terrain;

pub const MAP_WIDTH: u16 = 32;
pub const MAP_HEIGHT: u16 = 18;

// Exit-path geometry — must be consistent across all areas so neighbours align.
// Vertical N/S path: columns 14–16.
const PATH_COL_START: u32 = 14;
const PATH_COL_END: u32 = 16;
// Horizontal E/W path: rows 7–9 (y=0 is the bottom tile row).
const PATH_ROW_START: u32 = 7;
const PATH_ROW_END: u32 = 9;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    pub fn grid_offset(self) -> IVec2 {
        match self {
            Self::North => IVec2::new(0, 1),
            Self::South => IVec2::new(0, -1),
            Self::East => IVec2::new(1, 0),
            Self::West => IVec2::new(-1, 0),
        }
    }
}

/// One 32×18 section of the world map with defined exit directions.
pub struct Area {
    pub exits: BTreeSet<Direction>,
    /// Row-major grid: index = y * MAP_WIDTH + x.  y=0 is the bottom tile row.
    grid: Vec<Terrain>,
}

impl Area {
    /// Generate an area.  `required` exits must be present; `forbidden` exits
    /// must be absent (they border a neighbour that has no path toward us).
    /// `area_count` is the number of areas already in the world; the
    /// probability of optional exits decreases as the map approaches the
    /// 15–20 area target size.
    pub fn generate(
        required: BTreeSet<Direction>,
        forbidden: BTreeSet<Direction>,
        seed: u64,
        area_count: usize,
    ) -> Self {
        let exits = pick_exits(required, &forbidden, seed, area_count);
        let grid = build_grid(&exits);
        Self { exits, grid }
    }

    pub fn terrain_at(&self, x: u32, y: u32) -> Option<Terrain> {
        if x >= u32::from(MAP_WIDTH) || y >= u32::from(MAP_HEIGHT) {
            return None;
        }
        // u32 → usize: widening on all supported targets (usize ≥ 32 bits).
        #[allow(clippy::as_conversions)]
        let idx = (y * u32::from(MAP_WIDTH) + x) as usize;
        self.grid.get(idx).copied()
    }

}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn pick_exits(
    mut exits: BTreeSet<Direction>,
    forbidden: &BTreeSet<Direction>,
    seed: u64,
    area_count: usize,
) -> BTreeSet<Direction> {
    let all = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    // Collect optional directions (not required, not forbidden).
    let optional: Vec<Direction> = all
        .iter()
        .filter(|d| !exits.contains(d) && !forbidden.contains(d))
        .copied()
        .collect();

    if optional.is_empty() {
        return exits;
    }

    let mut rng = seed;

    // Target exit count varies with map density so the world grows fast
    // early and tapers to dead-ends near the 15-20 area target.
    // Required exits are already in `exits`; we only control optional ones.
    rng = lcg(rng);
    let roll = rng % 100;
    let ideal: usize = if area_count < 6 {
        // Early: grow fast — prefer 3-4 exits.
        if roll < 5 { 1 } else if roll < 25 { 2 } else if roll < 75 { 3 } else { 4 }
    } else if area_count < 11 {
        // Growing: balanced toward 3.
        if roll < 10 { 1 } else if roll < 40 { 2 } else if roll < 85 { 3 } else { 4 }
    } else if area_count < 15 {
        // Middle: original 1-3 distribution.
        if roll < 20 { 1 } else if roll < 60 { 2 } else if roll < 95 { 3 } else { 4 }
    } else if area_count < 18 {
        // Tapering: mostly 1-2, rare 3.
        if roll < 50 { 1 } else if roll < 90 { 2 } else { 3 }
    } else if area_count < 21 {
        // Near cap: dead-ends dominant.
        if roll < 75 { 1 } else { 2 }
    } else {
        // Hard cap zone: almost always a dead-end.
        if roll < 95 { 1 } else { 2 }
    };
    let target = ideal.max(exits.len()).min(exits.len() + optional.len());
    let n_to_add = target - exits.len();

    // Fisher-Yates shuffle of optional list, then take the first n_to_add.
    let mut shuffled = optional;
    for i in (1..shuffled.len()).rev() {
        rng = lcg(rng);
        let j = usize::try_from(rng % u64::try_from(i + 1).expect("i+1 fits u64"))
            .expect("mod result fits usize");
        shuffled.swap(i, j);
    }

    for dir in shuffled.into_iter().take(n_to_add) {
        exits.insert(dir);
    }

    exits
}

fn build_grid(exits: &BTreeSet<Direction>) -> Vec<Terrain> {
    let w = u32::from(MAP_WIDTH);
    let h = u32::from(MAP_HEIGHT);
    // u32 → usize: widening on all supported targets.
    #[allow(clippy::as_conversions)]
    let mut grid = vec![Terrain::Grass; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            if is_dirt(x, y, exits) {
                // u32 → usize: widening on all supported targets.
                #[allow(clippy::as_conversions)]
                let idx = (y * w + x) as usize;
                grid[idx] = Terrain::Dirt;
            }
        }
    }

    grid
}

/// Returns `true` when the tile at (x, y) should be dirt given the exit set.
///
/// The N/S path covers columns 14–16.  The N arm reaches from the
/// intersection (rows 7–9) to the top edge; the S arm to the bottom edge.
/// The E/W path covers rows 7–9.  The E arm reaches from the intersection
/// to the right edge; the W arm to the left edge.
fn is_dirt(x: u32, y: u32, exits: &BTreeSet<Direction>) -> bool {
    let on_vert = (PATH_COL_START..=PATH_COL_END).contains(&x);
    let on_horiz = (PATH_ROW_START..=PATH_ROW_END).contains(&y);

    (on_vert && exits.contains(&Direction::North) && y >= PATH_ROW_START)
        || (on_vert && exits.contains(&Direction::South) && y <= PATH_ROW_END)
        || (on_horiz && exits.contains(&Direction::East) && x >= PATH_COL_START)
        || (on_horiz && exits.contains(&Direction::West) && x <= PATH_COL_END)
}

/// Minimal LCG for cheap deterministic pseudo-randomness.
fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
