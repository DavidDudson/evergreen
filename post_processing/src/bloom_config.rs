use bevy::math::Vec2;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;

/// Bloom intensity tuned low for pixel art -- only true highlights halo.
const BLOOM_INTENSITY: f32 = 0.15;
/// Low-frequency bloom slice contribution.
const BLOOM_LOW_FREQUENCY_BOOST: f32 = 0.7;
/// Curvature of the boost falloff.
const BLOOM_LOW_FREQUENCY_BOOST_CURVATURE: f32 = 0.95;
/// Highpass filter response (1.0 = no highpass; lower preserves more haze).
const BLOOM_HIGH_PASS_FREQUENCY: f32 = 1.0;
/// Threshold (in HDR linear units) above which a pixel contributes to bloom.
/// Set at 1.0 so only emissive (>1.0) sprites trigger; standard sprites stay sharp.
const BLOOM_PREFILTER_THRESHOLD: f32 = 1.0;
/// Soft knee around the threshold for smoother transitions.
const BLOOM_PREFILTER_THRESHOLD_SOFTNESS: f32 = 0.4;
/// Maximum dimension of the bloom mip chain, in pixels. Matches Bevy's default.
const BLOOM_MAX_MIP_DIMENSION_PX: u32 = 512;

/// Tunable bloom parameters, surfaced as a resource so settings menus / debug
/// overlays can tweak the look without rebuilding.
///
/// `pixel_art_bloom` reads this resource's defaults to construct the camera's
/// `Bloom` component, keeping the camera setup API stable.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BloomConfig {
    pub intensity: f32,
    pub low_frequency_boost: f32,
    pub low_frequency_boost_curvature: f32,
    pub high_pass_frequency: f32,
    pub prefilter_threshold: f32,
    pub prefilter_threshold_softness: f32,
    pub max_mip_dimension_px: u32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            intensity: BLOOM_INTENSITY,
            low_frequency_boost: BLOOM_LOW_FREQUENCY_BOOST,
            low_frequency_boost_curvature: BLOOM_LOW_FREQUENCY_BOOST_CURVATURE,
            high_pass_frequency: BLOOM_HIGH_PASS_FREQUENCY,
            prefilter_threshold: BLOOM_PREFILTER_THRESHOLD,
            prefilter_threshold_softness: BLOOM_PREFILTER_THRESHOLD_SOFTNESS,
            max_mip_dimension_px: BLOOM_MAX_MIP_DIMENSION_PX,
        }
    }
}

impl BloomConfig {
    /// Build a `Bloom` component from this config.
    pub fn to_bloom(self) -> Bloom {
        Bloom {
            intensity: self.intensity,
            low_frequency_boost: self.low_frequency_boost,
            low_frequency_boost_curvature: self.low_frequency_boost_curvature,
            high_pass_frequency: self.high_pass_frequency,
            prefilter: BloomPrefilter {
                threshold: self.prefilter_threshold,
                threshold_softness: self.prefilter_threshold_softness,
            },
            composite_mode: BloomCompositeMode::Additive,
            max_mip_dimension: self.max_mip_dimension_px,
            scale: Vec2::ONE,
        }
    }
}
