use bevy::prelude::*;

/// Default bloom transition speed (per second).
const DEFAULT_BLOOM_LERP_SPEED_PER_SEC: f32 = 2.5;
/// Default color-grading transition speed between areas (per second).
const DEFAULT_GRADING_LERP_SPEED_PER_SEC: f32 = 2.5;
/// Default atmosphere darkness transition speed between areas (per second).
const DEFAULT_ATMOSPHERE_LERP_SPEED_PER_SEC: f32 = 3.0;

/// Tunable lerp speeds for post-processing transitions across area changes.
///
/// Each field is the alpha multiplier applied to `time.delta_secs()` before
/// being clamped to `[0, 1]`. Higher values converge faster.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct PostProcessLerpConfig {
    pub bloom_speed_per_sec: f32,
    pub grading_speed_per_sec: f32,
    pub atmosphere_speed_per_sec: f32,
}

impl Default for PostProcessLerpConfig {
    fn default() -> Self {
        Self {
            bloom_speed_per_sec: DEFAULT_BLOOM_LERP_SPEED_PER_SEC,
            grading_speed_per_sec: DEFAULT_GRADING_LERP_SPEED_PER_SEC,
            atmosphere_speed_per_sec: DEFAULT_ATMOSPHERE_LERP_SPEED_PER_SEC,
        }
    }
}
