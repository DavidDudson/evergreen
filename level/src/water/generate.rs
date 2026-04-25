//! Procedural water-body generation: ponds, hot springs, and lakes via
//! flood-fill. Rivers + ocean live in sibling modules; this file is the
//! orchestrator that calls into them.

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::math::{IVec2, UVec2};

use crate::area::{MAP_HEIGHT, MAP_WIDTH};
use crate::terrain::Terrain;
use crate::world::WorldMap;

use super::depth::classify_depths;
use super::ocean::generate_ocean_and_sand;
use super::pier::generate_piers;
use super::rivers::generate_rivers;
use super::tiles::{neighbour_key, WaterKey, WaterKind, WaterMap, TILE_NEIGHBOURS_4};

// ---------------------------------------------------------------------------
// Generation tuning
// ---------------------------------------------------------------------------

/// Max tiles in a per-area pond blob.
const POND_MAX_TILES: usize = 7;
/// Max tiles in a hot-spring blob.
const HOT_SPRING_MAX_TILES: usize = 5;
/// Max tiles in a lake (can cross area boundaries).
const LAKE_MAX_TILES: usize = 40;
/// Branch probability (0-100) for each flood-fill neighbour.
const FILL_BRANCH_CHANCE: u64 = 55;

/// Per-area chance (out of 100) to seed a plain pond.
const POND_CHANCE: u64 = 35;
/// Extra chance in greenwood areas.
const POND_CHANCE_GREENWOOD: u64 = 20;
/// Per-area chance (out of 100) to seed a hot spring in darkwood.
const HOT_SPRING_CHANCE: u64 = 25;
/// Alignment threshold above which hot springs are considered.
const HOT_SPRING_ALIGNMENT_MIN: u8 = 70;
/// Alignment threshold above which plain ponds lose eligibility (city).
const POND_ALIGNMENT_MIN: u8 = 20;
/// Number of lake seeds attempted globally per world.
const LAKE_SEED_COUNT: usize = 2;

const ALIGNMENT_GREENWOOD_LO: u8 = 35;
const ALIGNMENT_GREENWOOD_HI: u8 = 65;
const LAKE_SEED_ALIGNMENT_LO: u8 = 30;
const LAKE_SEED_ALIGNMENT_HI: u8 = 75;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate all pond / hot-spring / lake tiles for a freshly-built `WorldMap`.
pub fn generate_water_bodies(world: &WorldMap, seed: u64) -> WaterMap {
    let mut tiles: HashMap<WaterKey, WaterKind> = HashMap::new();
    let mut rng = seed.wrapping_mul(0xA5A5_5A5A_A5A5_5A5A) ^ 0xDEAD_BEEF_CAFE_BABE;

    // Lakes first (they grab the biggest area budgets; per-area bodies fill
    // remaining grass). Seed in greenwood or neutral areas.
    for _ in 0..LAKE_SEED_COUNT {
        rng = lcg(rng);
        if let Some(seed_key) = pick_seed_tile(
            world,
            &mut rng,
            LAKE_SEED_ALIGNMENT_LO..=LAKE_SEED_ALIGNMENT_HI,
        ) {
            flood_fill(
                &mut tiles,
                world,
                seed_key,
                WaterKind::Lake,
                LAKE_MAX_TILES,
                &mut rng,
            );
        }
    }

    // Per-area ponds + hot springs.
    for pos in world.area_positions() {
        let Some(area) = world.get_area(pos) else {
            continue;
        };
        let align = area.alignment;

        rng = lcg(rng);
        let roll = rng % 100;
        let pond_threshold = if (ALIGNMENT_GREENWOOD_LO..=ALIGNMENT_GREENWOOD_HI).contains(&align) {
            POND_CHANCE + POND_CHANCE_GREENWOOD
        } else {
            POND_CHANCE
        };
        if align >= POND_ALIGNMENT_MIN && roll < pond_threshold {
            rng = lcg(rng);
            if let Some(seed_key) = pick_grass_tile_in_area(world, pos, &mut rng) {
                flood_fill(
                    &mut tiles,
                    world,
                    seed_key,
                    WaterKind::Plain,
                    POND_MAX_TILES,
                    &mut rng,
                );
            }
        }

        if align >= HOT_SPRING_ALIGNMENT_MIN {
            rng = lcg(rng);
            if rng % 100 < HOT_SPRING_CHANCE {
                rng = lcg(rng);
                if let Some(seed_key) = pick_grass_tile_in_area(world, pos, &mut rng) {
                    flood_fill(
                        &mut tiles,
                        world,
                        seed_key,
                        WaterKind::HotSpring,
                        HOT_SPRING_MAX_TILES,
                        &mut rng,
                    );
                }
            }
        }
    }

    let mut map = WaterMap {
        tiles,
        depths: HashMap::new(),
        stones: HashSet::new(),
        sand: HashSet::new(),
        piers: HashSet::new(),
    };
    generate_rivers(&mut map, world, &mut rng);
    if world.has_ocean {
        generate_ocean_and_sand(&mut map, world);
    }
    classify_depths(&mut map);
    generate_piers(&mut map, world, &mut rng);
    map
}

