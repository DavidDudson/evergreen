use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};

/// Bloom intensity tuned low for pixel art -- only true highlights halo.
const BLOOM_INTENSITY: f32 = 0.15;
/// Low-frequency bloom slice contribution.
const BLOOM_LOW_FREQUENCY_BOOST: f32 = 0.7;
/// Curvature of the boost falloff.
const BLOOM_LOW_FREQUENCY_BOOST_CURVATURE: f32 = 0.95;
/// Highpass filter response (1.0 = no highpass; lower preserves more haze).
const BLOOM_HIGH_PASS_FREQUENCY: f32 = 1.0;
/// Threshold (in HDR linear units) above which a pixel contributes to bloom.
/// Pixel art bloom needs a high threshold so only emissive (>1.0) sprites trigger.
const BLOOM_PREFILTER_THRESHOLD: f32 = 0.9;
/// Soft knee around the threshold for smoother transitions.
const BLOOM_PREFILTER_THRESHOLD_SOFTNESS: f32 = 0.4;
/// Maximum dimension of the bloom mip chain, in pixels. Matches Bevy's default.
const BLOOM_MAX_MIP_DIMENSION_PX: u32 = 512;

/// Returns a `Bloom` component tuned for the project's pixel-art look.
pub fn pixel_art_bloom() -> Bloom {
    Bloom {
        intensity: BLOOM_INTENSITY,
        low_frequency_boost: BLOOM_LOW_FREQUENCY_BOOST,
        low_frequency_boost_curvature: BLOOM_LOW_FREQUENCY_BOOST_CURVATURE,
        high_pass_frequency: BLOOM_HIGH_PASS_FREQUENCY,
        prefilter: BloomPrefilter {
            threshold: BLOOM_PREFILTER_THRESHOLD,
            threshold_softness: BLOOM_PREFILTER_THRESHOLD_SOFTNESS,
        },
        composite_mode: BloomCompositeMode::Additive,
        max_mip_dimension: BLOOM_MAX_MIP_DIMENSION_PX,
        scale: bevy::math::Vec2::ONE,
    }
}
