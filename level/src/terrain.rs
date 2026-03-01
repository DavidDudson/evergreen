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

/// Wang corner tile index (0–15).
///
/// Bit ordering: NW=8, NE=4, SW=2, SE=1.
/// A bit is 1 (grass) if a majority (≥2) of the 4 tiles sharing that vertex
/// are grass. Grass wins on a 2-vs-2 tie.
pub fn wang_index(nw: bool, ne: bool, sw: bool, se: bool) -> u32 {
    (nw as u32) << 3 | (ne as u32) << 2 | (sw as u32) << 1 | (se as u32)
}

/// Maps Wang index (0–15) to the bevy_ecs_tilemap texture atlas index.
///
/// Filled in from the PixelLab tileset metadata after generation.
/// Placeholder: identity mapping (assumes 4-column sheet in Wang order).
pub const WANG_TO_ATLAS: [u32; 16] =
    [6, 7, 10, 9, 2, 11, 4, 15, 5, 14, 1, 8, 3, 0, 13, 12];

/// Deterministic tile hash for scenery placement.
///
/// `u32 → usize` widening is safe on all supported platforms (usize ≥ 32 bits).
#[allow(clippy::as_conversions)]
pub fn tile_hash(x: u32, y: u32, salt: u32) -> usize {
    let mut h = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(salt);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h = h ^ (h >> 16);
    h as usize
}
