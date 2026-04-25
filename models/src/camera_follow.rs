use bevy::prelude::Component;

/// Marker component for the entity the camera should follow.
///
/// Decouples the camera crate from the player crate -- any entity with this
/// marker (typically the player) drives the smooth-follow camera.
#[derive(Component)]
pub struct CameraFollow;
