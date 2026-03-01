/// Columns per row in the combined terrain.png sprite sheet.
const SHEET_COLS: u32 = 11;

/// Dirt tiles start at this row in the combined sheet (grass rows 0-6, dirt rows 7-13).
const DIRT_OFFSET: u32 = 7 * SHEET_COLS;

const GRASS_FILL: [u32; 12] = [55, 56, 57, 58, 59, 60, 66, 67, 68, 69, 70, 71];
const DIRT_FILL: [u32; 6] = [55, 56, 57, 66, 67, 68];

/// Terrain types that can appear in the tile map.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    Dirt,
}

impl TryFrom<u8> for Terrain {
    type Error = u8;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        match c {
            b'G' => Ok(Terrain::Grass),
            b'D' => Ok(Terrain::Dirt),
            _ => Err(c),
        }
    }
}

impl Terrain {
    /// Base offset into the combined sprite sheet for this terrain.
    pub fn offset(self) -> u32 {
        match self {
            Terrain::Grass => 0,
            Terrain::Dirt => DIRT_OFFSET,
        }
    }

    pub fn fill(self, x: u32, y: u32) -> u32 {
        let relative = match self {
            Terrain::Grass => GRASS_FILL[tile_hash(x, y, 0) % GRASS_FILL.len()],
            Terrain::Dirt => DIRT_FILL[tile_hash(x, y, 7) % DIRT_FILL.len()],
        };
        self.offset() + relative
    }
}

/// Simple deterministic hash for tile variation.
///
/// Returns `usize` for direct use as an array index.
/// The `u32 -> usize` widening has no `From` impl in std, but is
/// safe on all supported platforms (usize >= 32 bits).
#[allow(clippy::as_conversions)]
fn tile_hash(x: u32, y: u32, salt: u32) -> usize {
    let mut h = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(salt);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    h as usize
}
