//! Shared hashing + viewport-offset helpers used by every particle module.

use models::weather::WeatherKind;

/// Camera viewport half-width for particle spawning (pixels).
pub const VIEWPORT_HALF_W_PX: f32 = 280.0;
/// Camera viewport half-height for particle spawning (pixels).
pub const VIEWPORT_HALF_H_PX: f32 = 160.0;

/// Default alignment when the current area is unknown.
pub const DEFAULT_ALIGNMENT: u8 = 50;

/// Convert a fractional particle count to an integer, probabilistically rounding up.
pub fn fractional_to_count(fractional: f32, seed: u32) -> u32 {
    #[allow(clippy::as_conversions)]
    let whole = fractional as u32;
    let remainder = fractional - f32::from(u16::try_from(whole).unwrap_or(u16::MAX));
    let extra = if hash_frac(seed) < remainder { 1 } else { 0 };
    whole + extra
}

/// Hash a u32 seed into a float in [-range, +range].
pub fn hash_f32(seed: u32, range: f32) -> f32 {
    (hash_frac(seed) * 2.0 - 1.0) * range
}

/// Hash a u32 seed into a float in [0.0, 1.0).
pub fn hash_frac(seed: u32) -> f32 {
    let h = seed.wrapping_mul(374_761_393).wrapping_add(668_265_263);
    let h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    let h = h ^ (h >> 16);
    #[allow(clippy::as_conversions)]
    let frac = (h % 10000) as f32 / 10000.0;
    frac
}

/// Convert an f32 to a deterministic u32 seed.
pub fn f32_to_seed(value: f32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

/// Map a `WeatherKind` to a unique `u32` for use as a seed component.
pub fn weather_kind_discriminant(kind: WeatherKind) -> u32 {
    match kind {
        WeatherKind::Clear => 0,
        WeatherKind::Breezy => 1,
        WeatherKind::Windy => 2,
        WeatherKind::Rain => 3,
        WeatherKind::Storm => 4,
    }
}
