use std::collections::HashMap;

use bevy::prelude::*;
use level::area::AreaAlignment;
use models::decoration::Biome;

use crate::math::lerp;

/// Anchor alignment for the city biome.
const ALIGNMENT_CITY: f32 = 1.0;
/// Anchor alignment for the greenwood biome.
const ALIGNMENT_GREENWOOD: f32 = 50.0;
/// Anchor alignment for the darkwood biome.
const ALIGNMENT_DARKWOOD: f32 = 100.0;

/// City: bright, near-neutral daylight on stone; only a whisper of warmth.
const CITY_EXPOSURE: f32 = 0.05;
const CITY_TEMPERATURE: f32 = 0.04;
const CITY_TINT: f32 = 0.0;
const CITY_POST_SATURATION: f32 = 0.92;

/// Greenwood: vivid, neutral white balance.
const GREENWOOD_EXPOSURE: f32 = 0.0;
const GREENWOOD_TEMPERATURE: f32 = -0.02;
const GREENWOOD_TINT: f32 = 0.05;
const GREENWOOD_POST_SATURATION: f32 = 1.10;

/// Darkwood: cool, slightly underexposed, muted.
const DARKWOOD_EXPOSURE: f32 = -0.20;
const DARKWOOD_TEMPERATURE: f32 = -0.30;
const DARKWOOD_TINT: f32 = 0.10;
const DARKWOOD_POST_SATURATION: f32 = 0.85;

/// One biome's target grading values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradingPreset {
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub post_saturation: f32,
}

impl GradingPreset {
    pub const CITY: Self = Self {
        exposure: CITY_EXPOSURE,
        temperature: CITY_TEMPERATURE,
        tint: CITY_TINT,
        post_saturation: CITY_POST_SATURATION,
    };
    pub const GREENWOOD: Self = Self {
        exposure: GREENWOOD_EXPOSURE,
        temperature: GREENWOOD_TEMPERATURE,
        tint: GREENWOOD_TINT,
        post_saturation: GREENWOOD_POST_SATURATION,
    };
    pub const DARKWOOD: Self = Self {
        exposure: DARKWOOD_EXPOSURE,
        temperature: DARKWOOD_TEMPERATURE,
        tint: DARKWOOD_TINT,
        post_saturation: DARKWOOD_POST_SATURATION,
    };

    /// Linear interpolation between two presets by `t` in `[0, 1]`.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            exposure: lerp(self.exposure, other.exposure, t),
            temperature: lerp(self.temperature, other.temperature, t),
            tint: lerp(self.tint, other.tint, t),
            post_saturation: lerp(self.post_saturation, other.post_saturation, t),
        }
    }
}

/// Per-biome grading presets, keyed by `Biome`.
///
/// Lookups via `target_for_alignment` interpolate between adjacent anchor
/// presets so transitions across alignment boundaries remain smooth.
#[derive(Resource, Debug, Clone)]
pub struct BiomePresets {
    presets: HashMap<Biome, GradingPreset>,
}

impl BiomePresets {
    /// Get the preset for a specific biome, falling back to greenwood if missing.
    pub fn get(&self, biome: Biome) -> GradingPreset {
        self.presets
            .get(&biome)
            .copied()
            .unwrap_or(GradingPreset::GREENWOOD)
    }

    /// Map an alignment value to a target grading by interpolating between
    /// the city, greenwood, and darkwood anchor presets.
    pub fn target_for_alignment(&self, alignment: AreaAlignment) -> GradingPreset {
        let a = f32::from(alignment.clamp(1, 100));
        let city = self.get(Biome::City);
        let greenwood = self.get(Biome::Greenwood);
        let darkwood = self.get(Biome::Darkwood);
        if a <= ALIGNMENT_GREENWOOD {
            let t = (a - ALIGNMENT_CITY) / (ALIGNMENT_GREENWOOD - ALIGNMENT_CITY);
            city.lerp(greenwood, t)
        } else {
            let t = (a - ALIGNMENT_GREENWOOD) / (ALIGNMENT_DARKWOOD - ALIGNMENT_GREENWOOD);
            greenwood.lerp(darkwood, t)
        }
    }
}

impl Default for BiomePresets {
    fn default() -> Self {
        let mut presets = HashMap::new();
        presets.insert(Biome::City, GradingPreset::CITY);
        presets.insert(Biome::Greenwood, GradingPreset::GREENWOOD);
        presets.insert(Biome::Darkwood, GradingPreset::DARKWOOD);
        Self { presets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    fn approx_target(actual: GradingPreset, expected: GradingPreset) {
        approx(actual.exposure, expected.exposure);
        approx(actual.temperature, expected.temperature);
        approx(actual.tint, expected.tint);
        approx(actual.post_saturation, expected.post_saturation);
    }

    #[test]
    fn target_at_city_anchor_returns_city() {
        let presets = BiomePresets::default();
        approx_target(presets.target_for_alignment(1), GradingPreset::CITY);
    }

    #[test]
    fn target_at_greenwood_anchor_returns_greenwood() {
        let presets = BiomePresets::default();
        approx_target(presets.target_for_alignment(50), GradingPreset::GREENWOOD);
    }

    #[test]
    fn target_at_darkwood_anchor_returns_darkwood() {
        let presets = BiomePresets::default();
        approx_target(presets.target_for_alignment(100), GradingPreset::DARKWOOD);
    }

    #[test]
    fn target_midway_city_greenwood_is_average() {
        let presets = BiomePresets::default();
        let t = presets.target_for_alignment(25);
        let expected_exposure = lerp(CITY_EXPOSURE, GREENWOOD_EXPOSURE, (25.0 - 1.0) / 49.0);
        approx(t.exposure, expected_exposure);
    }

    #[test]
    fn target_clamps_below_one() {
        let presets = BiomePresets::default();
        let t = presets.target_for_alignment(0);
        approx(t.exposure, CITY_EXPOSURE);
    }
}
