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

/// Wind direction in radians (0 = east, PI/2 = north, PI = west, 3PI/2 = south).
/// Changes when weather state transitions.
#[derive(Resource, Clone, Copy)]
pub struct WindDirection(pub f32);

impl Default for WindDirection {
    fn default() -> Self {
        Self(0.0) // default: blowing east
    }
}

impl WindDirection {
    /// Unit vector in the wind direction.
    pub fn as_vec2(self) -> bevy::math::Vec2 {
        bevy::math::Vec2::new(self.0.cos(), self.0.sin())
    }

    /// Compass label for the direction the wind is blowing TOWARD.
    pub fn label(self) -> &'static str {
        use std::f32::consts::PI;
        let angle = self.0.rem_euclid(2.0 * PI);
        match angle {
            a if a < PI * 0.125 => "E",
            a if a < PI * 0.375 => "NE",
            a if a < PI * 0.625 => "N",
            a if a < PI * 0.875 => "NW",
            a if a < PI * 1.125 => "W",
            a if a < PI * 1.375 => "SW",
            a if a < PI * 1.625 => "S",
            a if a < PI * 1.875 => "SE",
            _ => "E",
        }
    }
}
