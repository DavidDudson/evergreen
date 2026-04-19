use bevy::prelude::*;

/// Possible weather states the world can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherKind {
    Clear,
    Breezy,
    Windy,
    Rain,
    Storm,
}

/// Minimum wind strength for the Clear weather state.
const CLEAR_WIND_MIN: f32 = 0.0;
/// Maximum wind strength for the Clear weather state.
const CLEAR_WIND_MAX: f32 = 0.2;
/// Minimum wind strength for the Breezy weather state.
const BREEZY_WIND_MIN: f32 = 0.3;
/// Maximum wind strength for the Breezy weather state.
const BREEZY_WIND_MAX: f32 = 0.5;
/// Minimum wind strength for the Windy weather state.
const WINDY_WIND_MIN: f32 = 0.6;
/// Maximum wind strength for the Windy weather state.
const WINDY_WIND_MAX: f32 = 0.8;
/// Minimum wind strength for the Rain weather state.
const RAIN_WIND_MIN: f32 = 0.5;
/// Maximum wind strength for the Rain weather state.
const RAIN_WIND_MAX: f32 = 0.7;
/// Minimum wind strength for the Storm weather state.
const STORM_WIND_MIN: f32 = 0.8;
/// Maximum wind strength for the Storm weather state.
const STORM_WIND_MAX: f32 = 1.0;

impl WeatherKind {
    /// Wind strength range for this weather state (min, max).
    pub fn wind_range(self) -> (f32, f32) {
        match self {
            Self::Clear => (CLEAR_WIND_MIN, CLEAR_WIND_MAX),
            Self::Breezy => (BREEZY_WIND_MIN, BREEZY_WIND_MAX),
            Self::Windy => (WINDY_WIND_MIN, WINDY_WIND_MAX),
            Self::Rain => (RAIN_WIND_MIN, RAIN_WIND_MAX),
            Self::Storm => (STORM_WIND_MIN, STORM_WIND_MAX),
        }
    }

    /// Whether this weather state spawns rain particles.
    pub fn has_rain(self) -> bool {
        matches!(self, Self::Rain | Self::Storm)
    }

    /// Whether this weather state spawns leaf particles.
    pub fn has_leaves(self) -> bool {
        matches!(self, Self::Breezy | Self::Windy | Self::Storm)
    }
}

/// Duration in seconds over which wind strength lerps to a new value.
const WIND_LERP_DURATION_SECS: f32 = 2.0;

/// Global weather state resource.
#[derive(Resource)]
pub struct WeatherState {
    /// Current active weather.
    pub current: WeatherKind,
    /// Target wind strength for the current state.
    pub target_wind: f32,
    /// Game-hour at which the next transition check occurs.
    pub next_check_hour: f32,
    /// Wind lerp timer (counts down from `WIND_LERP_DURATION_SECS`).
    pub wind_lerp_remaining: f32,
    /// Wind strength at the start of the current lerp.
    pub wind_lerp_start: f32,
}

impl WeatherState {
    /// Duration of wind lerp in seconds.
    pub const WIND_LERP_DURATION_SECS: f32 = WIND_LERP_DURATION_SECS;
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current: WeatherKind::Clear,
            target_wind: 0.1,
            next_check_hour: 11.0,
            wind_lerp_remaining: 0.0,
            wind_lerp_start: 0.0,
        }
    }
}

/// Visual variant of a weather particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleVariant {
    GreenLeaf,
    BrownLeaf,
    PaperScrap,
    Raindrop,
    Splash,
    Firefly,
    DustMote,
    FogPatch,
}

/// Marker and data for a weather particle entity.
#[derive(Component)]
pub struct WeatherParticle {
    /// Pixels per second.
    pub velocity: Vec2,
    /// Remaining lifetime.
    pub lifetime: Timer,
    /// Visual variant.
    pub variant: ParticleVariant,
}
