use crate::speed::Speed;
use bevy::prelude::*;

/// Marker component for the player entity.
#[derive(Component)]
#[require(Speed)]
pub struct Player;

/// Marker component placed on entities the player should hop on top of
/// (e.g. stepping stones across rivers). Decouples the player crate from
/// any specific level scenery type.
#[derive(Component)]
pub struct HopTrigger;
