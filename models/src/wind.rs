use bevy::prelude::Resource;

/// Global wind strength, driven by the weather system.
///
/// Ranges from 0.0 (calm) to 1.0 (storm-force).
/// Read by the grass sway animation system.
#[derive(Resource, Clone, Copy)]
pub struct WindStrength(pub f32);

impl Default for WindStrength {
    fn default() -> Self {
        Self(0.0)
    }
}
