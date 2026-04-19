//! Pure data describing 2D-light occluder geometry for game asset families.
//! Consumers in `level/` use these dimensions to spawn `LightOccluder2d`
//! rect children directly. No Bevy components live here.
//!
//! Tree sprites are 48x64 anchored BOTTOM_CENTER -- offsets are measured
//! UPWARDS from the entity Transform.

use bevy::math::Vec2;

// -- Tree --------------------------------------------------------------------
/// Trunk occluder: ~16 px wide, ~20 px tall starting at the base.
pub const TREE_TRUNK_HALF_PX: Vec2 = Vec2::new(8.0, 10.0);
/// Trunk centered 10 px above the bottom-center anchor.
pub const TREE_TRUNK_OFFSET_PX: Vec2 = Vec2::new(0.0, 10.0);

/// Canopy occluder: ~40 px wide, ~28 px tall in the upper half.
pub const TREE_CANOPY_HALF_PX: Vec2 = Vec2::new(20.0, 14.0);
/// Canopy centered 36 px above anchor (mid-canopy).
pub const TREE_CANOPY_OFFSET_PX: Vec2 = Vec2::new(0.0, 36.0);

// -- NPC ---------------------------------------------------------------------
/// NPC body: ~12 wide, ~20 tall. Wired in P2-T7.
pub const NPC_BODY_HALF_PX: Vec2 = Vec2::new(6.0, 10.0);
/// NPC body offset (assumes center-anchored sprite).
pub const NPC_BODY_OFFSET_PX: Vec2 = Vec2::new(0.0, 0.0);

// -- Grass tuft --------------------------------------------------------------
/// Grass tuft: small, low to ground. Wired in P2-T8.
pub const GRASS_OCCLUDER_HALF_PX: Vec2 = Vec2::new(3.0, 2.0);
/// Grass occluder offset (centered on sprite).
pub const GRASS_OCCLUDER_OFFSET_PX: Vec2 = Vec2::new(0.0, 0.0);
