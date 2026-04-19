//! Per-asset drop-shadow geometry. Used by `level::shadows::spawn_drop_shadow`.

use bevy::math::Vec2;

pub const PLAYER_SHADOW_HALF_PX: Vec2 = Vec2::new(8.0, 3.0);
pub const PLAYER_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const NPC_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const NPC_SHADOW_OFFSET_Y_PX: f32 = -10.0;

pub const TREE_SHADOW_HALF_PX: Vec2 = Vec2::new(18.0, 5.0);
/// Tree sprite is anchored at BOTTOM_CENTER, so the entity Transform sits
/// at the trunk base -- shadow rests at the same y.
pub const TREE_SHADOW_OFFSET_Y_PX: f32 = 0.0;

pub const GRASS_SHADOW_HALF_PX: Vec2 = Vec2::new(4.0, 1.5);
pub const GRASS_SHADOW_OFFSET_Y_PX: f32 = -2.0;

pub const CREATURE_SHADOW_HALF_PX: Vec2 = Vec2::new(3.0, 1.0);
pub const CREATURE_SHADOW_OFFSET_Y_PX: f32 = -3.0;

pub const GALEN_SHADOW_HALF_PX: Vec2 = Vec2::new(7.0, 2.5);
pub const GALEN_SHADOW_OFFSET_Y_PX: f32 = -10.0;
