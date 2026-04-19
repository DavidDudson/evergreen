//! Shared spawn helper for `LightOccluder2d` child entities. Used by
//! `scenery`, `npcs`, and `grass` spawn paths.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_light_2d::prelude::{LightOccluder2d, LightOccluder2dShape};

/// Spawn a single rect-shaped occluder as a child of `parent` at `offset`
/// (relative to parent transform) with the given `half_size`.
pub fn spawn_occluder(
    commands: &mut Commands,
    parent: Entity,
    half_size: Vec2,
    offset: Vec2,
) {
    commands.spawn((
        LightOccluder2d {
            shape: LightOccluder2dShape::Rectangle { half_size },
        },
        Transform::from_translation(offset.extend(0.0)),
        ChildOf(parent),
    ));
}
