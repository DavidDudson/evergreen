use std::collections::BTreeSet;

use bevy::math::IVec2;

use crate::terrain::Terrain;

/// Identifies an NPC for area events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcKind {
    Mordred,
    Drizella,
    Bigby,
    Gothel,
    Morgana,
    Cadwallader,
}

impl NpcKind {
    /// Alignment range this NPC will spawn in (min, max).
    pub fn alignment_range(self) -> (u8, u8) {
        match self {
            // City-aligned
            Self::Cadwallader => (1, 35),
            Self::Bigby => (1, 35),
            // Greenwood-aligned
            Self::Drizella => (25, 75),
            Self::Gothel => (25, 75),
            // Darkwood-aligned
            Self::Mordred => (60, 100),
            Self::Morgana => (60, 100),
        }
    }
}

/// All available NPC kinds, used for random selection.
pub const ALL_NPCS: [NpcKind; 6] = [
    NpcKind::Mordred,
    NpcKind::Drizella,
    NpcKind::Bigby,
    NpcKind::Gothel,
    NpcKind::Morgana,
    NpcKind::Cadwallader,
];

/// What happens when the player enters an area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AreaEvent {
    #[default]
    None,
    NpcEncounter(NpcKind),
}

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

/// Alignment scale: 1 = city, 50 = greenwood, 100 = darkwood.
pub type AreaAlignment = u8;

/// One 32x18 section of the world map with defined exit directions.
pub struct Area {
    pub exits: BTreeSet<Direction>,
    /// What happens when the player enters this area.
    pub event: AreaEvent,
    /// Biome alignment (1 = city, 50 = greenwood, 100 = darkwood).
    pub alignment: AreaAlignment,
    /// Row-major grid: index = y * MAP_WIDTH + x.  y=0 is the bottom tile row.
    grid: Vec<Terrain>,
}

impl Area {
    /// Generate an area.  `required` exits must be present; `forbidden` exits
    /// must be absent (they border a neighbour that has no path toward us).
    /// `area_count` is the number of areas already in the world; the
    /// probability of optional exits decreases as the map approaches the
    /// 15–20 area target size.
    /// `alignment` controls the biome (1 = city, 50 = greenwood, 100 = darkwood).
    pub fn generate(
        required: BTreeSet<Direction>,
        forbidden: BTreeSet<Direction>,
        seed: u64,
        area_count: usize,
        alignment: AreaAlignment,
    ) -> Self {
        let exits = pick_exits(required, &forbidden, seed, area_count);
        let grid = build_grid(&exits, alignment);
        Self {
            exits,
            event: AreaEvent::None,
            alignment,
            grid,
        }
    }