// ---------------------------------------------------------------------------
// Flood fill
// ---------------------------------------------------------------------------

fn flood_fill(
    tiles: &mut HashMap<WaterKey, WaterKind>,
    world: &WorldMap,
    start: WaterKey,
    kind: WaterKind,
    max_tiles: usize,
    rng: &mut u64,
) {
    let mut queue: VecDeque<WaterKey> = VecDeque::from([start]);
    let mut placed = 0usize;

    while let Some(key) = queue.pop_front() {
        if placed >= max_tiles {
            break;
        }
        if tiles.contains_key(&key) {
            continue;
        }
        if !is_grass(world, key) {
            continue;
        }
        tiles.insert(key, kind);
        placed += 1;

        for &(dx, dy) in &TILE_NEIGHBOURS_4 {
            let Some(nbr) = neighbour_key(key.0, key.1, dx, dy) else {
                continue;
            };
            *rng = lcg(*rng);
            if *rng % 100 < FILL_BRANCH_CHANCE {
                queue.push_back(nbr);
            }
        }
    }
}

fn is_grass(world: &WorldMap, key: WaterKey) -> bool {
    let (area_pos, local) = key;
    let lx = i32::try_from(local.x).unwrap_or(i32::MAX);
    let ly = i32::try_from(local.y).unwrap_or(i32::MAX);
    matches!(
        world.terrain_at_extended(area_pos, lx, ly),
        Some(Terrain::Grass)
    )
}

// ---------------------------------------------------------------------------
// Seed picking
// ---------------------------------------------------------------------------

fn pick_grass_tile_in_area(world: &WorldMap, pos: IVec2, rng: &mut u64) -> Option<WaterKey> {
    let area = world.get_area(pos)?;
    for _ in 0..20 {
        *rng = lcg(*rng);
        let lx = u32::try_from(*rng % u64::from(MAP_WIDTH)).ok()?;
        *rng = lcg(*rng);
        let ly = u32::try_from(*rng % u64::from(MAP_HEIGHT)).ok()?;
        if area.terrain_at(lx, ly) == Some(Terrain::Grass) {
            return Some((pos, UVec2::new(lx, ly)));
        }
    }
    None
}

fn pick_seed_tile(
    world: &WorldMap,
    rng: &mut u64,
    alignment_range: std::ops::RangeInclusive<u8>,
) -> Option<WaterKey> {
    let candidates: Vec<IVec2> = world
        .area_positions()
        .into_iter()
        .filter(|p| {
            world
                .get_area(*p)
                .is_some_and(|a| alignment_range.contains(&a.alignment))
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    *rng = lcg(*rng);
    let idx = usize::try_from(*rng % u64::try_from(candidates.len()).ok()?).ok()?;
    pick_grass_tile_in_area(world, candidates[idx], rng)
}

// ---------------------------------------------------------------------------
// RNG (also used by sibling river / ocean modules)
// ---------------------------------------------------------------------------

pub(super) fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
