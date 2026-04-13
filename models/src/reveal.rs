use bevy::prelude::Component;

/// Marks an entity that can reveal its stump when the player walks behind it.
#[derive(Component)]
pub struct Revealable {
    /// How far above the entity base the canopy/top extends (pixels).
    pub canopy_height_px: f32,
    /// Half-width of the reveal trigger zone (pixels).
    pub half_width_px: f32,
    /// Minimum alpha when revealed (0.0 = full swap to stump, 0.3 = semi-transparent).
    pub revealed_full_alpha: f32,
}

/// Current state of the reveal transition.
#[derive(Component, Default)]
pub enum RevealState {
    #[default]
    Full,
    /// Transitioning to stump. Progress 0.0 (full) -> 1.0 (stump).
    Revealing(f32),
    Revealed,
    /// Transitioning back to full. Progress 0.0 (stump) -> 1.0 (full).
    Hiding(f32),
}

/// Marker for the full (canopy) sprite child.
#[derive(Component)]
pub struct FullSprite;

/// Marker for the stump sprite child.
#[derive(Component)]
pub struct StumpSprite;
