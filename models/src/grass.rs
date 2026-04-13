use bevy::prelude::Component;

/// Marker for decorative grass tuft entities.
#[derive(Component, Default)]
pub struct GrassTuft;

/// Per-entity phase offset for wind sway animation.
///
/// The sway system applies:
/// `rotation = sin(time * FREQUENCY + phase) * MAX_ANGLE * wind_strength`
#[derive(Component)]
pub struct WindSway {
    /// Random phase offset in radians, set at spawn time.
    pub phase: f32,
}
