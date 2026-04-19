//! Water bodies: ponds, hot springs, and multi-area lakes.
//!
//! Each water "tile" sits on top of a grass tile at a `(area_pos, local_xy)`
//! coordinate. A `WaterMap` owned by `WorldMap` stores the set; spawners turn
//! map entries into world-space sprites + colliders.
//!
//! Generation runs once after the rest of world generation:
//!   * Per-area pond seeds -- small (<= 8 tiles), single-area blobs.
//!   * Darkwood-biased hot-spring seeds -- same shape, different tint.
//!   * A handful of lake seeds flood-fill up to `LAKE_MAX_TILES` across area
//!     boundaries via `WorldMap::terrain_at_extended`.
//!
//! Puddles (weather-driven) are NOT stored here; they're spawned directly by
//! the weather system.

use std::collections::{HashMap, VecDeque};

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use models::layer::Layer;
use models::scenery::{Scenery, SceneryCollider};

use crate::area::{Area, Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::spawning::{area_world_offset, TILE_SIZE_PX};
use crate::terrain::Terrain;
use crate::world::WorldMap;

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
}

/// `(area grid position, local tile within area)` key for a single water tile.
pub type WaterKey = (IVec2, UVec2);

/// Marker on each spawned water-tile entity.
#[derive(Component)]
pub struct WaterTile {
    pub kind: WaterKind,
}

/// Marker added to every water sprite. A shared animation system alpha-pulses
/// all water in lockstep so wang-tiled edges stay seamless.
#[derive(Component)]
pub struct AnimatedWater;

/// All generated water tiles in the world.
#[derive(Default, Debug)]
pub struct WaterMap {
    tiles: HashMap<WaterKey, WaterKind>,
    /// Tiles where a stepping stone sits on top of the water (walkable).
    stones: std::collections::HashSet<WaterKey>,
    /// Sand tiles inland of ocean tiles. Not water -- but stored here so
    /// spawn systems only need a single map to query.
    sand: std::collections::HashSet<WaterKey>,
}