    /// An impassable area filled entirely with grass (dense forest).
    /// Used as a visual border for edges that have no neighboring area.
    pub fn dense_forest() -> Self {
        #[allow(clippy::as_conversions)]
        let grid = vec![Terrain::Grass; (u32::from(MAP_WIDTH) * u32::from(MAP_HEIGHT)) as usize];
        Self {
            exits: BTreeSet::new(),
            event: AreaEvent::None,
            alignment: 100, // dense forest = darkwood
            grid,
        }
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
        if roll < 5 {
            1
        } else if roll < 25 {
            2
        } else if roll < 75 {
            3
        } else {
            4
        }
    } else if area_count < 11 {
        // Growing: balanced toward 3.
        if roll < 10 {
            1
        } else if roll < 40 {
            2
        } else if roll < 85 {
            3
        } else {
            4
        }
    } else if area_count < 15 {
        // Middle: original 1-3 distribution.
        if roll < 20 {
            1
        } else if roll < 60 {
            2
        } else if roll < 95 {
            3
        } else {
            4
        }
    } else if area_count < 18 {
        // Tapering: mostly 1-2, rare 3.
        if roll < 50 {
            1
        } else if roll < 90 {
            2
        } else {
            3
        }
    } else if area_count < 21 {
        // Near cap: dead-ends dominant.
        if roll < 75 {
            1
        } else {
            2
        }
    } else {
        // Hard cap zone: almost always a dead-end.
        if roll < 95 {
            1
        } else {
            2
        }
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

fn build_grid(exits: &BTreeSet<Direction>, alignment: AreaAlignment) -> Vec<Terrain> {
    let w = u32::from(MAP_WIDTH);
    let h = u32::from(MAP_HEIGHT);
    // u32 → usize: widening on all supported targets.
    #[allow(clippy::as_conversions)]
    let mut grid = vec![Terrain::Grass; (w * h) as usize];

    let extent = path_extent(alignment);

    for y in 0..h {
        for x in 0..w {
            if is_dirt(x, y, &exits, extent) {
                #[allow(clippy::as_conversions)]
                let idx = (y * w + x) as usize;
                grid[idx] = Terrain::Dirt;
            }
        }
    }

    // City clearings: large open dirt area around the intersection.
    if alignment < 30 {
        let clearing_r = city_clearing_radius(alignment);
        let cx = (PATH_COL_START + PATH_COL_END) / 2;
        let cy = (PATH_ROW_START + PATH_ROW_END) / 2;
        for y in 0..h {
            for x in 0..w {
                let dx = x.abs_diff(cx);
                let dy = y.abs_diff(cy);
                if dx + dy <= clearing_r {
                    #[allow(clippy::as_conversions)]
                    let idx = (y * w + x) as usize;
                    grid[idx] = Terrain::Dirt;
                }
            }
        }
    }

    // Darkwood scattered dirt patches: random disconnected spots.
    if alignment > 70 {
        let seed = u64::from(alignment).wrapping_mul(2_654_435_761);
        scatter_dirt_patches(&mut grid, w, h, alignment, seed);
    }

    grid
}

/// Total path half-width in tiles from the centre column/row.
/// City (1) = 5 (11-wide), Greenwood (50) = 2 (5-wide),
/// Deep green (65) = 1 (2-wide), Darkwood (100) = 0 (1-wide).
fn path_extent(alignment: AreaAlignment) -> u32 {
    #[allow(clippy::as_conversions)]
    match alignment {
        1..=25 => 5,  // City: wide open roads
        26..=50 => 2, // Greenwood: comfortable path
        51..=65 => 1, // Deep green: narrow 2-wide trail
        _ => 0,       // Darkwood: single-tile track
    }
}

/// Manhattan-distance clearing radius for city areas.
/// Lower alignment = bigger clearing (up to 10 tiles).
fn city_clearing_radius(alignment: AreaAlignment) -> u32 {
    let t = f32::from(alignment.clamp(1, 30)) / 30.0;
    #[allow(clippy::as_conversions)]
    let r = (10.0 * (1.0 - t)).round() as u32;
    r
}

/// Scatter small disconnected dirt patches in darkwood areas.
fn scatter_dirt_patches(grid: &mut [Terrain], w: u32, h: u32, alignment: AreaAlignment, seed: u64) {
    let intensity = f32::from(alignment.saturating_sub(70)) / 30.0;
    #[allow(clippy::as_conversions)]
    let patch_count = (intensity * 8.0).round() as u32;
    let mut rng = seed;

    for _ in 0..patch_count {
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let px = (rng % u64::from(w)) as u32;
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let py = (rng % u64::from(h)) as u32;
        rng = lcg(rng);
        #[allow(clippy::as_conversions)]
        let radius = (rng % 3) as u32;

        for dy in 0..=radius {
            for dx in 0..=(radius - dy) {
                for &(sx, sy) in &[
                    (px + dx, py + dy),
                    (px.wrapping_sub(dx), py + dy),
                    (px + dx, py.wrapping_sub(dy)),
                    (px.wrapping_sub(dx), py.wrapping_sub(dy)),
                ] {
                    if sx < w && sy < h {
                        #[allow(clippy::as_conversions)]
                        let idx = (sy * w + sx) as usize;
                        grid[idx] = Terrain::Dirt;
                    }
                }
            }
        }
    }
}

/// Returns `true` when the tile at (x, y) should be dirt given the exit set
/// and path half-width.
fn is_dirt(x: u32, y: u32, exits: &BTreeSet<Direction>, half_w: u32) -> bool {
    // Centre of the cross intersection.
    let cx = (PATH_COL_START + PATH_COL_END) / 2; // 15
    let cy = (PATH_ROW_START + PATH_ROW_END) / 2; // 8

    let on_vert = x.abs_diff(cx) <= half_w;
    let on_horiz = y.abs_diff(cy) <= half_w;

    (on_vert && exits.contains(&Direction::North) && y >= cy.saturating_sub(half_w))
        || (on_vert && exits.contains(&Direction::South) && y <= cy + half_w)
        || (on_horiz && exits.contains(&Direction::East) && x >= cx.saturating_sub(half_w))
        || (on_horiz && exits.contains(&Direction::West) && x <= cx + half_w)
}

/// Minimal LCG for cheap deterministic pseudo-randomness.
fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
