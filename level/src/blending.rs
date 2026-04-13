use bevy::math::IVec2;

use crate::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use crate::world::WorldMap;

/// Horizontal blend width in tiles (20% of 32).
const BLEND_W: u32 = 6;
/// Vertical blend width in tiles (20% of 18).
const BLEND_H: u32 = 4;

/// Result of a blend calculation for a single tile.
pub struct BlendResult {
    /// Effective alignment after blending (lerped toward neighbor).
    pub alignment: u8,
    /// Raw blend factor 0.0 (no neighbor influence) to ~0.5 (edge of area).
    pub factor: f32,
    /// The neighbor's alignment, if any neighbor influences this tile.
    pub neighbor_alignment: Option<u8>,
}

/// Compute the effective biome alignment for a tile at `(x, y)` within an area,
/// blending toward neighbor areas near the borders.
///
/// Returns the area's own alignment if the tile is outside all blend zones
/// or if no neighbor exists in the relevant direction.
pub fn blended_alignment(
    area_alignment: u8,
    x: u32,
    y: u32,
    area_pos: IVec2,
    world: &WorldMap,
) -> u8 {
    blend_at(area_alignment, x, y, area_pos, world).alignment
}

/// Full blend calculation returning alignment, factor, and neighbor info.
pub fn blend_at(
    area_alignment: u8,
    x: u32,
    y: u32,
    area_pos: IVec2,
    world: &WorldMap,
) -> BlendResult {
    let w = u32::from(MAP_WIDTH);
    let h = u32::from(MAP_HEIGHT);

    // Find the strongest neighbor influence.
    let mut best_t: f32 = 0.0;
    let mut best_neighbor_align: Option<u8> = None;

    // West edge
    if x < BLEND_W {
        #[allow(clippy::as_conversions)]
        let t = 1.0 - (x as f32 / BLEND_W as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::West, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // East edge
    let dist_right = w.saturating_sub(1).saturating_sub(x);
    if dist_right < BLEND_W {
        #[allow(clippy::as_conversions)]
        let t = 1.0 - (dist_right as f32 / BLEND_W as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::East, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // South edge
    if y < BLEND_H {
        #[allow(clippy::as_conversions)]
        let t = 1.0 - (y as f32 / BLEND_H as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::South, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    // North edge
    let dist_top = h.saturating_sub(1).saturating_sub(y);
    if dist_top < BLEND_H {
        #[allow(clippy::as_conversions)]
        let t = 1.0 - (dist_top as f32 / BLEND_H as f32);
        if let Some(align) = neighbor_alignment(area_pos, Direction::North, world) {
            if t > best_t {
                best_t = t;
                best_neighbor_align = Some(align);
            }
        }
    }

    let factor = best_t * 0.5;
    let alignment = match best_neighbor_align {
        Some(neighbor) => lerp_alignment(area_alignment, neighbor, factor),
        None => area_alignment,
    };

    BlendResult {
        alignment,
        factor,
        neighbor_alignment: best_neighbor_align,
    }
}

fn neighbor_alignment(area_pos: IVec2, dir: Direction, world: &WorldMap) -> Option<u8> {
    let neighbor_pos = area_pos + dir.grid_offset();
    world.get_area(neighbor_pos).map(|a| a.alignment)
}

#[allow(clippy::as_conversions)]
fn lerp_alignment(a: u8, b: u8, t: f32) -> u8 {
    let result = f32::from(a) + (f32::from(b) - f32::from(a)) * t;
    result.round().clamp(1.0, 100.0) as u8
}