impl WaterMap {
    pub fn get(&self, area_pos: IVec2, local: UVec2) -> Option<WaterKind> {
        self.tiles.get(&(area_pos, local)).copied()
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
        TILE_NEIGHBOURS_4
            .iter()
            .any(|&(dx, dy)| neighbour_key(area_pos, local, dx, dy).is_some_and(|k| !self.tiles.contains_key(&k)))
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
}

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

/// 4-neighbour deltas for flood-fill (N/S/E/W).
const TILE_NEIGHBOURS_4: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

/// Stepping-stone sprite (placed over walkable river tiles).
const STONE_SPRITE: &str = "sprites/scenery/ponds/stepping_stone.webp";

/// Rendered sprite size in pixels (wider than a tile so neighbours overlap
/// and the pond outline looks organic).
const WATER_SPRITE_SIZE_PX: f32 = 20.0;

/// Pixel dimensions of one map area (wraps `MAP_W_PX`/`MAP_H_PX`).
#[allow(clippy::as_conversions)]
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

/// Collider half-extent (slightly smaller than the tile so the player can
/// squeeze along shorelines without getting stuck).
const WATER_COLLIDER_HALF_PX: f32 = 7.0;

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
        if let Some(seed_key) = pick_seed_tile(world, &mut rng, 30..=75) {
            flood_fill(&mut tiles, world, seed_key, WaterKind::Lake, LAKE_MAX_TILES, &mut rng);
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
        let pond_threshold = if (35..=65).contains(&align) {
            POND_CHANCE + POND_CHANCE_GREENWOOD
        } else {
            POND_CHANCE
        };
        if align >= POND_ALIGNMENT_MIN && roll < pond_threshold {
            rng = lcg(rng);
            if let Some(seed_key) = pick_grass_tile_in_area(world, pos, &mut rng) {
                flood_fill(&mut tiles, world, seed_key, WaterKind::Plain, POND_MAX_TILES, &mut rng);
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
        stones: std::collections::HashSet::new(),
        sand: std::collections::HashSet::new(),
    };
    generate_rivers(&mut map, world, &mut rng);
    if world.has_ocean {
        generate_ocean_and_sand(&mut map, world);
    }
    map
}

// ---------------------------------------------------------------------------
// Ocean / sand
// ---------------------------------------------------------------------------

fn generate_ocean_and_sand(map: &mut WaterMap, world: &WorldMap) {
    let width = u32::from(MAP_WIDTH);
    let height = u32::from(MAP_HEIGHT);
    for pos in world.area_positions() {
        let missing = missing_neighbours(world, pos);
        if missing.is_empty() {
            continue;
        }
        for y in 0..height {
            for x in 0..width {
                let dist = edge_distance_to_missing(x, y, width, height, &missing);
                let Some(dist) = dist else {
                    continue;
                };
                let local = UVec2::new(x, y);
                let key = (pos, local);
                if dist < OCEAN_DEPTH {
                    // Ocean tiles overwrite anything except stepping stones.
                    if !map.stones.contains(&key) {
                        map.tiles.insert(key, WaterKind::Ocean);
                    }
                } else if dist < OCEAN_DEPTH + SAND_DEPTH && !map.tiles.contains_key(&key) {
                    map.sand.insert(key);
                }
            }
        }
    }
}

/// Directions (from a given area) whose neighbour area does not exist --
/// those become world-edge facing and are where ocean spills outward.
fn missing_neighbours(world: &WorldMap, pos: IVec2) -> Vec<Direction> {
    [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ]
    .into_iter()
    .filter(|d| world.get_area(pos + d.grid_offset()).is_none())
    .collect()
}

/// Shortest distance (tiles) from `(x,y)` to the nearest edge that faces a
/// missing neighbour. Returns `None` when there's no such edge.
fn edge_distance_to_missing(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    missing: &[Direction],
) -> Option<u32> {
    missing
        .iter()
        .map(|dir| match dir {
            Direction::North => height.saturating_sub(1).saturating_sub(y),
            Direction::South => y,
            Direction::East => width.saturating_sub(1).saturating_sub(x),
            Direction::West => x,
        })
        .min()
}

// ---------------------------------------------------------------------------
// River generation
// ---------------------------------------------------------------------------

/// River flow axis. Always perpendicular to the area's road exits.
#[derive(Debug, Clone, Copy)]
enum RiverAxis {
    NorthSouth,
    EastWest,
}

/// Per-area chance (out of 100) to add a river when the axis is valid.
const RIVER_CHANCE: u64 = 45;

/// Ocean band depth (tiles) along the outermost edge of an edge-facing area.
const OCEAN_DEPTH: u32 = 4;
/// Sand band depth (tiles) inland of the ocean band.
const SAND_DEPTH: u32 = 2;

/// Central 3x3 block where road crosses river -- stones go here.
const CROSSING_COL_START: u32 = 14;
const CROSSING_COL_END: u32 = 16;
const CROSSING_ROW_START: u32 = 7;
const CROSSING_ROW_END: u32 = 9;

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

fn generate_rivers(map: &mut WaterMap, world: &WorldMap, rng: &mut u64) {
    for pos in world.area_positions() {
        let Some(area) = world.get_area(pos) else {
            continue;
        };
        let Some(axis) = river_axis_for(area) else {
            continue;
        };
        *rng = lcg(*rng);
        if *rng % 100 >= RIVER_CHANCE {
            continue;
        }
        carve_river(map, world, pos, axis);
    }
}

fn carve_river(map: &mut WaterMap, world: &WorldMap, pos: IVec2, axis: RiverAxis) {
    let waterfall_at_north = matches!(axis, RiverAxis::NorthSouth)
        && world.get_area(pos + Direction::North.grid_offset()).is_none();

    let iter: Vec<(u32, u32)> = match axis {
        RiverAxis::NorthSouth => (0..u32::from(MAP_HEIGHT))
            .flat_map(|y| (CROSSING_COL_START..=CROSSING_COL_END).map(move |x| (x, y)))
            .collect(),
        RiverAxis::EastWest => (0..u32::from(MAP_WIDTH))
            .flat_map(|x| (CROSSING_ROW_START..=CROSSING_ROW_END).map(move |y| (x, y)))
            .collect(),
    };

    for (x, y) in iter {
        let key = (pos, UVec2::new(x, y));
        // Skip if there's already a pond/lake here (river joins body naturally).
        if map.tiles.contains_key(&key) {
            continue;
        }
        // River can run through grass; any other terrain (dirt path etc.) also OK.
        // Top ~20% of the area becomes waterfall when the area faces the
        // north world edge (no neighbour beyond).
        let waterfall_start_row = u32::from(MAP_HEIGHT) * 4 / 5;
        let kind = if waterfall_at_north && y >= waterfall_start_row {
            WaterKind::Waterfall
        } else {
            match axis {
                RiverAxis::NorthSouth => WaterKind::RiverNS,
                RiverAxis::EastWest => WaterKind::RiverEW,
            }
        };
        map.tiles.insert(key, kind);

        // Central 3x3 crossing gets stepping stones (walkable).
        if (CROSSING_COL_START..=CROSSING_COL_END).contains(&x)
            && (CROSSING_ROW_START..=CROSSING_ROW_END).contains(&y)
            && !matches!(kind, WaterKind::Waterfall)
        {
            map.stones.insert(key);
        }
    }
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

/// Step one tile in cardinal direction from `(area_pos, local)`. If we walk
/// off the area's `MAP_WIDTH x MAP_HEIGHT` grid, the returned key is in the
/// neighbouring area at the wrapped local coord. Returns `None` only when
/// `local` underflows negative for a start tile already at 0.
fn neighbour_key(area_pos: IVec2, local: UVec2, dx: i32, dy: i32) -> Option<WaterKey> {
    let w = i32::from(MAP_WIDTH);
    let h = i32::from(MAP_HEIGHT);
    let lx = i32::try_from(local.x).ok()? + dx;
    let ly = i32::try_from(local.y).ok()? + dy;

    let area_dx = if lx < 0 { -1 } else if lx >= w { 1 } else { 0 };
    let area_dy = if ly < 0 { -1 } else if ly >= h { 1 } else { 0 };

    let new_area = area_pos + IVec2::new(area_dx, area_dy);
    let new_lx = u32::try_from(lx - area_dx * w).ok()?;
    let new_ly = u32::try_from(ly - area_dy * h).ok()?;
    Some((new_area, UVec2::new(new_lx, new_ly)))
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
// Spawning
// ---------------------------------------------------------------------------

/// Spawn wang-tiled water sprites for every tile position in `area_pos`
/// that has at least one water-owning vertex. Transition tiles on land
/// adjacent to water render automatically.
pub fn spawn_area_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    wang: &crate::wang::WangTilesets,
    world: &WorldMap,
    area_pos: IVec2,
) {
    let base = area_world_offset(area_pos);
    let base_offset_x = base.x - MAP_W_PX / 2.0;
    let base_offset_y = base.y - MAP_H_PX / 2.0;
    let tile_px = f32::from(TILE_SIZE_PX);

    // Each kind family uses one wang tileset and one same-kind predicate.
    let families: [(&crate::wang::WangTileset, fn(WaterKind) -> bool, WaterKind); 5] = [
        (
            &wang.pond_grass,
            |k| matches!(k, WaterKind::Plain | WaterKind::Lake),
            WaterKind::Plain,
        ),
        (&wang.hotspring_grass, |k| k == WaterKind::HotSpring, WaterKind::HotSpring),
        (
            &wang.river_grass,
            |k| matches!(k, WaterKind::RiverNS | WaterKind::RiverEW),
            WaterKind::RiverNS,
        ),
        (&wang.waterfall_grass, |k| k == WaterKind::Waterfall, WaterKind::Waterfall),
        (&wang.ocean_sand, |k| k == WaterKind::Ocean, WaterKind::Ocean),
    ];

    for y in 0..u32::from(MAP_HEIGHT) {
        for x in 0..u32::from(MAP_WIDTH) {
            for &(tileset, predicate, marker_kind) in &families {
                let mask = kind_mask(&world.water, area_pos, x, y, predicate);
                if mask == 0 {
                    continue;
                }
                let world_x = base_offset_x + f32::from(u16::try_from(x).unwrap_or(0)) * tile_px
                    + tile_px / 2.0;
                let world_y = base_offset_y + f32::from(u16::try_from(y).unwrap_or(0)) * tile_px
                    + tile_px / 2.0;
                let atlas_idx = tileset.lut[usize::from(mask)];
                let local = UVec2::new(x, y);
                let is_center = mask == 0b1111;
                let has_stone = world.water.has_stone(area_pos, local);
                let z = Layer::Tilemap.z_f32() + 0.4 + f32::from(u16::try_from(mask).unwrap_or(0)) * 0.001;
                let mut entity = commands.spawn((
                    WaterTile { kind: marker_kind },
                    AnimatedWater,
                    Scenery,
                    Sprite {
                        image: tileset.texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: tileset.layout.clone(),
                            index: atlas_idx,
                        }),
                        custom_size: Some(Vec2::splat(tile_px)),
                        ..default()
                    },
                    Transform::from_xyz(world_x, world_y, z),
                ));
                // Only fully-water tiles block the player. Transition (partial)
                // tiles are walkable so players can step onto shore edges.
                if is_center && !has_stone {
                    entity.insert(SceneryCollider {
                        half_extents: Vec2::splat(WATER_COLLIDER_HALF_PX),
                        center_offset: Vec2::ZERO,
                    });
                }
            }
        }
    }

    // Stepping stones rendered on top of the river water at crossings.
    for local in world.water.stones_in_area(area_pos) {
        let world_x = base_offset_x + f32::from(u16::try_from(local.x).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        let world_y = base_offset_y + f32::from(u16::try_from(local.y).unwrap_or(0)) * tile_px
            + tile_px / 2.0;
        commands.spawn((
            SteppingStone,
            Scenery,
            Sprite {
                image: asset_server.load(STONE_SPRITE),
                custom_size: Some(Vec2::splat(WATER_SPRITE_SIZE_PX)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, Layer::Tilemap.z_f32() + 0.7),
        ));
    }
}

/// Marker for stepping-stone sprites. Player systems use this to trigger the
/// hop-bob animation while a player is standing on a stone.
#[derive(Component)]
pub struct SteppingStone;

/// Wang corner mask for a tile at `(x, y)` given a same-kind predicate.
/// Returns 0 when no vertex of this tile is adjacent to a matching water
/// tile. Bit order matches `wang::wang_mask`.
fn kind_mask(
    water: &WaterMap,
    area_pos: IVec2,
    x: u32,
    y: u32,
    same: fn(WaterKind) -> bool,
) -> u8 {
    let nw = vertex_is_kind(water, area_pos, x, y + 1, same);
    let ne = vertex_is_kind(water, area_pos, x + 1, y + 1, same);
    let sw = vertex_is_kind(water, area_pos, x, y, same);
    let se = vertex_is_kind(water, area_pos, x + 1, y, same);
    crate::wang::wang_mask(nw, ne, sw, se)
}

/// True if any of the (up to 4) tiles touching vertex `(vx, vy)` matches
/// `same`. Vertices land on the integer grid; a vertex at `(vx, vy)` is
/// shared by tiles `(vx-1, vy-1)`, `(vx, vy-1)`, `(vx-1, vy)`, `(vx, vy)`.
fn vertex_is_kind(
    water: &WaterMap,
    area_pos: IVec2,
    vx: u32,
    vy: u32,
    same: fn(WaterKind) -> bool,
) -> bool {
    for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let tile_x = i32::try_from(vx).unwrap_or(0) + dx;
        let tile_y = i32::try_from(vy).unwrap_or(0) + dy;
        if let Some(tile_key) = neighbour_key(area_pos, UVec2::new(0, 0), tile_x, tile_y) {
            if let Some(kind) = water.get(tile_key.0, tile_key.1) {
                if same(kind) {
                    return true;
                }
            }
        }
    }
    false
}

/// Wang mask for sand tiles at `(x, y)` in `area_pos`.
pub fn sand_mask(water: &WaterMap, area_pos: IVec2, x: u32, y: u32) -> u8 {
    let nw = sand_vertex(water, area_pos, x, y + 1);
    let ne = sand_vertex(water, area_pos, x + 1, y + 1);
    let sw = sand_vertex(water, area_pos, x, y);
    let se = sand_vertex(water, area_pos, x + 1, y);
    crate::wang::wang_mask(nw, ne, sw, se)
}

fn sand_vertex(water: &WaterMap, area_pos: IVec2, vx: u32, vy: u32) -> bool {
    for (dx, dy) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let tile_x = i32::try_from(vx).unwrap_or(0) + dx;
        let tile_y = i32::try_from(vy).unwrap_or(0) + dy;
        if let Some(tile_key) = neighbour_key(area_pos, UVec2::new(0, 0), tile_x, tile_y) {
            if water.has_sand(tile_key.0, tile_key.1) {
                return true;
            }
        }
    }
    false
}

/// Per-frame system: pulse every water sprite's alpha in lockstep so wang-tiled
/// water bodies keep seamless edges. No per-tile phase (would reveal seams)
/// and no scale animation (would gap between neighbours).
pub fn animate_water_surface(time: Res<Time>, mut query: Query<&mut Sprite, With<AnimatedWater>>) {
    const FREQ_HZ: f32 = 0.55;
    const ALPHA_AMPLITUDE: f32 = 0.06;
    let s = (time.elapsed_secs() * FREQ_HZ * std::f32::consts::TAU).sin();
    let alpha = 1.0 - ALPHA_AMPLITUDE + s.abs() * ALPHA_AMPLITUDE;
    for mut sprite in &mut query {
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

/// Despawn every water tile on world teardown.
pub fn despawn_water(mut commands: Commands, q: Query<Entity, With<WaterTile>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Despawn every stepping stone on world teardown.
pub fn despawn_stones(mut commands: Commands, q: Query<Entity, With<SteppingStone>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// RNG
// ---------------------------------------------------------------------------

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
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
        let key = neighbour_key(IVec2::ZERO, UVec2::new(u32::from(MAP_WIDTH) - 1, 5), 1, 0).unwrap();
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
