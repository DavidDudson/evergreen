//! Wang-tileset autotiling for water bodies + sand.
//!
//! Each tileset is a 4x4 grid of 16 tiles, each tile indexed by a 4-bit
//! corner mask (NW=8, NE=4, SW=2, SE=1; bit set = "upper" / land, cleared =
//! "lower" / the body material). Metadata JSON exported by pixellab tells
//! us the `(corners → bounding_box)` mapping; we parse it once at startup
//! and build a `[u8; 16]` lookup table per tileset.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Tileset registry keys
// ---------------------------------------------------------------------------

pub const OCEAN_SAND: &str = "ocean_sand";
pub const OCEAN_DEEP_SHALLOW: &str = "ocean_deep_shallow";
pub const RIVER_DEEP_SHALLOW: &str = "river_deep_shallow";
pub const PIER_OCEAN: &str = "pier_ocean";
pub const SAND_GRASS: &str = "sand_grass";
pub const POND_GRASS: &str = "pond_grass";
pub const HOTSPRING_GRASS: &str = "hotspring_grass";
pub const RIVER_GRASS: &str = "river_grass";
pub const WATERFALL_GRASS: &str = "waterfall_grass";

// ---------------------------------------------------------------------------
// Metadata JSON shape
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TilesetMetadata {
    tileset_data: TilesetData,
}

#[derive(Debug, Deserialize)]
struct TilesetData {
    tiles: Vec<TileEntry>,
}

#[derive(Debug, Deserialize)]
struct TileEntry {
    corners: Corners,
    bounding_box: BoundingBox,
}

#[derive(Debug, Deserialize)]
struct Corners {
    #[serde(rename = "NE")]
    ne: String,
    #[serde(rename = "NW")]
    nw: String,
    #[serde(rename = "SE")]
    se: String,
    #[serde(rename = "SW")]
    sw: String,
}

#[derive(Debug, Deserialize)]
struct BoundingBox {
    x: u32,
    y: u32,
}

// ---------------------------------------------------------------------------
// Public tileset entry used by water/beach spawning
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct WangTileset {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    /// `lut[wang_mask]` -> atlas index 0..=15.
    pub lut: [usize; 16],
}

/// Shared resource exposing every loaded tileset, keyed by string key
/// (see the `OCEAN_SAND` / `POND_GRASS` / ... constants above).
#[derive(Resource, Default)]
pub struct WangTilesets(HashMap<&'static str, WangTileset>);

impl WangTilesets {
    pub fn get(&self, key: &'static str) -> &WangTileset {
        self.0
            .get(key)
            .unwrap_or_else(|| panic!("wang tileset '{key}' not registered"))
    }
}

// ---------------------------------------------------------------------------
// Wang mask encoding
// ---------------------------------------------------------------------------

/// Encode 4 corners (each `true` = lower / body material present) to 0..=15.
/// Bit order: NW=8, NE=4, SW=2, SE=1.
pub fn wang_mask(nw: bool, ne: bool, sw: bool, se: bool) -> u8 {
    (u8::from(nw) << 3) | (u8::from(ne) << 2) | (u8::from(sw) << 1) | u8::from(se)
}

fn parse_lut(json_bytes: &[u8]) -> [usize; 16] {
    let md: TilesetMetadata = serde_json::from_slice(json_bytes).expect("tileset json parses");
    let mut lut = [0usize; 16];
    for tile in md.tileset_data.tiles {
        // Pixellab "lower" represents the body (water/sand). We want a mask
        // where bit=1 means "body present at that corner", so "lower"->1.
        let mask = wang_mask(
            is_lower(&tile.corners.nw),
            is_lower(&tile.corners.ne),
            is_lower(&tile.corners.sw),
            is_lower(&tile.corners.se),
        );
        let col = tile.bounding_box.x / 16;
        let row = tile.bounding_box.y / 16;
        let idx = usize::try_from(row * 4 + col).unwrap_or(0);
        lut[usize::from(mask)] = idx;
    }
    lut
}

fn is_lower(s: &str) -> bool {
    s == "lower"
}

// ---------------------------------------------------------------------------
// Startup loader
// ---------------------------------------------------------------------------

const OCEAN_SAND_JSON: &[u8] = include_bytes!("../../assets/tilesets/ocean_sand.json");
const OCEAN_DEEP_SHALLOW_JSON: &[u8] =
    include_bytes!("../../assets/tilesets/ocean_deep_shallow.json");
const RIVER_DEEP_SHALLOW_JSON: &[u8] =
    include_bytes!("../../assets/tilesets/river_deep_shallow.json");
const PIER_OCEAN_JSON: &[u8] = include_bytes!("../../assets/tilesets/pier_ocean.json");
const SAND_GRASS_JSON: &[u8] = include_bytes!("../../assets/tilesets/sand_grass.json");
const POND_GRASS_JSON: &[u8] = include_bytes!("../../assets/tilesets/pond_grass.json");
const HOTSPRING_GRASS_JSON: &[u8] = include_bytes!("../../assets/tilesets/hotspring_grass.json");
const RIVER_GRASS_JSON: &[u8] = include_bytes!("../../assets/tilesets/river_grass.json");
const WATERFALL_GRASS_JSON: &[u8] = include_bytes!("../../assets/tilesets/waterfall_grass.json");

const TILESET_DEFS: &[(&str, &str, &[u8])] = &[
    (OCEAN_SAND, "tilesets/ocean_sand.webp", OCEAN_SAND_JSON),
    (
        OCEAN_DEEP_SHALLOW,
        "tilesets/ocean_deep_shallow.webp",
        OCEAN_DEEP_SHALLOW_JSON,
    ),
    (
        RIVER_DEEP_SHALLOW,
        "tilesets/river_deep_shallow.webp",
        RIVER_DEEP_SHALLOW_JSON,
    ),
    (PIER_OCEAN, "tilesets/pier_ocean.webp", PIER_OCEAN_JSON),
    (SAND_GRASS, "tilesets/sand_grass.webp", SAND_GRASS_JSON),
    (POND_GRASS, "tilesets/pond_grass.webp", POND_GRASS_JSON),
    (
        HOTSPRING_GRASS,
        "tilesets/hotspring_grass.webp",
        HOTSPRING_GRASS_JSON,
    ),
    (RIVER_GRASS, "tilesets/river_grass.webp", RIVER_GRASS_JSON),
    (
        WATERFALL_GRASS,
        "tilesets/waterfall_grass.webp",
        WATERFALL_GRASS_JSON,
    ),
];

pub fn init_wang_tilesets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        4,
        4,
        None,
        None,
    ));

    let mut map: HashMap<&'static str, WangTileset> = HashMap::with_capacity(TILESET_DEFS.len());
    for &(key, path, json) in TILESET_DEFS {
        map.insert(
            key,
            WangTileset {
                texture: asset_server.load(path),
                layout: layout.clone(),
                lut: parse_lut(json),
            },
        );
    }

    commands.insert_resource(WangTilesets(map));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wang_mask_packs_bits() {
        assert_eq!(wang_mask(false, false, false, false), 0);
        assert_eq!(wang_mask(true, true, true, true), 15);
        assert_eq!(wang_mask(true, false, false, false), 8);
        assert_eq!(wang_mask(false, false, false, true), 1);
    }

    #[test]
    fn ocean_sand_lut_parses() {
        let lut = parse_lut(OCEAN_SAND_JSON);
        // Every entry should have been populated (no all-zero slots unless
        // the tile genuinely sits at origin).
        let zero_count = lut.iter().filter(|&&v| v == 0).count();
        assert!(zero_count <= 1, "LUT has {zero_count} zero entries");
    }
}
