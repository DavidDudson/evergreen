use std::collections::{BTreeSet, HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::area::{
    Area, AreaAlignment, AreaEvent, Direction, NpcKind, ALL_NPCS, MAP_HEIGHT, MAP_WIDTH,
};
use crate::terrain::Terrain;

/// Maximum number of NPC encounters in the world.
const MAX_NPC_ENCOUNTERS: usize = 3;

/// Minimum Manhattan distance from origin for an NPC encounter.
const MIN_NPC_DISTANCE: i32 = 3;

/// Probability (out of 100) that an eligible area gets an NPC encounter.
const NPC_ENCOUNTER_CHANCE: u64 = 30;

/// Number of biome zone seeds placed in the world.
const ZONE_SEED_COUNT: usize = 4;

/// Minimum grid-distance between any two zone seeds.
const MIN_ZONE_SPACING: i32 = 2;

/// Maximum placement radius from origin for zone seeds.
const ZONE_RADIUS: i32 = 5;

/// Alignment anchors for the biome types (spread to avoid averaging to 50).
const ANCHOR_CITY: u8 = 10;
const ANCHOR_LIGHT_GREEN: u8 = 35;
const ANCHOR_DEEP_GREEN: u8 = 65;
const ANCHOR_DARKWOOD: u8 = 90;

/// A biome influence point in the world grid.
struct ZoneSeed {
    pos: IVec2,
    alignment: AreaAlignment,
}

/// Fired when the player crosses an area boundary and the current area changes.
#[derive(Message, Clone, Copy)]
pub struct AreaChanged {
    pub direction: Direction,
}

/// All generated areas in the world, keyed by grid position.
///
/// The origin (0, 0) is the starting area.  Positive Y is north; positive X is east.
#[derive(Resource)]
pub struct WorldMap {
    areas: HashMap<IVec2, Area>,
    pub current: IVec2,
    seed: u64,
    /// NPCs available for encounters, shuffled at creation.
    npc_pool: Vec<NpcKind>,
    /// How many NPC encounters have been placed so far.
    npc_count: usize,
    /// Areas the player has entered.
    visited: HashSet<IVec2>,
    /// Areas visible on the minimap (visited + their exit neighbors).
    revealed: HashSet<IVec2>,
    /// Biome zone influence points for alignment interpolation.
    zone_seeds: Vec<ZoneSeed>,
    /// The dead-end area containing the level exit.
    pub exit_area: IVec2,
}

impl WorldMap {
    /// Create the world and seed the starting 4-way cross area plus two rings
    /// of neighbours (enough to populate the initial minimap).
    ///
    /// `dominant_alignment` is the player's dominant faction expressed on the
    /// 1-100 scale (1 = city, 50 = greenwood, 100 = darkwood).  The start
    /// position is chosen near the best-matching zone seed.
    pub fn new(seed: u64, dominant_alignment: AreaAlignment) -> Self {
        // Shuffle NPC pool deterministically from seed.
        let mut npc_pool: Vec<NpcKind> = ALL_NPCS.to_vec();
        let mut rng = seed.wrapping_mul(7_046_029_254_386_353_131);
        for i in (1..npc_pool.len()).rev() {
            rng = lcg(rng);
            let j = usize::try_from(rng % u64::try_from(i + 1).expect("i+1 fits u64"))
                .expect("mod fits usize");
            npc_pool.swap(i, j);
        }

        let zone_seeds = generate_zone_seeds(seed);

        let mut map = Self {
            areas: HashMap::new(),
            current: IVec2::ZERO,
            seed,
            npc_pool,
            npc_count: 0,
            visited: HashSet::new(),
            revealed: HashSet::new(),
            zone_seeds,
            exit_area: IVec2::ZERO,
        };

        // Seed the origin as a 4-way cross to bootstrap generation.
        let all_exits = BTreeSet::from([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]);
        let origin_seed = map.area_seed(IVec2::ZERO);
        let origin_align = map.alignment_at(IVec2::ZERO);
        let origin_area = Area::generate(all_exits, BTreeSet::new(), origin_seed, 0, origin_align);
        map.areas.insert(IVec2::ZERO, origin_area);

        // Expand the entire map: keep generating neighbors until no new
        // areas appear. This produces the full 15-20 area world.
        loop {
            let positions: Vec<IVec2> = map.areas.keys().copied().collect();
            let before = map.areas.len();
            for pos in positions {
                map.ensure_neighbors(pos);
            }
            if map.areas.len() == before {
                break;
            }
        }

        // Find dead-end areas (exactly 1 exit).
        let dead_ends: Vec<IVec2> = map
            .areas
            .iter()
            .filter(|(_, a)| a.exits.len() == 1)
            .map(|(pos, _)| *pos)
            .collect();

        // Pick start: dead end closest to player's alignment-preferred zone.
        let start =
            pick_dead_end_for_alignment(&dead_ends, &map.zone_seeds, dominant_alignment, seed);
        map.current = start;
        map.visited.insert(start);
        map.revealed.insert(start);
        map.reveal_exits(start);

        // Pick exit: dead end farthest from start.
        let exit = dead_ends
            .iter()
            .filter(|&&p| p != start)
            .max_by_key(|&&p| manhattan(p, start))
            .copied()
            .unwrap_or(IVec2::ZERO);
        map.exit_area = exit;

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

    /// All generated area positions.
    pub fn area_positions(&self) -> Vec<IVec2> {
        self.areas.keys().copied().collect()
    }

    /// The world seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Look up terrain at `(local_x, local_y)` relative to `area_pos`.
    ///
    /// Coordinates outside the 32x18 area bounds wrap into the adjacent area.
    /// Returns `None` if the neighbouring area has not been generated yet.
    pub fn terrain_at_extended(
        &self,
        area_pos: IVec2,
        local_x: i32,
        local_y: i32,
    ) -> Option<Terrain> {
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

    /// Whether an area should be visible on the minimap.
    /// Persists across transitions so previously-seen areas stay visible.
    pub fn is_revealed(&self, pos: IVec2) -> bool {
        self.revealed.contains(&pos)
    }

    /// Move to the area in `dir`, generating it and its neighbours if needed.
    pub fn transition(&mut self, dir: Direction) {
        let new_pos = self.current + dir.grid_offset();
        self.current = new_pos;
        self.visited.insert(new_pos);
        self.revealed.insert(new_pos);
        self.ensure_area(new_pos);
        self.ensure_neighbors(new_pos);
        self.reveal_exits(new_pos);
    }

    /// Compute the biome alignment for a grid position via inverse-distance
    /// weighting from zone seeds.  Falls back to greenwood (50) if no seeds.
    pub fn alignment_at(&self, pos: IVec2) -> AreaAlignment {
        alignment_from_zones(&self.zone_seeds, pos)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Find the next unplaced NPC whose alignment range contains `alignment`.
    fn find_npc_for_alignment(&self, alignment: AreaAlignment) -> Option<&NpcKind> {
        self.npc_pool.iter().skip(self.npc_count).find(|npc| {
            let (lo, hi) = npc.alignment_range();
            alignment >= lo && alignment <= hi
        })
    }

    /// Mark all exit-connected neighbors of `pos` as revealed on the minimap.
    fn reveal_exits(&mut self, pos: IVec2) {
        let exits: Vec<Direction> = self
            .areas
            .get(&pos)
            .map(|a| a.exits.iter().copied().collect())
            .unwrap_or_default();

        for dir in exits {
            self.revealed.insert(pos + dir.grid_offset());
        }
    }

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
        let alignment = self.alignment_at(pos);
        let mut area = Area::generate(required, forbidden, seed, area_count, alignment);

        // Assign event: NPC encounters only beyond MIN_NPC_DISTANCE from origin,
        // and only if the NPC's alignment range matches this area.
        let distance = pos.x.abs() + pos.y.abs();
        if distance >= MIN_NPC_DISTANCE && self.npc_count < MAX_NPC_ENCOUNTERS {
            let event_rng = lcg(seed.wrapping_add(0xCAFE));
            if event_rng % 100 < NPC_ENCOUNTER_CHANCE {
                if let Some(&npc) = self.find_npc_for_alignment(alignment) {
                    area.event = AreaEvent::NpcEncounter(npc);
                    self.npc_count += 1;
                }
            }
        }

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
        let px = u64::from(u32::from_ne_bytes(pos.x.to_ne_bytes()));
        let py = u64::from(u32::from_ne_bytes(pos.y.to_ne_bytes()));
        self.seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(px.wrapping_mul(1_442_695_040_888_963_407))
            .wrapping_add(py.wrapping_mul(2_654_435_761))
    }
}

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

// ---------------------------------------------------------------------------
// Zone seed generation
// ---------------------------------------------------------------------------

/// Place biome zone seeds at random positions with random alignment anchors.
fn generate_zone_seeds(seed: u64) -> Vec<ZoneSeed> {
    let anchors = [
        ANCHOR_CITY,
        ANCHOR_LIGHT_GREEN,
        ANCHOR_DEEP_GREEN,
        ANCHOR_DARKWOOD,
    ];
    let mut seeds = Vec::with_capacity(ZONE_SEED_COUNT);
    let mut rng = lcg(seed.wrapping_add(0xB10_E));

    for i in 0..ZONE_SEED_COUNT {
        // Try up to 20 times to find a position far enough from existing seeds.
        let mut pos = IVec2::ZERO;
        for _ in 0..20 {
            rng = lcg(rng);
            #[allow(clippy::as_conversions)]
            let x =
                (rng % u64::try_from(ZONE_RADIUS * 2 + 1).expect("fits u64")) as i32 - ZONE_RADIUS;
            rng = lcg(rng);
            #[allow(clippy::as_conversions)]
            let y =
                (rng % u64::try_from(ZONE_RADIUS * 2 + 1).expect("fits u64")) as i32 - ZONE_RADIUS;
            pos = IVec2::new(x, y);

            let far_enough = seeds
                .iter()
                .all(|s: &ZoneSeed| manhattan(pos, s.pos) >= MIN_ZONE_SPACING);
            if far_enough {
                break;
            }
        }

        let alignment = anchors[i % anchors.len()];
        seeds.push(ZoneSeed { pos, alignment });
    }

    seeds
}

/// Compute alignment at a grid position from the nearest zone seed.
///
/// Uses straight nearest-zone assignment with deterministic jitter
/// (up to +/-15) for noticeable variation between adjacent areas.
fn alignment_from_zones(zones: &[ZoneSeed], pos: IVec2) -> AreaAlignment {
    if zones.is_empty() {
        return ANCHOR_LIGHT_GREEN;
    }

    let nearest = zones
        .iter()
        .min_by_key(|z| manhattan(pos, z.pos))
        .expect("zones is non-empty");

    // Deterministic jitter based on position -- large enough for visible
    // biome variation between neighbors.
    let px = u64::from(u32::from_ne_bytes(pos.x.to_ne_bytes()));
    let py = u64::from(u32::from_ne_bytes(pos.y.to_ne_bytes()));
    let hash = px
        .wrapping_mul(2_654_435_761)
        .wrapping_add(py.wrapping_mul(1_013_904_223));
    #[allow(clippy::as_conversions)]
    let jitter = (hash % 31) as i16 - 15; // -15..+15

    // Clamp to within 15 of the anchor so adjacent areas can differ by at
    // most ~30 (each jittering in opposite directions from the same anchor).
    let anchor = i16::from(nearest.alignment);
    #[allow(clippy::as_conversions)]
    let raw = (anchor + jitter)
        .clamp(anchor - 15, anchor + 15)
        .clamp(1, 100) as u8;
    raw
}

fn manhattan(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// Pick a dead-end area near the zone seed closest to the player's alignment.
fn pick_dead_end_for_alignment(
    dead_ends: &[IVec2],
    zones: &[ZoneSeed],
    dominant: AreaAlignment,
    seed: u64,
) -> IVec2 {
    if dead_ends.is_empty() {
        return IVec2::ZERO;
    }
    if zones.is_empty() {
        return dead_ends[0];
    }

    // Find the zone seed closest to the player's alignment.
    let best_zone = zones
        .iter()
        .min_by_key(|z| z.alignment.abs_diff(dominant))
        .expect("zones is non-empty");

    // Pick the dead end closest to that zone seed, with a small random offset.
    let rng = lcg(seed.wrapping_add(0xDE_AD));
    let mut scored: Vec<(IVec2, i32)> = dead_ends
        .iter()
        .map(|&p| (p, manhattan(p, best_zone.pos)))
        .collect();
    scored.sort_by_key(|&(_, d)| d);

    // 80% closest, 20% second closest.
    let roll = rng % 100;
    if roll < 80 || scored.len() < 2 {
        scored[0].0
    } else {
        scored[1].0
    }
}
