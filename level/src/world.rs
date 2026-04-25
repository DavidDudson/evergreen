use std::collections::{BTreeSet, HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::area::{
    Area, AreaAlignment, AreaEvent, Direction, NpcKind, ALL_NPCS, MAP_HEIGHT, MAP_WIDTH,
};
use crate::portal::{pick_portal_kind, PortalPlacement};
use crate::terrain::Terrain;
use crate::water::{generate_water_bodies, WaterMap};

/// Maximum number of NPC encounters in the world.
const MAX_NPC_ENCOUNTERS: usize = 3;

/// Minimum Manhattan distance from origin for an NPC encounter.
const MIN_NPC_DISTANCE: i32 = 3;

/// Probability (out of 100) that an eligible area gets an NPC encounter.
const NPC_ENCOUNTER_CHANCE: u64 = 30;

/// Alignment ceiling for the city biome -- areas with alignment <= this are
/// treated as urban and reject road exits toward missing neighbours so a
/// grass buffer sits between the road and the beach. Tight (10) so
/// greenwood maps still grow toward their target area count when they
/// happen to sit on the coast.
const CITY_ALIGNMENT_MAX: u8 = 10;

/// Map sizing formula bounds. `MAP_AREAS_AT_MIN` areas at alignment 1,
/// `MAP_AREAS_AT_HI_ALIGN` areas at alignment `MAP_SIZE_PEAK_ALIGNMENT`,
/// clamped at `MAP_AREAS_HARD_CAP` for higher alignments.
pub const MAP_AREAS_AT_MIN: usize = 5;
const MAP_AREAS_AT_HI_ALIGN: usize = 50;
const MAP_SIZE_PEAK_ALIGNMENT: u8 = 30;
const MAP_AREAS_HARD_CAP: usize = 60;
/// Maximum number of regeneration attempts before accepting a small map.
const GENERATE_RETRY_CAP: usize = 10;

/// Default alignment for the bootstrap (root) map: light greenwood. The
/// player can portal to other biomes but the entry point sits in greenwood.
pub const ROOT_MAP_ALIGNMENT: AreaAlignment = 15;

/// Compute the target area count for a map of the given alignment.
/// Linear lerp between [`MAP_AREAS_AT_MIN`] @ alignment 1 and
/// [`MAP_AREAS_AT_HI_ALIGN`] @ alignment [`MAP_SIZE_PEAK_ALIGNMENT`], clamped
/// at [`MAP_AREAS_HARD_CAP`] for higher alignments.
pub fn map_area_count_target(alignment: AreaAlignment) -> usize {
    let a = u32::from(alignment.max(1));
    let lo = u32::try_from(MAP_AREAS_AT_MIN).unwrap_or(5);
    let hi = u32::try_from(MAP_AREAS_AT_HI_ALIGN).unwrap_or(50);
    let peak = u32::from(MAP_SIZE_PEAK_ALIGNMENT.max(2));
    let cap = MAP_AREAS_HARD_CAP;
    if a <= peak {
        let span = hi.saturating_sub(lo);
        let scaled = lo + (span * (a - 1)) / (peak - 1);
        usize::try_from(scaled).unwrap_or(MAP_AREAS_AT_MIN)
    } else {
        cap
    }
}

/// Identifier for a map within the multiverse. Phase 1 keeps a single root
/// map; phase 2 uses this to address portal targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapId(pub u32);

impl MapId {
    pub const ROOT: Self = Self(0);
}

/// Fired when the player crosses an area boundary and the current area changes.
#[derive(Message, Clone, Copy)]
pub struct AreaChanged {
    pub direction: Direction,
}

/// One generated map's state. A map has a uniform biome alignment and is
/// reached either as the root world or by stepping through a portal.
///
/// Phase 1: only the root map is exposed via [`WorldMap`]; phase 2 adds a
/// `Multiverse` resource holding many `Map`s keyed by [`MapId`].
#[derive(Resource)]
pub struct WorldMap {
    /// This map's identity within the multiverse.
    pub id: MapId,
    /// Areas generated for this map keyed by grid position.
    areas: HashMap<IVec2, Area>,
    /// Player's current area within this map.
    pub current: IVec2,
    /// RNG seed for this map.
    seed: u64,
    /// Uniform biome alignment -- every area in this map shares it.
    pub alignment: AreaAlignment,
    /// Target number of areas to grow this map to.
    pub area_count_target: usize,
    /// NPCs available for encounters, shuffled at creation.
    npc_pool: Vec<NpcKind>,
    /// How many NPC encounters have been placed so far.
    npc_count: usize,
    /// Areas the player has entered.
    visited: HashSet<IVec2>,
    /// Areas visible on the minimap (visited + their exit neighbors).
    revealed: HashSet<IVec2>,
    /// The dead-end area containing the level exit.
    pub exit_area: IVec2,
    /// Water tiles (ponds, hot springs, lakes, rivers, ocean) generated after terrain.
    pub water: WaterMap,
    /// Whether this map has an ocean surrounding its boundary.
    pub has_ocean: bool,
    /// At most one portal per map; placed in a dead-end area so the player
    /// has to seek it out. `None` for maps whose alignment doesn't match
    /// any portal kind's bridge range.
    pub portal: Option<PortalPlacement>,
}

impl WorldMap {
    /// Create the root map for a fresh game. Greenwood-aligned by default.
    pub fn new(seed: u64, _dominant_alignment: AreaAlignment) -> Self {
        Self::generate(MapId::ROOT, seed, ROOT_MAP_ALIGNMENT)
    }

    /// Generate a map with the specified id, seed and alignment. Used by
    /// phase 2 portal-target generation as well.
    ///
    /// Retries with a re-derived seed when the resulting graph has fewer
    /// than [`MAP_AREAS_AT_MIN`] areas (a degenerate single-room layout).
    /// Caps at [`GENERATE_RETRY_CAP`] attempts; after that the smallest
    /// map is accepted to avoid infinite loops.
    pub fn generate(id: MapId, seed: u64, alignment: AreaAlignment) -> Self {
        let mut attempt_seed = seed;
        for _ in 0..GENERATE_RETRY_CAP {
            let candidate = Self::try_generate(id, attempt_seed, alignment);
            if candidate.areas.len() >= MAP_AREAS_AT_MIN {
                return candidate;
            }
            attempt_seed = lcg(attempt_seed.wrapping_add(0x_F00D_BABE));
        }
        Self::try_generate(id, attempt_seed, alignment)
    }

    fn try_generate(id: MapId, seed: u64, alignment: AreaAlignment) -> Self {
        let target = map_area_count_target(alignment);

        // Shuffle NPC pool deterministically from seed.
        let mut npc_pool: Vec<NpcKind> = ALL_NPCS.to_vec();
        let mut rng = seed.wrapping_mul(7_046_029_254_386_353_131);
        for i in (1..npc_pool.len()).rev() {
            rng = lcg(rng);
            let j = usize::try_from(rng % u64::try_from(i + 1).expect("i+1 fits u64"))
                .expect("mod fits usize");
            npc_pool.swap(i, j);
        }

        // Seeded coin flip: ~60% of maps have an ocean border. City maps
        // (very small) almost always have one to feel coastal; deep
        // greenwood / darkwood usually don't.
        let has_ocean = match alignment {
            0..=15 => true,
            16..=40 => (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) % 100 < 60,
            _ => (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) % 100 < 25,
        };

        let mut map = Self {
            id,
            areas: HashMap::new(),
            current: IVec2::ZERO,
            seed,
            alignment,
            area_count_target: target,
            npc_pool,
            npc_count: 0,
            visited: HashSet::new(),
            revealed: HashSet::new(),
            exit_area: IVec2::ZERO,
            water: WaterMap::default(),
            has_ocean,
            portal: None,
        };

        // Seed the origin as a 4-way cross to bootstrap generation.
        let all_exits = BTreeSet::from([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]);
        let origin_seed = map.area_seed(IVec2::ZERO);
        let origin_area = Area::generate(
            all_exits,
            BTreeSet::new(),
            origin_seed,
            0,
            alignment,
            IVec2::ZERO,
        );
        map.areas.insert(IVec2::ZERO, origin_area);

        // Expand: keep generating neighbours until we either run out of new
        // exits to follow or hit the per-map area-count target.
        loop {
            if map.areas.len() >= map.area_count_target {
                break;
            }
            let positions: Vec<IVec2> = map.areas.keys().copied().collect();
            let before = map.areas.len();
            for pos in positions {
                if map.areas.len() >= map.area_count_target {
                    break;
                }
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

        // Pick start: the dead end closest to the origin so the player
        // begins on the map's edge and explores inward.
        let start = pick_start_dead_end(&dead_ends, seed);
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

        // Pick portal kind eligible for this map's alignment, then place it
        // in a dead-end area that isn't the start or exit. If only the
        // start+exit are available, fall back to the exit area; if no
        // dead-end at all, no portal this map.
        let kind = pick_portal_kind(alignment, seed.wrapping_add(0xC0FE_5A75));
        let portal_area = dead_ends
            .iter()
            .copied()
            .find(|&p| p != start && p != exit)
            .or(Some(exit))
            .unwrap_or(IVec2::ZERO);
        map.portal = Some(PortalPlacement {
            kind,
            area_pos: portal_area,
            tile_x: u32::from(MAP_WIDTH) / 2,
            tile_y: u32::from(MAP_HEIGHT) / 2,
        });
        // Override the portal area's NPC encounter to the portal kind's
        // signature NPC -- Cadwallader / Bloody Mary / Mother Gothel.
        if let Some(area) = map.areas.get_mut(&portal_area) {
            area.event = AreaEvent::NpcEncounter(kind.signature_npc());
        }

        // Water bodies generated last so flood-fill can use final terrain.
        map.water = generate_water_bodies(&map, seed);

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

    /// Returns the uniform biome alignment for any position in this map.
    /// Areas no longer vary their alignment within a map.
    pub fn alignment_at(&self, _pos: IVec2) -> AreaAlignment {
        self.alignment
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
        // Hard cap: stop generating once the map has reached its target.
        // The bootstrap loop already enforces this, but `transition()` can
        // also call us, so re-check here.
        if self.areas.len() >= self.area_count_target {
            // Still allow generation if this position is required by
            // existing exits -- otherwise the player would walk into a
            // missing area.
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
        let alignment = self.alignment;

        // City-aligned coastal areas must keep a grass buffer to the beach.
        // Forbid road exits toward any missing-neighbour direction (which
        // becomes ocean) so the road never abuts the sand band directly.
        if alignment <= CITY_ALIGNMENT_MAX && self.has_ocean {
            for dir in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                if self.areas.get(&(pos + dir.grid_offset())).is_none() {
                    forbidden.insert(dir);
                }
            }
        }

        // Cap-aware exit picking: when the map is at-or-past its area target,
        // forbid all optional (missing-neighbour) exits so we don't overgrow.
        if self.areas.len() >= self.area_count_target {
            for dir in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                if !required.contains(&dir) {
                    forbidden.insert(dir);
                }
            }
        }

        let mut area = Area::generate(required, forbidden, seed, area_count, alignment, pos);

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

fn manhattan(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// Pick the starting dead-end -- the one closest to the origin so the player
/// begins on the map's edge. Falls back to origin if there are no dead ends.
fn pick_start_dead_end(dead_ends: &[IVec2], seed: u64) -> IVec2 {
    if dead_ends.is_empty() {
        return IVec2::ZERO;
    }
    let mut sorted: Vec<IVec2> = dead_ends.to_vec();
    sorted.sort_by_key(|p| manhattan(*p, IVec2::ZERO));
    let rng = lcg(seed.wrapping_add(0xDE_AD));
    let roll = rng % 100;
    if roll < 80 || sorted.len() < 2 {
        sorted[0]
    } else {
        sorted[1]
    }
}
