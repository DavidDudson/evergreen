use crate::terrain::Terrain;

pub const MAP_WIDTH: u16 = 30;
pub const MAP_HEIGHT: u16 = 16;

// Shared tileset edge indices (both grass and dirt use the same layout within their rows).
const EDGE_TL: u32 = 0;
const EDGE_T: u32 = 1;
const EDGE_TR: u32 = 2;
const EDGE_L: u32 = 11;
const EDGE_R: u32 = 13;
const EDGE_BL: u32 = 22;
const EDGE_B: u32 = 23;
const EDGE_BR: u32 = 24;

/// 30x16 tile map. G = grass, D = dirt.
/// Rows are top-to-bottom to match the visual layout in-game.
/// Plus-sign shape: 3-tile-wide dirt paths.
#[rustfmt::skip]
#[allow(clippy::as_conversions)] // const context: no From/Into available
const TILE_MAP: [&str; MAP_HEIGHT as usize] = [
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
    "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
    "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
    "GGGGGGGGGGGGGDDDGGGGGGGGGGGGGG",
];

/// Look up terrain by array row and column index directly.
pub fn terrain_at_row(row: usize, col: usize) -> Option<Terrain> {
    TILE_MAP
        .get(row)
        .and_then(|r| r.as_bytes().get(col))
        .and_then(|&b| b.try_into().ok())
}

pub fn terrain_at(x: u32, y: u32) -> Option<Terrain> {
    let x: u16 = x.try_into().ok()?;
    let y: u16 = y.try_into().ok()?;
    if x >= MAP_WIDTH || y >= MAP_HEIGHT {
        return None;
    }
    let row = usize::from(MAP_HEIGHT - 1 - y);
    TILE_MAP[row].as_bytes()[usize::from(x)].try_into().ok()
}

/// Determine the tile index in the combined sprite sheet for a cell.
pub fn tile_index(x: u32, y: u32, terrain: Terrain) -> u32 {
    let edge_l = x.checked_sub(1).and_then(|nx| terrain_at(nx, y)) != Some(terrain);
    let edge_r = terrain_at(x + 1, y) != Some(terrain);
    let edge_b = y.checked_sub(1).and_then(|ny| terrain_at(x, ny)) != Some(terrain);
    let edge_t = terrain_at(x, y + 1) != Some(terrain);

    let relative = match (edge_l, edge_r, edge_b, edge_t) {
        (true, _, true, _) => EDGE_BL,
        (_, true, true, _) => EDGE_BR,
        (true, _, _, true) => EDGE_TL,
        (_, true, _, true) => EDGE_TR,
        (true, _, _, _) => EDGE_L,
        (_, true, _, _) => EDGE_R,
        (_, _, true, _) => EDGE_B,
        (_, _, _, true) => EDGE_T,
        _ => return terrain.fill(x, y),
    };

    terrain.offset() + relative
}
