use bevy::prelude::Resource;

/// Fraction of the area half-extent inside which the camera stays locked to
/// the area centre.  Outside this fraction the camera blends toward the
/// follow target, reaching full tracking at the area boundary.
const DEFAULT_DEAD_ZONE_FRAC: f32 = 0.6;

/// Time-based lerp speed (per second) for camera movement toward the
/// dialogue midpoint while a conversation is active.
const DEFAULT_LERP_SPEED_PER_SEC: f32 = 5.0;

/// Time-based lerp speed (per second) for the residual offset returning to
/// zero after a dialogue ends.
const DEFAULT_DIALOGUE_RETURN_SPEED_PER_SEC: f32 = 5.0;

/// Tunable parameters shared by all camera systems.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraConfig {
    /// Fraction of area half-extent that is fully area-locked (0..1).
    pub dead_zone_frac: f32,
    /// Lerp speed (per second) toward the dialogue midpoint.
    pub lerp_speed_per_sec: f32,
    /// Lerp speed (per second) for the post-dialogue return slide.
    pub dialogue_return_speed_per_sec: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            dead_zone_frac: DEFAULT_DEAD_ZONE_FRAC,
            lerp_speed_per_sec: DEFAULT_LERP_SPEED_PER_SEC,
            dialogue_return_speed_per_sec: DEFAULT_DIALOGUE_RETURN_SPEED_PER_SEC,
        }
    }
}
